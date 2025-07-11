mod config;
mod culling;
mod game;
mod input;
mod menu;
mod plugins;
mod rendering;
mod server;
mod skybox;
mod types;
mod ui;
mod utils;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_tokio_tasks::TokioTasksPlugin;
use chrono::Local;
use config::AppConfig;
use plugins::*;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, Layer, fmt};
use types::*;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    // Load configuration
    let config_path = Path::new("config.toml");
    let app_config = AppConfig::load_or_create(config_path)?;

    // Ensure logs directory exists
    fs::create_dir_all("logs")?;

    // Date and time for log file
    let date = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

    // Setup logging
    let general_log =
        RollingFileAppender::new(Rotation::NEVER, "logs", format!("{}_general.log", date));
    let server_log =
        RollingFileAppender::new(Rotation::NEVER, "logs", format!("{}_server.log", date));

    let filter_general = EnvFilter::new(
        "debug,\
        server=info,\
        wgpu=off,\
        naga=off,\
        cosmic_text=off,\
        bevy_render=info,\
        bevy_ecs=info,\
        bevy_app=info,\
        bevy_winit=info,\
        bevy_asset=info,\
        bevy_scene=info,\
        bevy_ui=info,\
        bevy=info",
    )
    .add_directive("server=off".parse().unwrap());

    let filter_server = EnvFilter::new("server=info");

    let filter_stdout = EnvFilter::new(
        "debug,\
        server=info,\
        wgpu=off,\
        naga=off,\
        cosmic_text=off,\
        bevy_render=info,\
        bevy_ecs=info,\
        bevy_app=info,\
        bevy_winit=info,\
        bevy_asset=info,\
        bevy_scene=info,\
        bevy_ui=info,\
        bevy=info",
    );

    let general_layer = fmt::layer()
        .with_writer(general_log)
        .with_ansi(false)
        .with_filter(filter_general);

    let server_layer = fmt::layer()
        .with_writer(server_log)
        .with_ansi(false)
        .with_filter(filter_server);

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_filter(filter_stdout);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(general_layer)
        .with(server_layer)
        .init();

    // Create server configuration
    let server_config = ServerConfig {
        url: app_config.server.url.clone(),
        token: config::get_api_token()?,
        tick_rate: Duration::from_millis(app_config.server.tick_rate_ms),
        auto_reconnect: app_config.server.auto_reconnect,
    };

    // Build and run the Bevy app
    App::new()
        // Core Bevy plugins
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "DatsPulse - Ant Colony Strategy".to_string(),
                        resolution: (1280.0, 720.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                .disable::<bevy::log::LogPlugin>(), // We use our own logging
        )
        // External plugins
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            TokioTasksPlugin::default(),
            EguiPlugin::default(),
        ))
        // Custom plugins (events are now managed within each plugin)
        .add_plugins((
            ServerPlugin,           // Handles all server communication and API events
            GamePlugin,             // Handles game logic and game events
            InputPlugin,            // Handles input processing
            MenuPlugin,             // Handles UI menus
            UiPlugin,               // Handles HUD and UI elements
            RenderingPlugin,        // Handles 3D rendering
            SkyboxPlugin,           // Handles skybox rendering
            OcclusionCullingPlugin, // Handles occlusion culling
        ))
        // Resources (shared state)
        .insert_resource(app_config)
        .insert_resource(server_config)
        .insert_resource(GameState::default())
        .insert_resource(ConnectionState::default())
        // Run the application
        .run();

    Ok(())
}
