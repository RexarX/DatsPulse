use crate::types::*;
use anyhow::Result;
use bevy::prelude::*;
use bevy_tokio_tasks::{TaskContext, TokioTasksRuntime};
use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

#[derive(Resource, Clone)]
pub struct ServerClient {
    client: Client,
    config: ServerConfig,
    registered: bool,
    registration_data: Option<ApiRegistrationResponse>,
}

#[derive(Resource)]
pub struct ServerTicker {
    pub timer: Timer,
    pub registration_timer: Timer,
    pub registration_attempts: u32,
    pub waiting_for_lobby: bool,
    pub lobby_wait_timer: Timer,
    pub registration_backoff: f32,
}

impl ServerClient {
    pub fn new(config: ServerConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("DatsPulse-Bot/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            registered: false,
            registration_data: None,
        }
    }

    pub fn is_registered(&self) -> bool {
        self.registered
    }

    pub fn get_registration_data(&self) -> Option<&ApiRegistrationResponse> {
        self.registration_data.as_ref()
    }

    pub async fn register(&mut self) -> Result<ApiRegistrationResponse> {
        let url = format!("{}/register", self.config.url.trim_end_matches('/'));

        info!(target: "server", "Registering at: {}", url);
        //info!(target: "server", "Using token: {}...", self.config.token[..8.min(self.config.token.len())]);
        let response = self
            .client
            .post(&url)
            .header("X-Auth-Token", &self.config.token)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if response.status().is_success() {
            let registration: ApiRegistrationResponse = response.json().await?;
            self.registered = true;
            self.registration_data = Some(registration.clone());
            info!(target: "server", "Registration successful: realm={}, name={}",
                registration.realm, registration.name);
            Ok(registration)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!(target: "server", "Registration failed: {} - {}", status, error_text);
            Err(anyhow::anyhow!(
                "Registration failed: {} - {}",
                status,
                error_text
            ))
        }
    }

    pub async fn get_arena_state(&self) -> Result<ApiArenaResponse> {
        if !self.registered {
            return Err(anyhow::anyhow!("Not registered"));
        }

        self.get_endpoint("arena").await
    }

    pub async fn send_moves(&self, moves: &ApiMoveRequest) -> Result<ApiMoveResponse> {
        if !self.registered {
            return Err(anyhow::anyhow!("Not registered"));
        }

        self.post_endpoint("move", moves).await
    }

    pub async fn get_logs(&self) -> Result<Vec<ApiLogMessage>> {
        if !self.registered {
            return Err(anyhow::anyhow!("Not registered"));
        }

        self.get_endpoint("logs").await
    }

    async fn get_endpoint<T>(&self, endpoint: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = format!(
            "{}/{}",
            self.config.url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        );

        debug!(target: "server", "GET {}", url);

        let response = self
            .client
            .get(&url)
            .header("X-Auth-Token", &self.config.token)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            debug!(target: "server", "Response body: {}", response_text);

            match serde_json::from_str::<T>(&response_text) {
                Ok(data) => {
                    debug!(target: "server", "GET {} succeeded", endpoint);
                    Ok(data)
                }
                Err(e) => {
                    error!(target: "server", "Failed to parse JSON response for {}: {}", endpoint, e);
                    error!(target: "server", "Response body was: {}", response_text);
                    Err(anyhow::anyhow!(
                        "JSON parsing error: {} - Response: {}",
                        e,
                        response_text
                    ))
                }
            }
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!(target: "server", "GET {} failed: {} - {}", endpoint, status, error_text);
            Err(anyhow::anyhow!("Server error: {} - {}", status, error_text))
        }
    }

    async fn post_endpoint<T, R>(&self, endpoint: &str, data: &T) -> Result<R>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let url = format!(
            "{}/{}",
            self.config.url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        );

        debug!(target: "server", "POST {}", url);

        let response = self
            .client
            .post(&url)
            .header("X-Auth-Token", &self.config.token)
            .header("Content-Type", "application/json")
            .json(data)
            .send()
            .await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            debug!(target: "server", "Response body: {}", response_text);

            match serde_json::from_str::<R>(&response_text) {
                Ok(response_data) => {
                    debug!(target: "server", "POST {} succeeded", endpoint);
                    Ok(response_data)
                }
                Err(e) => {
                    error!(target: "server", "Failed to parse JSON response for {}: {}", endpoint, e);
                    error!(target: "server", "Response body was: {}", response_text);
                    Err(anyhow::anyhow!(
                        "JSON parsing error: {} - Response: {}",
                        e,
                        response_text
                    ))
                }
            }
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!(target: "server", "POST {} failed: {} - {}", endpoint, status, error_text);
            Err(anyhow::anyhow!("Server error: {} - {}", status, error_text))
        }
    }
}

