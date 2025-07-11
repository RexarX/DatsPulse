use crate::types::*;
use bevy::prelude::*;
use std::collections::HashMap;
use std::time::Duration;
use tracing::debug;

#[derive(Resource)]
pub struct GameLogic {
    update_count: u64,
    last_action_time: std::time::Instant,
    action_interval: Duration,
}

impl Default for GameLogic {
    fn default() -> Self {
        Self {
            update_count: 0,
            last_action_time: std::time::Instant::now(),
            action_interval: Duration::from_secs(1),
        }
    }
}

pub fn setup_game_logic(mut commands: Commands) {
    commands.insert_resource(GameLogic::default());
}

pub fn game_logic_system(
    mut game_logic: ResMut<GameLogic>,
    game_state: Res<GameState>,
    mut action_events: EventWriter<GameActionEvent>,
    _time: Res<Time>,
) {
    game_logic.update_count += 1;

    // Only take action if enough time has passed
    if game_logic.last_action_time.elapsed() < game_logic.action_interval {
        return;
    }

    // Run your game algorithm here
    if let Some(action) = calculate_next_action(&game_state) {
        action_events.write(GameActionEvent(action));
        game_logic.last_action_time = std::time::Instant::now();
    }

    // Log progress periodically
    if game_logic.update_count % 100 == 0 {
        debug!("Game update #{}", game_logic.update_count);
    }
}

fn calculate_next_action(game_state: &GameState) -> Option<GameAction> {
    let current_pos = game_state.player_position;
    let distance_from_origin = current_pos.length();

    if distance_from_origin > 5.0 {
        // Move towards origin
        let mut parameters = HashMap::new();
        parameters.insert("direction".to_string(), serde_json::json!("origin"));
        parameters.insert("speed".to_string(), serde_json::json!(1.0));

        Some(GameAction {
            action_type: "move".to_string(),
            parameters,
            timestamp: chrono::Utc::now(),
        })
    } else {
        // Random exploration
        let mut parameters = HashMap::new();
        parameters.insert("direction".to_string(), serde_json::json!("random"));
        parameters.insert("speed".to_string(), serde_json::json!(0.5));

        Some(GameAction {
            action_type: "explore".to_string(),
            parameters,
            timestamp: chrono::Utc::now(),
        })
    }
}
