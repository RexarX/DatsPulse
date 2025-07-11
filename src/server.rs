use crate::types::*;
use anyhow::Result;
use bevy::prelude::*;
use bevy_tokio_tasks::{TaskContext, TokioTasksRuntime};
use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{error, info};

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
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    pub async fn test_connection(&self) -> Result<()> {
        let url = format!("{}/health", self.config.url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .send()
            .await?;

        if response.status().is_success() {
            info!(target: "server", "Server health check succeeded");
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!(target: "server", "Server health check failed: {} - {}", status, error_text);
            Err(anyhow::anyhow!(
                "Server returned status: {} - {}",
                status,
                error_text
            ))
        }
    }

    pub async fn get_game_state(&self) -> Result<GameState> {
        self.get_endpoint("game/state").await
    }

    pub async fn send_action(&self, action: &GameAction) -> Result<GameResponse> {
        self.post_endpoint("game/action", action).await
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

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if response.status().is_success() {
            let data: T = response.json().await?;
            info!(target: "server", "GET {} succeeded", endpoint);
            Ok(data)
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

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("Content-Type", "application/json")
            .json(data)
            .send()
            .await?;

        if response.status().is_success() {
            let response_data: R = response.json().await?;
            info!(target: "server", "POST {} succeeded", endpoint);
            Ok(response_data)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!(target: "server", "POST {} failed: {} - {}", endpoint, status, error_text);
            Err(anyhow::anyhow!("Server error: {} - {}", status, error_text))
        }
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
    time: Res<Time>,
    tokio_tasks: Res<TokioTasksRuntime>,
) {
    server_ticker.timer.tick(time.delta());

    if server_ticker.timer.just_finished() {
        let config = server_client.config.clone();
        if connection_state.connected {
            spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
                let server_client = ServerClient::new(config);
                server_client.get_game_state().await
            });
        } else {
            spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
                let server_client = ServerClient::new(config);
                server_client.test_connection().await
            });
        }
    }
}

pub fn handle_server_connection_tasks(
    mut commands: Commands,
    mut connection_state: ResMut<ConnectionState>,
    mut connection_events: EventWriter<ConnectionEvent>,
    mut query: Query<(Entity, &mut ServerTask<Result<()>>)>,
) {
    for (entity, mut task) in &mut query {
        if let Some(handle) = &mut task.handle {
            if handle.is_finished() {
                let handle = task.handle.take().unwrap();
                match futures::executor::block_on(handle) {
                    Ok(Ok(_)) => {
                        if !connection_state.connected {
                            connection_state.connected = true;
                            connection_state.connection_message = "Connected to server".to_string();
                            connection_events.write(ConnectionEvent {
                                connected: true,
                                message: "Successfully connected to server".to_string(),
                            });
                            info!(target: "server", "Connected to server");
                        }
                    }
                    Ok(Err(e)) => {
                        if connection_state.connected {
                            connection_state.connected = false;
                            connection_state.connection_message =
                                format!("Connection failed: {}", e);
                            connection_events.write(ConnectionEvent {
                                connected: false,
                                message: format!("Connection failed: {}", e),
                            });
                            error!(target: "server", "Connection failed: {}", e);
                        }
                    }
                    Err(e) => {
                        error!(target: "server", "Task join error: {e}");
                    }
                }
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn handle_game_state_tasks(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut query: Query<(Entity, &mut ServerTask<Result<GameState>>)>,
) {
    for (entity, mut task) in &mut query {
        if let Some(handle) = &mut task.handle {
            if handle.is_finished() {
                let handle = task.handle.take().unwrap();
                match futures::executor::block_on(handle) {
                    Ok(Ok(new_state)) => {
                        *game_state = new_state;
                        game_state.connected = true;
                        info!(target: "server", "Game state updated");
                    }
                    Ok(Err(e)) => {
                        error!(target: "server", "Failed to fetch game state: {e}");
                        game_state.connected = false;
                    }
                    Err(e) => {
                        error!(target: "server", "Task join error: {e}");
                    }
                }
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn handle_game_actions(
    mut commands: Commands,
    mut action_events: EventReader<GameActionEvent>,
    server_client: Res<ServerClient>,
    connection_state: Res<ConnectionState>,
    tokio_tasks: Res<TokioTasksRuntime>,
) {
    for event in action_events.read() {
        if connection_state.connected {
            let config = server_client.config.clone();
            let action = event.0.clone();

            spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
                let server_client = ServerClient::new(config);
                server_client.send_action(&action).await
            });
        }
    }
}

pub fn handle_action_response_tasks(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ServerTask<Result<GameResponse>>)>,
) {
    for (entity, mut task) in &mut query {
        if let Some(handle) = &mut task.handle {
            if handle.is_finished() {
                let handle = task.handle.take().unwrap();
                match futures::executor::block_on(handle) {
                    Ok(Ok(response)) => {
                        info!(target: "server", "Action response: {:?}", response);
                    }
                    Ok(Err(e)) => {
                        error!(target: "server", "Action failed: {}", e);
                    }
                    Err(e) => {
                        error!(target: "server", "Task join error: {e}");
                    }
                }
                commands.entity(entity).despawn();
            }
        }
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
        connection_state.connection_message = "Reconnecting...".to_string();

        let config = server_client.config.clone();

        spawn_server_task(&mut commands, &tokio_tasks, move |_ctx| async move {
            let server_client = ServerClient::new(config);
            server_client.test_connection().await
        });
        info!(target: "server", "Reconnect requested");
    }
}
