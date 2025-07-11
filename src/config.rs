use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub renderer: RendererConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererConfig {
    pub target_fps: u32,
    pub vsync: bool,
    pub resolution: (u32, u32),
    pub window_mode: String,        // "windowed", "borderless", "fullscreen"
    pub anisotropic_filtering: u32, // 1, 2, 4, 8, 16
    pub anti_aliasing: String,      // "none", "msaa2", "msaa4", "msaa8", "fxaa", "smaa", "taa"
    pub ssao_enabled: bool,
    pub clear_color: (f32, f32, f32),
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                url: "https://games-test.datsteam.dev/api".to_string(),
                token: "your-token-here".to_string(),
                tick_rate_ms: 1000,
                auto_reconnect: true,
                timeout_seconds: 10,
            },
            renderer: RendererConfig {
                target_fps: 60,
                vsync: true,
                resolution: (1280, 720),
                window_mode: "windowed".to_string(),
                anisotropic_filtering: 16,
                anti_aliasing: "msaa4".to_string(),
                ssao_enabled: false,
                clear_color: (0.0, 0.0, 0.0), // Black background
            },
            camera: CameraConfig {
                movement_speed: 5.0,
                sprint_multiplier: 2.0,
                mouse_sensitivity: 0.002,
                fov: 75.0,
            },
            ui: UiConfig {
                show_fps: false,
                show_connection: false,
                show_debug_text: false,
                show_game_state: false,
                enable_docking: true,
                menu_font_size: 16.0,
                ui_font_size: 20.0,
                menu_title: "DatsPulse Settings".to_string(),
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
