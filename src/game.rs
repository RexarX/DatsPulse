use crate::types::*;
use crate::utils::*;
use bevy::prelude::*;
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
}

pub fn game_logic_system(
    mut game_logic: ResMut<GameLogic>,
    game_state: Res<GameState>,
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

    // Analyze current situation and decide strategy
    let strategy = analyze_situation(&game_state);
    if strategy != game_logic.current_strategy {
        game_logic.current_strategy = strategy.clone();
        debug!("Strategy changed to: {:?}", strategy);
    }

    // Execute strategy for each ant
    for (ant_id, ant) in &game_state.my_ants {
        if let Some(path) = calculate_ant_move(ant, &game_state, &strategy) {
            move_events.write(MoveCommandEvent {
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

fn analyze_situation(game_state: &GameState) -> GameStrategy {
    let ant_count = game_state.my_ants.len();
    let enemy_count = game_state.enemy_ants.len();
    let food_count = game_state.food_on_map.len();
    let home_distance = calculate_average_distance_from_home(game_state);

    // Simple strategy selection based on game state
    if enemy_count > 0 && home_distance < 3.0 {
        GameStrategy::Defend
    } else if enemy_count > ant_count && home_distance > 5.0 {
        GameStrategy::Gather
    } else if food_count > 0 {
        GameStrategy::Gather
    } else {
        GameStrategy::Explore
    }
}

fn calculate_average_distance_from_home(game_state: &GameState) -> f32 {
    if game_state.my_ants.is_empty() {
        return 0.0;
    }

    let total_distance: i32 = game_state
        .my_ants
        .values()
        .map(|ant| ant.position.distance_to(&game_state.main_spot))
        .sum();

    total_distance as f32 / game_state.my_ants.len() as f32
}

fn calculate_ant_move(
    ant: &Ant,
    game_state: &GameState,
    strategy: &GameStrategy,
) -> Option<Vec<HexCoord>> {
    match strategy {
        GameStrategy::Explore => calculate_explore_move(ant, game_state),
        GameStrategy::Gather => calculate_gather_move(ant, game_state),
        GameStrategy::Defend => calculate_defend_move(ant, game_state),
        GameStrategy::Attack => calculate_attack_move(ant, game_state),
    }
}

fn calculate_explore_move(ant: &Ant, game_state: &GameState) -> Option<Vec<HexCoord>> {
    // Find unexplored areas or move randomly
    let current_pos = ant.position;
    let max_distance = ant.ant_type.speed();

    // Look for edges of known map to explore
    let mut best_target = None;
    let mut best_score = 0;

    for (pos, _tile) in &game_state.visible_tiles {
        let distance = current_pos.distance_to(pos);
        if distance <= max_distance {
            // Score based on distance from home and current position
            let home_distance = pos.distance_to(&game_state.main_spot);
            let score = home_distance - distance;
            if score > best_score {
                best_score = score;
                best_target = Some(*pos);
            }
        }
    }

    if let Some(target) = best_target {
        PathFinder::find_path(current_pos, target, &game_state.visible_tiles, max_distance)
    } else {
        // Random exploration
        let neighbors = current_pos.neighbors();
        if let Some(neighbor) = neighbors.into_iter().find(|pos| {
            game_state
                .visible_tiles
                .get(pos)
                .map(|tile| tile.tile_type.is_passable())
                .unwrap_or(false)
        }) {
            Some(vec![current_pos, neighbor])
        } else {
            None
        }
    }
}

fn calculate_gather_move(ant: &Ant, game_state: &GameState) -> Option<Vec<HexCoord>> {
    let current_pos = ant.position;
    let max_distance = ant.ant_type.speed();

    // If carrying food, return to home
    if ant.food.is_some() {
        return find_closest_home_path(current_pos, game_state, max_distance);
    }

    // Find closest food
    let mut closest_food = None;
    let mut closest_distance = i32::MAX;

    for food in game_state.food_on_map.values() {
        let distance = current_pos.distance_to(&food.position);
        if distance < closest_distance {
            closest_distance = distance;
            closest_food = Some(food.position);
        }
    }

    if let Some(food_pos) = closest_food {
        PathFinder::find_path(
            current_pos,
            food_pos,
            &game_state.visible_tiles,
            max_distance,
        )
    } else {
        // No food visible, explore
        calculate_explore_move(ant, game_state)
    }
}

fn calculate_defend_move(ant: &Ant, game_state: &GameState) -> Option<Vec<HexCoord>> {
    let current_pos = ant.position;
    let max_distance = ant.ant_type.speed();

    // Find closest enemy
    let mut closest_enemy = None;
    let mut closest_distance = i32::MAX;

    for enemy in game_state.enemy_ants.values() {
        let distance = current_pos.distance_to(&enemy.position);
        if distance < closest_distance {
            closest_distance = distance;
            closest_enemy = Some(enemy.position);
        }
    }

    if let Some(enemy_pos) = closest_enemy {
        // Move towards enemy (for attack) or defensive position
        let target = if closest_distance > 2 {
            enemy_pos // Chase if far
        } else {
            game_state.main_spot // Retreat to home if close
        };

        PathFinder::find_path(current_pos, target, &game_state.visible_tiles, max_distance)
    } else {
        // No enemies visible, patrol around home
        let home_neighbors = game_state.main_spot.neighbors();
        if let Some(patrol_pos) = home_neighbors.into_iter().find(|pos| {
            game_state
                .visible_tiles
                .get(pos)
                .map(|tile| tile.cost != 0)
                .unwrap_or(false)
        }) {
            PathFinder::find_path(
                current_pos,
                patrol_pos,
                &game_state.visible_tiles,
                max_distance,
            )
        } else {
            None
        }
    }
}

fn calculate_attack_move(ant: &Ant, game_state: &GameState) -> Option<Vec<HexCoord>> {
    let current_pos = ant.position;
    let max_distance = ant.ant_type.speed();

    // Find weakest enemy or closest enemy
    let mut target_enemy = None;
    let mut best_score = i32::MIN;

    for enemy in game_state.enemy_ants.values() {
        let distance = current_pos.distance_to(&enemy.position);
        // Score: prioritize closer enemies and weaker enemies
        let score = -distance * 10 - enemy.health;
        if score > best_score {
            best_score = score;
            target_enemy = Some(enemy.position);
        }
    }

    if let Some(enemy_pos) = target_enemy {
        PathFinder::find_path(
            current_pos,
            enemy_pos,
            &game_state.visible_tiles,
            max_distance,
        )
    } else {
        // No enemies, switch to gathering
        calculate_gather_move(ant, game_state)
    }
}

fn find_closest_home_path(
    current_pos: HexCoord,
    game_state: &GameState,
    max_distance: i32,
) -> Option<Vec<HexCoord>> {
    let mut closest_home = None;
    let mut closest_distance = i32::MAX;

    for home_pos in &game_state.home_tiles {
        let distance = current_pos.distance_to(home_pos);
        if distance < closest_distance {
            closest_distance = distance;
            closest_home = Some(*home_pos);
        }
    }

    if let Some(home_pos) = closest_home {
        PathFinder::find_path(
            current_pos,
            home_pos,
            &game_state.visible_tiles,
            max_distance,
        )
    } else {
        None
    }
}

// Helper function to check if position is safe
fn _is_position_safe(pos: &HexCoord, game_state: &GameState) -> bool {
    // Check if any enemies are nearby
    for enemy in game_state.enemy_ants.values() {
        if enemy.position.distance_to(pos) <= 2 {
            return false;
        }
    }

    // Check if tile is dangerous
    if let Some(tile) = game_state.visible_tiles.get(pos) {
        if tile.tile_type.damage() > 0 {
            return false;
        }
    }

    true
}
