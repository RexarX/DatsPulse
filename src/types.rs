use bevy::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Game-specific types
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct GameState {
    pub connected: bool,
    pub game_data: HashMap<String, serde_json::Value>,
    pub last_update: DateTime<Utc>,
    pub player_position: Vec3,
    pub score: i32,
    pub level: i32,
    pub time_remaining: f32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            connected: false,
            game_data: HashMap::new(),
            last_update: Utc::now(),
            player_position: Vec3::ZERO,
            score: 0,
            level: 1,
            time_remaining: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAction {
    pub action_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Resource)]
pub struct ServerConfig {
    pub url: String,
    pub token: String,
    pub tick_rate: std::time::Duration,
    pub auto_reconnect: bool,
}

#[derive(Debug, Clone, Resource)]
pub struct ConnectionState {
    pub connected: bool,
    pub last_connection_attempt: Option<DateTime<Utc>>,
    pub connection_message: String,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            connected: false,
            last_connection_attempt: None,
            connection_message: "Waiting for server connection...".to_string(),
        }
    }
}

// Events
#[derive(Event)]
pub struct GameActionEvent(pub GameAction);

#[derive(Event)]
pub struct ConnectionEvent {
    pub connected: bool,
    pub message: String,
}

#[derive(Event)]
pub struct ReconnectRequestEvent;

// Components
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct DebugMarker;

#[derive(thiserror::Error, Debug)]
pub enum GameError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Server error: {message}")]
    Server { message: String },

    #[error("Connection error: {message}")]
    Connection { message: String },
}

pub type GameResult<T> = Result<T, GameError>;
