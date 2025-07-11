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

#[derive(Component)]
pub struct GameStateText;

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
            // FPS Text (hidden by default)
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
                Visibility::Hidden, // Hidden by default
            ));

            // Connection Status (hidden by default)
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
                Visibility::Hidden, // Hidden by default
            ));

            // Debug Text (hidden by default)
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
                Visibility::Hidden, // Hidden by default
            ));

            // Game State Info (hidden by default)
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
                Visibility::Hidden, // Hidden by default
            ));

            // Simplified controls text (always visible)
            parent.spawn((
                Text::new("Press Insert to open menu"),
                TextFont {
                    font: font_handle.clone(),
                    font_size: app_config.ui.ui_font_size * 0.8,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
            ));
        });
}

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
            if connection_state.registered {
                text.0 = format!("Connected & Registered");
                color.0 = Color::srgb(0.0, 1.0, 0.0);
            } else {
                text.0 = "Connected - Registering...".to_string();
                color.0 = Color::srgb(1.0, 1.0, 0.0);
            }
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
        if game_state.connected {
            let ant_count = game_state.my_ants.len();
            let enemy_count = game_state.enemy_ants.len();
            let food_count = game_state.food_on_map.len();
            let visible_tiles = game_state.visible_tiles.len();

            // Calculate ant type distribution
            let mut worker_count = 0;
            let mut soldier_count = 0;
            let mut scout_count = 0;

            for ant in game_state.my_ants.values() {
                match ant.ant_type {
                    AntType::Worker => worker_count += 1,
                    AntType::Soldier => soldier_count += 1,
                    AntType::Scout => scout_count += 1,
                }
            }

            // Calculate total food being carried
            let mut carrying_food = 0;
            for ant in game_state.my_ants.values() {
                carrying_food += ant.food.amount;
            }

            text.0 = format!(
                "Turn: {} | Score: {} | Next turn: {:.1}s
Ants: {} (W:{} S:{} Sc:{}) | Enemies: {} | Food: {}
Carrying: {} | Visible tiles: {}
Home: ({}, {})",
                game_state.turn_number,
                game_state.score,
                game_state.next_turn_in,
                ant_count,
                worker_count,
                soldier_count,
                scout_count,
                enemy_count,
                food_count,
                carrying_food,
                visible_tiles,
                game_state.main_spot.q,
                game_state.main_spot.r
            );
        } else {
            text.0 = "Game State: Disconnected".to_string();
        }
    }
}