pub fn handle_game_move_commands(
    mut commands: Commands,
    mut move_command_events: EventReader<MoveCommandEvent>,
    server_client: Res<ServerClient>,
    tokio_tasks: Res<TokioTasksRuntime>,
) {
    if move_command_events.is_empty() {
        return;
    }

    let mut api_commands = Vec::new();

    for event in move_command_events.read() {
        // Convert HexCoord path to ApiHex path
        let api_path: Vec<ApiHex> = event.path.iter().map(|coord| (*coord).into()).collect();

        // Create API move command
        api_commands.push(ApiMoveCommand {
            ant: event.ant_id.clone(),
            path: api_path,
        });
    }

    if !api_commands.is_empty() {
        // Clone the server client for the async task
        let client = ServerClient {
            client: server_client.client.clone(),
            config: server_client.config.clone(),
            registered: server_client.registered,
            registration_data: server_client.registration_data.clone(),
        };

        let move_request = ApiMoveRequest {
            moves: api_commands.clone(),
        };
        info!(target: "server", "Sending {} move commands directly to server", api_commands.len());

        spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
            client.send_moves(&move_request).await
        });
    }
}

#[derive(Component)]
pub struct ServerTask<T> {
    pub handle: Option<JoinHandle<T>>,
}

impl<T> ServerTask<T> {
    pub fn new(handle: JoinHandle<T>) -> Self {
        Self {
            handle: Some(handle),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.handle.as_ref().map_or(true, |h| h.is_finished())
    }

    pub fn take_handle(&mut self) -> Option<JoinHandle<T>> {
        self.handle.take()
    }
}

fn spawn_server_task<T, Fut>(
    commands: &mut Commands,
    tokio_tasks: &TokioTasksRuntime,
    fut: impl FnOnce(TaskContext) -> Fut + Send + 'static,
) where
    T: Send + 'static,
    Fut: std::future::Future<Output = T> + Send + 'static,
{
    let handle = tokio_tasks.spawn_background_task(fut);
    commands.spawn(ServerTask::new(handle));
}

pub fn setup_server_client(mut commands: Commands, config: Res<ServerConfig>) {
    let client = ServerClient::new(config.clone());
    commands.insert_resource(client);

    let game_timer = Timer::new(config.tick_rate, TimerMode::Repeating);
    let registration_timer = Timer::new(Duration::from_secs(2), TimerMode::Repeating);
    let lobby_wait_timer = Timer::new(Duration::from_secs(30), TimerMode::Repeating); // Try every 30s when waiting

    commands.insert_resource(ServerTicker {
        timer: game_timer,
        registration_timer,
        registration_attempts: 0,
        waiting_for_lobby: false,
        lobby_wait_timer,
        registration_backoff: 2.0, // start with 2 seconds
    });

    info!(target: "server", "Server client initialized with URL: {}", config.url);
}

pub fn server_tick_system(
    mut commands: Commands,
    mut server_ticker: ResMut<ServerTicker>,
    server_client: Res<ServerClient>,
    time: Res<Time>,
    tokio_tasks: Res<TokioTasksRuntime>,
) {
    // Handle lobby waiting
    if server_ticker.waiting_for_lobby {
        server_ticker.lobby_wait_timer.tick(time.delta());
        if server_ticker.lobby_wait_timer.just_finished() {
            try_register(&mut commands, &tokio_tasks, &server_client.config);
            info!(target: "server", "Waiting for next round... trying to register again.");
        }
        return;
    }

    // Handle registration
    server_ticker.registration_timer.tick(time.delta());
    if server_ticker.registration_timer.just_finished() && !server_client.registered {
        try_register(&mut commands, &tokio_tasks, &server_client.config);
        info!(target: "server", "Registration attempt #{}", server_ticker.registration_attempts + 1);

        let new_backoff = (server_ticker.registration_backoff * 1.5).min(60.0);
        server_ticker.registration_attempts += 1;
        server_ticker.registration_backoff = new_backoff;
        server_ticker
            .registration_timer
            .set_duration(Duration::from_secs_f32(new_backoff));
        server_ticker.registration_timer.reset();
    }

    // Handle arena state requests
    server_ticker.timer.tick(time.delta());
    if server_ticker.timer.just_finished() && server_client.registered {
        info!(target: "server", "Requesting arena state (registered: {})", server_client.registered);

        let client = ServerClient {
            client: server_client.client.clone(),
            config: server_client.config.clone(),
            registered: server_client.registered,
            registration_data: server_client.registration_data.clone(),
        };

        spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
            let result = client.get_arena_state().await;
            info!(target: "server", "Arena state request result: {:?}", result.is_ok());
            result
        });
    }
}

