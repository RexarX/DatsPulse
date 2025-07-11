use crate::menu::MenuState;
use crate::types::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct ConnectionText;

#[derive(Component)]
pub struct DebugText;

pub fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_config: Res<crate::config::AppConfig>,
) {
    // Load custom font
    let font_handle = asset_server.load("fonts/Roboto-Bold.ttf");

    // Root UI container
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|parent| {
            // FPS Text
            parent.spawn((
                Text::new("FPS: 0"),
                TextFont {
                    font: font_handle.clone(),
                    font_size: app_config.ui.ui_font_size,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                FpsText,
            ));

            // Connection Status
            parent.spawn((
                Text::new("Disconnected"),
                TextFont {
                    font: font_handle.clone(),
                    font_size: app_config.ui.ui_font_size * 0.9,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.0, 0.0)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0 + app_config.ui.ui_font_size * 1.5),
                    left: Val::Px(10.0),
                    ..default()
                },
                ConnectionText,
            ));

            // Debug Text
            parent.spawn((
                Text::new("Debug: OFF"),
                TextFont {
                    font: font_handle.clone(),
                    font_size: app_config.ui.ui_font_size * 0.8,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0 + app_config.ui.ui_font_size * 3.0),
                    left: Val::Px(10.0),
                    ..default()
                },
                DebugText,
            ));

            // Game State Info
            parent.spawn((
                Text::new("Game State: Loading..."),
                TextFont {
                    font: font_handle.clone(),
                    font_size: app_config.ui.ui_font_size * 0.8,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0 + app_config.ui.ui_font_size * 4.5),
                    left: Val::Px(10.0),
                    ..default()
                },
                GameStateText,
            ));

            // Controls Text
            parent.spawn((
                Text::new("Controls:\nWASD: Camera | Space/Ctrl: Up/Down\nEscape: Toggle Mouse | F1: Toggle Debug\nInsert: Menu | R: Reconnect"),
                TextFont {
                    font: font_handle.clone(),
                    font_size: app_config.ui.ui_font_size * 0.7,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
            ));
        });
}

#[derive(Component)]
pub struct GameStateText;

pub fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.0 = format!("FPS: {:.1}", average);
            }
        }
    }
}

pub fn update_connection_text(
    connection_state: Res<ConnectionState>,
    mut query: Query<(&mut Text, &mut TextColor), With<ConnectionText>>,
) {
    for (mut text, mut color) in &mut query {
        if connection_state.connected {
            text.0 = "Connected".to_string();
            color.0 = Color::srgb(0.0, 1.0, 0.0);
        } else {
            text.0 = format!("Disconnected: {}", connection_state.connection_message);
            color.0 = Color::srgb(1.0, 0.0, 0.0);
        }
    }
}

pub fn update_debug_text(menu_state: Res<MenuState>, mut query: Query<&mut Text, With<DebugText>>) {
    for mut text in &mut query {
        text.0 = if menu_state.debug_mode {
            "Debug: ON".to_string()
        } else {
            "Debug: OFF".to_string()
        };
    }
}

pub fn update_game_state_text(
    game_state: Res<GameState>,
    mut query: Query<&mut Text, With<GameStateText>>,
) {
    for mut text in &mut query {
        text.0 = format!(
            "Score: {} | Level: {} | Time: {:.1}s\nPosition: ({:.1}, {:.1}, {:.1})",
            game_state.score,
            game_state.level,
            game_state.time_remaining,
            game_state.player_position.x,
            game_state.player_position.y,
            game_state.player_position.z
        );
    }
}
