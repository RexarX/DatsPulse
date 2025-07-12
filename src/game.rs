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
    use std::collections::{HashMap, HashSet};

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

    // Step 1: Collect all planned moves
    let mut planned_moves: HashMap<&String, Vec<HexCoord>> = HashMap::new();
    let mut strategy_names: HashMap<&String, &str> = HashMap::new();

    for (ant_id, ant) in &game_state.my_ants {
        let best_strategy = strategy_manager.select_strategy(ant, &game_state);
        let path = best_strategy.execute(ant, &game_state);
        planned_moves.insert(ant_id, path);
        strategy_names.insert(ant_id, best_strategy.name());
    }

    // Step 2: Reservation table to avoid move conflicts
    let mut reserved: HashSet<HexCoord> = HashSet::new();
    for (ant_id, path) in planned_moves {
        let strategy_name = strategy_names.get(ant_id).unwrap_or(&"Unknown");
        info!(
            "Ant {} (type: {:?}) assigned '{}' strategy, path: {:?}",
            ant_id, game_state.my_ants[ant_id].ant_type, strategy_name, path
        );

        // If the path is not empty, send a move command
        if !path.is_empty() {
            move_events.write(MoveCommandEvent {
                ant_id: ant_id.clone(),
                path: path,
            });
        }
    }

    game_logic.last_action_time = std::time::Instant::now();

    // Log progress periodically
    if game_logic.update_count % 100 == 0 {
        debug!("Game update #{}", game_logic.update_count);
    }
}