fn try_register(commands: &mut Commands, tokio_tasks: &TokioTasksRuntime, config: &ServerConfig) {
    let config = config.clone();
    spawn_server_task(commands, tokio_tasks, move |_ctx| async move {
        let mut client = ServerClient::new(config);
        client.register().await
    });
}

pub fn handle_registration_tasks(
    mut commands: Commands,
    mut server_client: ResMut<ServerClient>,
    mut server_ticker: ResMut<ServerTicker>,
    mut connection_state: ResMut<ConnectionState>,
    mut connection_events: EventWriter<ConnectionEvent>,
    mut registration_events: EventWriter<ApiRegistrationEvent>,
    mut query: Query<(Entity, &mut ServerTask<Result<ApiRegistrationResponse>>)>,
) {
    for (entity, mut task) in &mut query {
        if let Some(handle) = task.take_handle() {
            if let Ok(reg_result) = futures::executor::block_on(handle) {
                match reg_result {
                    Ok(registration) => {
                        server_client.registered = true;
                        server_client.registration_data = Some(registration.clone());
                        server_ticker.registration_attempts = 0;
                        server_ticker.waiting_for_lobby = false;
                        server_ticker.registration_backoff = 2.0;

                        // Set timer after all mutable borrows
                        let new_backoff = server_ticker.registration_backoff;
                        commands.queue(move |world: &mut bevy::prelude::World| {
                            let mut ticker = world.resource_mut::<ServerTicker>();
                            ticker
                                .registration_timer
                                .set_duration(Duration::from_secs_f32(new_backoff));
                            ticker.registration_timer.reset();
                        });

                        connection_state.connected = true;
                        connection_state.connection_message = format!(
                            "Registered successfully: {} ({})",
                            registration.name, registration.realm
                        );
                        connection_state.last_connection_attempt = Some(chrono::Utc::now());

                        connection_events.write(ConnectionEvent {
                            connected: true,
                            message: "Registration successful".to_string(),
                        });

                        registration_events.write(ApiRegistrationEvent(registration));
                        info!(target: "server", "Registration completed successfully");
                    }
                    Err(e) => {
                        let msg = format!("{}", e);
                        if msg.contains("no active game") || msg.contains("lobby ended") {
                            server_ticker.waiting_for_lobby = true;
                            server_ticker.lobby_wait_timer.reset();
                            server_ticker.registration_attempts = 0;
                            server_ticker.registration_backoff = 2.0;
                            connection_state.connection_message = extract_next_round_info(&msg);
                            connection_state.connected = false;
                            info!(target: "server", "No active game, waiting for next round...");
                        } else {
                            server_client.registered = false;
                            connection_state.connected = false;
                            connection_state.connection_message =
                                format!("Registration failed: {}", e);
                            connection_state.last_connection_attempt = Some(chrono::Utc::now());

                            connection_events.write(ConnectionEvent {
                                connected: false,
                                message: format!("Registration failed: {}", e),
                            });

                            error!(target: "server", "Registration failed: {}", e);
                        }
                    }
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

fn extract_next_round_info(msg: &str) -> String {
    // Try to extract next round info from error message
    if let Some(start) = msg.find("next rounds:") {
        let after_rounds = &msg[start..];
        if let Some(bracket_start) = after_rounds.find('[') {
            if let Some(bracket_end) = after_rounds.find(']') {
                let round_info = &after_rounds[bracket_start + 1..bracket_end];
                return format!("Next round: {}", round_info);
            }
        }
    }
    "Waiting for next round...".to_string()
}

pub fn handle_arena_state_tasks(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut arena_events: EventWriter<ApiArenaEvent>,
    mut query: Query<(Entity, &mut ServerTask<Result<ApiArenaResponse>>)>,
) {
    for (entity, mut task) in &mut query {
        if !task.is_finished() {
            continue;
        }

        if let Some(handle) = task.take_handle() {
            match futures::executor::block_on(handle) {
                Ok(Ok(arena_response)) => {
                    *game_state = GameState::from_api_response(&arena_response);
                    arena_events.write(ApiArenaEvent(arena_response));
                    debug!(target: "server", "Arena state updated");
                }
                Ok(Err(e)) => {
                    error!(target: "server", "Failed to fetch arena state: {e}");
                    game_state.connected = false;
                }
                Err(e) => {
                    error!(target: "server", "Arena state task join error: {e}");
                }
            }
        }

        commands.entity(entity).despawn();
    }
}

pub fn handle_move_commands(
    mut commands: Commands,
    mut move_events: EventReader<ApiMoveEvent>,
    server_client: Res<ServerClient>,
    tokio_tasks: Res<TokioTasksRuntime>,
) {
    for event in move_events.read() {
        if !server_client.registered {
            warn!(target: "server", "Attempted to send moves while not registered");
            continue;
        }

        // Clone the entire client with its registration state
        let client = ServerClient {
            client: server_client.client.clone(),
            config: server_client.config.clone(),
            registered: server_client.registered,
            registration_data: server_client.registration_data.clone(),
        };
        let moves = event.0.clone();

        info!(target: "server", "Sending {} move commands", moves.moves.len());

        spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
            client.send_moves(&moves).await
        });
    }
}

pub fn handle_move_response_tasks(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut arena_events: EventWriter<ApiArenaEvent>,
    mut query: Query<(Entity, &mut ServerTask<Result<ApiMoveResponse>>)>,
) {
    for (entity, mut task) in &mut query {
        if !task.is_finished() {
            continue;
        }

        if let Some(handle) = task.take_handle() {
            match futures::executor::block_on(handle) {
                Ok(Ok(move_response)) => {
                    // Convert move response to arena response format
                    let arena_response = ApiArenaResponse {
                        ants: move_response.ants,
                        enemies: move_response.enemies,
                        food: move_response.food,
                        home: move_response.home,
                        map: move_response.map,
                        next_turn_in: move_response.next_turn_in,
                        score: move_response.score,
                        spot: move_response.spot,
                        turn_no: move_response.turn_no,
                    };

                    *game_state = GameState::from_api_response(&arena_response);
                    arena_events.write(ApiArenaEvent(arena_response));

                    if !move_response.errors.is_empty() {
                        warn!(target: "server", "Move errors: {:?}", move_response.errors);
                    }

                    info!(target: "server", "Move response processed successfully");
                }
                Ok(Err(e)) => {
                    error!(target: "server", "Move command failed: {}", e);
                }
                Err(e) => {
                    error!(target: "server", "Move response task join error: {e}");
                }
            }
        }

        commands.entity(entity).despawn();
    }
}

pub fn handle_logs_requests(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    server_client: Res<ServerClient>,
    tokio_tasks: Res<TokioTasksRuntime>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyL) && server_client.registered {
        // Clone the entire client with its registration state
        let client = ServerClient {
            client: server_client.client.clone(),
            config: server_client.config.clone(),
            registered: server_client.registered,
            registration_data: server_client.registration_data.clone(),
        };

        spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
            client.get_logs().await
        });

        info!(target: "server", "Requesting game logs");
    }
}

