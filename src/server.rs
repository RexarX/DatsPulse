use crate::types::*;
use anyhow::Result;
use bevy::prelude::*;
use bevy_tokio_tasks::{TaskContext, TokioTasksRuntime};
use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

#[derive(Resource)]
pub struct ServerClient {
    client: Client,
    config: ServerConfig,
}

#[derive(Resource)]
pub struct ServerTicker {
    pub timer: Timer,
}

impl ServerClient {
    pub fn new(config: ServerConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    pub async fn register(&self) -> Result<ApiRegistrationResponse> {
        let url = format!("{}/api/register", self.config.url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if response.status().is_success() {
            let data: ApiRegistrationResponse = response.json().await?;
            info!(target: "server", "Registration successful: realm={}, name={}", data.realm, data.name);
            Ok(data)
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
        let url = format!("{}/api/arena", self.config.url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if response.status().is_success() {
            let data: ApiArenaResponse = response.json().await?;
            info!(target: "server", "Arena state retrieved: turn={}, ants={}, enemies={}",
                data.turn_no, data.ants.len(), data.enemies.len());
            Ok(data)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!(target: "server", "Failed to get arena state: {} - {}", status, error_text);
            Err(anyhow::anyhow!(
                "Arena state error: {} - {}",
                status,
                error_text
            ))
        }
    }

    pub async fn send_moves(&self, moves: Vec<ApiMoveCommand>) -> Result<ApiMoveResponse> {
        let url = format!("{}/api/move", self.config.url);
        let request = ApiMoveRequest { moves };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let data: ApiMoveResponse = response.json().await?;
            info!(target: "server", "Moves sent successfully: {} commands", request.moves.len());
            if !data.errors.is_empty() {
                warn!(target: "server", "Move errors: {:?}", data.errors);
            }
            Ok(data)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!(target: "server", "Failed to send moves: {} - {}", status, error_text);
            Err(anyhow::anyhow!("Move error: {} - {}", status, error_text))
        }
    }

    pub async fn get_logs(&self) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/api/logs", self.config.url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if response.status().is_success() {
            let data: Vec<serde_json::Value> = response.json().await?;
            info!(target: "server", "Retrieved {} log entries", data.len());
            Ok(data)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!(target: "server", "Failed to get logs: {} - {}", status, error_text);
            Err(anyhow::anyhow!("Log error: {} - {}", status, error_text))
        }
    }
}

// Convert API data to internal game state
impl GameState {
    pub fn update_from_api(&mut self, api_data: &ApiArenaResponse) -> Result<()> {
        self.connected = true;
        self.score = api_data.score;
        self.turn_number = api_data.turn_no;
        self.next_turn_in = api_data.next_turn_in;
        self.main_spot = api_data.spot;
        self.home_tiles = api_data.home.clone();
        self.last_update = chrono::Utc::now();

        // Update ants
        self.my_ants.clear();
        for api_ant in &api_data.ants {
            let ant_type = AntType::from_int(api_ant.ant_type)
                .ok_or_else(|| GameError::InvalidAntType(api_ant.ant_type))?;

            let food = if api_ant.food.amount > 0 {
                FoodType::from_int(api_ant.food.food_type).map(|ft| (ft, api_ant.food.amount))
            } else {
                None
            };

            let ant = Ant {
                id: api_ant.id.clone(),
                ant_type,
                position: HexCoord::new(api_ant.q, api_ant.r),
                health: api_ant.health,
                food,
                last_move: api_ant.last_move.clone(),
                current_move: api_ant.current_move.clone(),
                last_attack: api_ant.last_attack,
                last_enemy_ant: api_ant.last_enemy_ant.clone(),
            };

            self.my_ants.insert(api_ant.id.clone(), ant);
        }

        // Update enemies
        self.enemy_ants.clear();
        for (idx, api_enemy) in api_data.enemies.iter().enumerate() {
            let ant_type = AntType::from_int(api_enemy.ant_type)
                .ok_or_else(|| GameError::InvalidAntType(api_enemy.ant_type))?;

            let food = if api_enemy.food.amount > 0 {
                FoodType::from_int(api_enemy.food.food_type).map(|ft| (ft, api_enemy.food.amount))
            } else {
                None
            };

            let enemy = EnemyAnt {
                ant_type,
                position: HexCoord::new(api_enemy.q, api_enemy.r),
                health: api_enemy.health,
                food,
                attack: api_enemy.attack,
            };

            self.enemy_ants.insert(format!("enemy_{}", idx), enemy);
        }

        // Update food
        self.food_on_map.clear();
        for api_food in &api_data.food {
            let food_type = FoodType::from_int(api_food.food_type)
                .ok_or_else(|| GameError::InvalidFoodType(api_food.food_type))?;

            let food = FoodOnMap {
                position: HexCoord::new(api_food.q, api_food.r),
                food_type,
                amount: api_food.amount,
            };

            self.food_on_map.insert(food.position, food);
        }

        // Update tiles
        self.visible_tiles.clear();
        for api_tile in &api_data.map {
            let tile_type = TileType::from_int(api_tile.tile_type)
                .ok_or_else(|| GameError::InvalidTileType(api_tile.tile_type))?;

            let tile = Tile {
                position: HexCoord::new(api_tile.q, api_tile.r),
                tile_type,
                movement_cost: tile_type.movement_cost(),
            };

            self.visible_tiles.insert(tile.position, tile);
        }

        Ok(())
    }

