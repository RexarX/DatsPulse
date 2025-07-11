mod config;
mod culling;
mod game;
mod input;
mod menu;
mod plugins;
mod renderer; // Add this line
mod rendering;
mod server;
mod skybox;
mod strategy;
mod types;
mod ui;
mod utils;

use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin,
    diagnostic::FrameTimeDiagnosticsPlugin, prelude::*,
};
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
    if app_config.server.token.is_empty() || app_config.server.token == "your-token-here" {
        eprintln!("ERROR: Please set your API token in config.toml under [server] token = \"...\"");
        std::process::exit(1);
    }

    let clear_color = ClearColor(Color::srgb(
        app_config.renderer.clear_color.0,
        app_config.renderer.clear_color.1,
        app_config.renderer.clear_color.2,
    ));

    let server_config = ServerConfig {
        url: app_config.server.url.clone(),
        token: app_config.server.token.clone(),
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
                        resolution: (
                            app_config.renderer.resolution.0 as f32,
                            app_config.renderer.resolution.1 as f32,
                        )
                            .into(),
                        present_mode: if app_config.renderer.vsync {
                            bevy::window::PresentMode::AutoVsync
                        } else {
                            bevy::window::PresentMode::AutoNoVsync
                        },
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
        // Custom plugins
        .add_plugins((
            ServerPlugin,
            GamePlugin,
            InputPlugin,
            TemporalAntiAliasPlugin,
            MenuPlugin,
            UiPlugin,
            RenderingPlugin,
            SkyboxPlugin,
            OcclusionCullingPlugin,
            RendererPlugin,
        ))
        // Resources
        .insert_resource(clear_color)
        .insert_resource(app_config)
        .insert_resource(server_config)
        .insert_resource(GameState::default())
        .insert_resource(ConnectionState::default())
        .run();

    Ok(())
}
