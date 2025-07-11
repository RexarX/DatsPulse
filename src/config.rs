use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub camera: CameraConfig,
    pub ui: UiConfig,
    pub debug: DebugConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub url: String,
    pub token: String,
    pub tick_rate_ms: u64,
    pub auto_reconnect: bool,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    pub movement_speed: f32,
    pub sprint_multiplier: f32,
    pub mouse_sensitivity: f32,
    pub fov: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub show_fps: bool,
    pub show_connection: bool,
    pub show_debug_text: bool,
    pub show_game_state: bool,
    pub enable_docking: bool,
    pub menu_font_size: f32,
    pub ui_font_size: f32,
    pub menu_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub debug_mode: bool,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                url: "https://games-test.datsteam.dev".to_string(),
                token: std::env::var("API_TOKEN").unwrap_or_else(|_| "your-token-here".to_string()),
                tick_rate_ms: 1000,
                auto_reconnect: true,
                timeout_seconds: 10,
            },
            camera: CameraConfig {
                movement_speed: 5.0,
                sprint_multiplier: 2.0,
                mouse_sensitivity: 0.003,
                fov: 75.0,
            },
            ui: UiConfig {
                show_fps: true,
                show_connection: true,
                show_debug_text: true,
                show_game_state: true,
                enable_docking: true,
                menu_font_size: 16.0,
                ui_font_size: 20.0,
                menu_title: "Debug Menu".to_string(),
            },
            debug: DebugConfig {
                debug_mode: false,
                log_level: "info".to_string(),
            },
        }
    }
}

impl AppConfig {
    pub fn load_or_create(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = AppConfig::default();
            config.save(path)?;
            Ok(config)
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }
}

pub fn get_api_token() -> anyhow::Result<String> {
    match std::env::var("API_TOKEN") {
        Ok(token) if !token.is_empty() && token != "your-token-here" => Ok(token),
        _ => Err(anyhow::anyhow!(
            "API_TOKEN environment variable not set or invalid. \
            Please create a .env file in your project root with:\n\nAPI_TOKEN=your-actual-token-here\n"
        )),
    }
}