    pub fn get_moves_to_send(&self) -> Vec<ApiMoveCommand> {
        self.pending_moves
            .iter()
            .map(|(ant_id, path)| ApiMoveCommand {
                ant: ant_id.clone(),
                path: path.clone(),
            })
            .collect()
    }

    pub fn clear_pending_moves(&mut self) {
        self.pending_moves.clear();
    }

    pub fn add_move(&mut self, ant_id: String, path: Vec<HexCoord>) {
        self.pending_moves.insert(ant_id, path);
    }
}

#[derive(Component)]
pub struct ServerTask<T> {
    pub handle: Option<JoinHandle<T>>,
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
    commands.spawn(ServerTask {
        handle: Some(handle),
    });
}

pub fn setup_server_client(mut commands: Commands, config: Res<ServerConfig>) {
    let client = ServerClient::new(config.clone());
    commands.insert_resource(client);

    let timer = Timer::new(config.tick_rate, TimerMode::Repeating);
    commands.insert_resource(ServerTicker { timer });
}

pub fn server_tick_system(
    mut commands: Commands,
    mut server_ticker: ResMut<ServerTicker>,
    server_client: Res<ServerClient>,
    connection_state: Res<ConnectionState>,
    game_state: Res<GameState>,
    time: Res<Time>,
    tokio_tasks: Res<TokioTasksRuntime>,
) {
    server_ticker.timer.tick(time.delta());

    if server_ticker.timer.just_finished() {
        return;
    }
    let config = server_client.config.clone();

    if connection_state.registered {
        // Send pending moves if any, then get arena state
        let moves = game_state.get_moves_to_send();
        if !moves.is_empty() {
            spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
                let server_client = ServerClient::new(config);
                server_client.send_moves(moves).await
            });
        } else {
            spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
                let server_client = ServerClient::new(config);
                server_client.get_arena_state().await
            });
        }
    } else if connection_state.connected {
        // Register for the game
        spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
            let server_client = ServerClient::new(config);
            server_client.register().await
        });
    }
}

pub fn handle_registration_tasks(
    mut commands: Commands,
    mut connection_state: ResMut<ConnectionState>,
    mut connection_events: EventWriter<ConnectionEvent>,
    mut query: Query<(Entity, &mut ServerTask<Result<ApiRegistrationResponse>>)>,
) {
    for (entity, mut task) in &mut query {
        let handle = match &mut task.handle {
            Some(handle) => handle,
            None => continue,
        };

        if !handle.is_finished() {
            continue;
        }

        let handle = task.handle.take().unwrap();
        let result = futures::executor::block_on(handle);

        match result {
            Ok(Ok(registration_data)) => {
                connection_state.registered = true;
                connection_state.current_realm = Some(registration_data.realm.clone());
                connection_state.connection_message = format!(
                    "Registered for realm: {}, lobby ends in: {}s",
                    registration_data.realm, registration_data.lobby_ends_in
                );
                connection_events.write(ConnectionEvent {
                    connected: true,
                    message: "Successfully registered for game".to_string(),
                });
                info!(target: "server", "Successfully registered for game");
            }
            Ok(Err(e)) => {
                connection_state.registered = false;
                connection_state.connection_message = format!("Registration failed: {}", e);
                connection_events.write(ConnectionEvent {
                    connected: false,
                    message: format!("Registration failed: {}", e),
                });
                error!(target: "server", "Registration failed: {}", e);
            }
            Err(e) => {
                error!(target: "server", "Registration task join error: {e}");
            }
        }

        commands.entity(entity).despawn();
    }
}

