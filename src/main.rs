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

    // Setup logging (unchanged)
    let general_log =
        RollingFileAppender::new(Rotation::NEVER, "logs", format!("{}_general.log", date));
    let server_log =
        RollingFileAppender::new(Rotation::NEVER, "logs", format!("{}_server.log", date));

    let filter_general = EnvFilter::new(
        "info,\
        server=info,\
        wgpu=off,\
        naga=off,\
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
        "info,\
        server=info,\
        wgpu=off,\
        naga=off,\
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

    let server_config = ServerConfig {
        url: app_config.server.url.clone(),
        token: app_config.server.token.clone(),
        tick_rate: Duration::from_millis(app_config.server.tick_rate_ms),
        auto_reconnect: app_config.server.auto_reconnect,
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "DatsPulse".to_string(),
                        resolution: (1280.0, 720.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                .disable::<bevy::log::LogPlugin>(),
        )
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(TokioTasksPlugin::default())
        .add_plugins(EguiPlugin::default())
        // Custom Plugins
        .add_plugins((
            InputPlugin,
            MenuPlugin,
            UiPlugin,
            GamePlugin,
            ServerPlugin,
            RenderingPlugin,
            SkyboxPlugin,
            OcclusionCullingPlugin,
        ))
        // Resources
        .insert_resource(app_config)
        .insert_resource(server_config)
        .insert_resource(GameState::default())
        .insert_resource(ConnectionState::default())
        // Events
        .add_event::<GameActionEvent>()
        .add_event::<ConnectionEvent>()
        .add_event::<ReconnectRequestEvent>()
        .add_event::<RegisterRequestEvent>()
        .add_event::<MoveCommandEvent>()
        .run();

    Ok(())
}
