use crate::strategy::StrategyManager;
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
    current_strategy: GameStrategy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameStrategy {
    Explore,
    Gather,
    Defend,
    Attack,
}

impl Default for GameLogic {
    fn default() -> Self {
        Self {
            update_count: 0,
            last_action_time: std::time::Instant::now(),
            action_interval: Duration::from_millis(1500), // Slightly faster than server tick
            current_strategy: GameStrategy::Explore,
        }
    }
}

pub fn setup_game_logic(mut commands: Commands) {
    commands.insert_resource(GameLogic::default());
    commands.insert_resource(StrategyManager::default());
}

pub fn game_logic_system(
    mut game_logic: ResMut<GameLogic>,
    game_state: Res<GameState>,
    mut strategy_manager: ResMut<StrategyManager>,
    mut move_events: EventWriter<MoveCommandEvent>,
    _time: Res<Time>,
) {
    game_logic.update_count += 1;

    // Only take action if enough time has passed
    if game_logic.last_action_time.elapsed() < game_logic.action_interval {
        return;
    }

    // Skip if not connected or no game data
    if !game_state.connected || game_state.my_ants.is_empty() {
        return;
    }

    info!("Turn #{}: Strategy assignments:", game_state.turn_number);

    // Process all ants and decide strategies
    for (ant_id, ant) in &game_state.my_ants {
        // First, get the strategy name and path without keeping a reference
        let (strategy_name, path) = {
            // Get the best strategy for this ant
            let best_strategy = strategy_manager.select_strategy(ant, &game_state);

            // Get the strategy name and the execution path
            (
                best_strategy.name(),
                best_strategy.execute(ant, &game_state),
            )
        }; // The immutable borrow of strategy_manager ends here

        // Now we can mutably borrow strategy_manager
        strategy_manager.set_ant_strategy(ant_id, &strategy_name);

        info!(
            "  Ant {} (Type: {:?}, Pos: {:?}): {} strategy",
            ant_id, ant.ant_type, ant.position, strategy_name
        );

        // If the path is not empty, send a move command
        if !path.is_empty() {
            move_events.send(MoveCommandEvent {
                ant_id: ant_id.clone(),
                path,
            });
        }
    }

    game_logic.last_action_time = std::time::Instant::now();

    // Log progress periodically
    if game_logic.update_count % 100 == 0 {
        debug!("Game update #{}", game_logic.update_count);
    }
}