pub fn handle_arena_state_tasks(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut query: Query<(Entity, &mut ServerTask<Result<ApiArenaResponse>>)>,
) {
    for (entity, mut task) in &mut query {
        let handle = match &mut task.handle {
            Some(handle) => handle,
            None => continue,
        };

        if !handle.is_finished() {
            continue;
        }

        let handle = task.handle.take().unwrap();
        let result = futures::executor::block_on(handle);

        match result {
            Ok(Ok(arena_data)) => {
                if let Err(e) = game_state.update_from_api(&arena_data) {
                    error!(target: "server", "Failed to update game state: {}", e);
                } else {
                    info!(target: "server", "Game state updated successfully");
                }
            }
            Ok(Err(e)) => {
                error!(target: "server", "Failed to fetch arena state: {e}");
                game_state.connected = false;
            }
            Err(e) => {
                error!(target: "server", "Arena state task join error: {e}");
            }
        }

        commands.entity(entity).despawn();
    }
}

pub fn handle_move_response_tasks(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut query: Query<(Entity, &mut ServerTask<Result<ApiMoveResponse>>)>,
) {
    for (entity, mut task) in &mut query {
        let handle = match &mut task.handle {
            Some(handle) => handle,
            None => continue,
        };

        if !handle.is_finished() {
            continue;
        }

        let handle = task.handle.take().unwrap();
        let result = futures::executor::block_on(handle);

        match result {
            Ok(Ok(move_response)) => {
                // Clear pending moves since they were sent
                game_state.clear_pending_moves();

                // Update game state from response
                let arena_data = ApiArenaResponse {
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

                if let Err(e) = game_state.update_from_api(&arena_data) {
                    error!(target: "server", "Failed to update game state from move response: {}", e);
                } else {
                    info!(target: "server", "Game state updated from move response");
                }
            }
            Ok(Err(e)) => {
                error!(target: "server", "Move command failed: {}", e);
                game_state.clear_pending_moves();
            }
            Err(e) => {
                error!(target: "server", "Move response task join error: {e}");
            }
        }

        commands.entity(entity).despawn();
    }
}

pub fn handle_move_commands(
    mut move_events: EventReader<MoveCommandEvent>,
    mut game_state: ResMut<GameState>,
) {
    for event in move_events.read() {
        game_state.add_move(event.ant_id.clone(), event.path.clone());
        info!(target: "server", "Queued move for ant {}: {} steps", event.ant_id, event.path.len());
    }
}

pub fn handle_register_requests(
    mut commands: Commands,
    mut register_events: EventReader<RegisterRequestEvent>,
    mut connection_state: ResMut<ConnectionState>,
    server_client: Res<ServerClient>,
    tokio_tasks: Res<TokioTasksRuntime>,
) {
    for _event in register_events.read() {
        connection_state.registered = false;
        connection_state.connection_message = "Registering for game...".to_string();

        let config = server_client.config.clone();
        spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
            let server_client = ServerClient::new(config);
            server_client.register().await
        });
        info!(target: "server", "Registration requested");
    }
}

pub fn handle_reconnect_requests(
    mut commands: Commands,
    mut reconnect_events: EventReader<ReconnectRequestEvent>,
    mut connection_state: ResMut<ConnectionState>,
    server_client: Res<ServerClient>,
    tokio_tasks: Res<TokioTasksRuntime>,
) {
    for _event in reconnect_events.read() {
        connection_state.connected = false;
        connection_state.registered = false;
        connection_state.connection_message = "Reconnecting...".to_string();

        let config = server_client.config.clone();
        spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
            let server_client = ServerClient::new(config);
            server_client.register().await
        });
        info!(target: "server", "Reconnect requested");
    }
}