pub fn handle_logs_response_tasks(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ServerTask<Result<Vec<ApiLogMessage>>>)>,
) {
    for (entity, mut task) in &mut query {
        if !task.is_finished() {
            continue;
        }

        if let Some(handle) = task.take_handle() {
            match futures::executor::block_on(handle) {
                Ok(Ok(logs)) => {
                    info!(target: "server", "Received {} log messages", logs.len());
                    for log in logs {
                        info!(target: "server", "[{}] {}", log.time, log.message);
                    }
                }
                Ok(Err(e)) => {
                    error!(target: "server", "Failed to fetch logs: {}", e);
                }
                Err(e) => {
                    error!(target: "server", "Logs task join error: {e}");
                }
            }
        }

        commands.entity(entity).despawn();
    }
}

pub fn handle_reconnect_requests(
    mut commands: Commands,
    mut reconnect_events: EventReader<ReconnectRequestEvent>,
    mut server_client: ResMut<ServerClient>,
    mut server_ticker: ResMut<ServerTicker>,
    mut connection_state: ResMut<ConnectionState>,
    tokio_tasks: Res<TokioTasksRuntime>,
) {
    for _event in reconnect_events.read() {
        server_client.registered = false;
        server_client.registration_data = None;
        server_ticker.registration_attempts = 0;

        connection_state.connected = false;
        connection_state.connection_message = "Reconnecting...".to_string();

        let config = server_client.config.clone();
        spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
            let mut client = ServerClient::new(config);
            client.register().await
        });

        info!(target: "server", "Reconnect requested - resetting registration");
    }
}

// Helper functions for creating move commands
pub fn create_move_command(ant_id: String, path: Vec<HexCoord>) -> ApiMoveCommand {
    ApiMoveCommand {
        ant: ant_id,
        path: path.into_iter().map(|coord| coord.into()).collect(),
    }
}

pub fn create_move_request(commands: Vec<ApiMoveCommand>) -> ApiMoveRequest {
    ApiMoveRequest { moves: commands }
}

// System for automatic move generation (example implementation)
pub fn auto_move_system(
    mut move_events: EventWriter<ApiMoveEvent>,
    mut arena_events: EventReader<ApiArenaEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Only send moves when user presses M key for now
    if !keyboard_input.just_pressed(KeyCode::KeyM) {
        return;
    }

    // Process the latest arena state
    let latest_arena = arena_events.read().last();
    if let Some(arena_event) = latest_arena {
        let arena = &arena_event.0;

        // Create simple move commands for all ants
        let mut commands = Vec::new();

        for ant in &arena.ants {
            // Simple AI: move towards the closest food or explore randomly
            let target = find_closest_food(&arena.food, &HexCoord::new(ant.q, ant.r))
                .unwrap_or_else(|| {
                    // Random exploration
                    let neighbors = HexCoord::new(ant.q, ant.r).neighbors();
                    neighbors
                        .into_iter()
                        .next()
                        .unwrap_or(HexCoord::new(ant.q, ant.r))
                });

            // Create a simple path (just one step towards target)
            let path = vec![target];
            commands.push(create_move_command(ant.id.clone(), path));
        }

        if !commands.is_empty() {
            let move_request = create_move_request(commands);
            move_events.write(ApiMoveEvent(move_request));
            info!(target: "server", "Sent move commands for {} ants", arena.ants.len());
        }
    }
}

fn find_closest_food(food_list: &[ApiFoodOnMap], position: &HexCoord) -> Option<HexCoord> {
    food_list
        .iter()
        .map(|food| HexCoord::new(food.q, food.r))
        .min_by_key(|food_pos| position.distance(food_pos))
}

// Connection status monitoring
pub fn monitor_connection_system(
    server_client: Res<ServerClient>,
    mut connection_state: ResMut<ConnectionState>,
    time: Res<Time>,
) {
    // Update connection state based on registration status
    if server_client.registered != connection_state.connected {
        connection_state.connected = server_client.registered;

        if server_client.registered {
            if let Some(reg_data) = server_client.get_registration_data() {
                connection_state.connection_message =
                    format!("Connected to {} as {}", reg_data.realm, reg_data.name);
            } else {
                connection_state.connection_message = "Connected".to_string();
            }
        } else {
            connection_state.connection_message = "Disconnected".to_string();
        }
    }
}

// Rate limiting helper
#[derive(Resource)]
pub struct RateLimiter {
    last_request_time: std::time::Instant,
    min_interval: Duration,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self {
            last_request_time: std::time::Instant::now() - Duration::from_secs(1),
            min_interval: Duration::from_millis(334), // ~3 requests per second
        }
    }
}

impl RateLimiter {
    pub fn can_make_request(&mut self) -> bool {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_request_time) >= self.min_interval {
            self.last_request_time = now;
            true
        } else {
            false
        }
    }
}

pub fn setup_rate_limiter(mut commands: Commands) {
    commands.insert_resource(RateLimiter::default());
}
