use crate::types::*;
use crate::utils::PathFinder;
use bevy::prelude::*;
use std::collections::HashMap;

// Strategy trait that all strategies must implement
pub trait Strategy {
    fn name(&self) -> &'static str;
    fn base_priority(&self, ant_type: AntType) -> f32;

    // Calculate global priority modifiers based on game state
    fn global_priority_modifier(&self, game_state: &GameState) -> f32;

    // Calculate individual priority modifiers based on ant's state
    fn individual_priority_modifier(&self, ant: &Ant, game_state: &GameState) -> f32;

    // Execute the strategy for a specific ant
    fn execute(&self, ant: &Ant, game_state: &GameState) -> Vec<HexCoord>;
}

// Strategy manager to handle all strategies
#[derive(Resource)]
pub struct StrategyManager {
    strategies: Vec<Box<dyn Strategy + Send + Sync>>,
    ant_strategies: HashMap<String, String>, // Maps ant_id to current strategy name
}

impl Default for StrategyManager {
    fn default() -> Self {
        let mut strategies: Vec<Box<dyn Strategy + Send + Sync>> = Vec::new();

        // Add all strategies
        strategies.push(Box::new(ExploreStrategy));
        strategies.push(Box::new(GatherStrategy));
        strategies.push(Box::new(DefendStrategy));
        strategies.push(Box::new(AttackStrategy));

        Self {
            strategies,
            ant_strategies: HashMap::new(),
        }
    }
}

impl StrategyManager {
    // Calculate priorities for an ant and return the best strategy
    pub fn select_strategy(&self, ant: &Ant, game_state: &GameState) -> &dyn Strategy {
        let mut best_strategy = &self.strategies[0];
        let mut highest_priority = f32::MIN;

        for strategy in &self.strategies {
            // Calculate total priority
            let base = strategy.base_priority(ant.ant_type);
            let global = strategy.global_priority_modifier(game_state);
            let individual = strategy.individual_priority_modifier(ant, game_state);

            let total_priority = base + global + individual;

            if total_priority > highest_priority {
                highest_priority = total_priority;
                best_strategy = strategy;
            }
        }

        best_strategy.as_ref()
    }

    // Track which strategy each ant is using
    pub fn set_ant_strategy(&mut self, ant_id: &str, strategy_name: &str) {
        self.ant_strategies
            .insert(ant_id.to_string(), strategy_name.to_string());
    }

    pub fn get_ant_strategy(&self, ant_id: &str) -> Option<&String> {
        self.ant_strategies.get(ant_id)
    }
}

// Strategy types
pub struct ExploreStrategy;
pub struct GatherStrategy;
pub struct DefendStrategy;
pub struct AttackStrategy;

// Implementation for explore strategy
impl Strategy for ExploreStrategy {
    fn name(&self) -> &'static str {
        "Explore"
    }

    fn base_priority(&self, ant_type: AntType) -> f32 {
        match ant_type {
            AntType::Scout => 8000.0,
            AntType::Worker => 5000.0,
            AntType::Soldier => 3000.0,
        }
    }

    fn global_priority_modifier(&self, game_state: &GameState) -> f32 {
        // Count visible tiles as a metric for exploration coverage
        let visible_tile_count = game_state.visible_tiles.len();

        // Use match to determine priority boost based on exploration level
        match visible_tile_count {
            0..=30 => 7.0,   // High priority for early exploration
            31..=80 => 4.0,  // Medium priority
            81..=150 => 2.0, // Lower priority
            _ => 0.0,        // Low priority once we've explored a lot
        }
    }

    fn individual_priority_modifier(&self, ant: &Ant, game_state: &GameState) -> f32 {
        // Simplified priority modifier that just checks if there are unexplored neighbors
        let has_unexplored_neighbors = ant
            .position
            .neighbors()
            .iter()
            .any(|pos| !game_state.visible_tiles.contains_key(pos));

        let frontier_bonus = if has_unexplored_neighbors { 4.0 } else { 0.0 };

        // Check if the ant is already moving (continuity bonus)
        let movement_bonus = if !ant.current_move.is_empty() {
            1.0
        } else {
            0.0
        };

        frontier_bonus + movement_bonus
    }

    fn execute(&self, ant: &Ant, game_state: &GameState) -> Vec<HexCoord> {
        // Get all passable neighboring tiles
        let passable_neighbors: Vec<HexCoord> = ant
            .position
            .neighbors()
            .into_iter()
            .filter(|pos| {
                match game_state.visible_tiles.get(pos) {
                    Some(tile) => tile.tile_type.is_passable(),
                    None => true, // Assume unexplored tiles are passable
                }
            })
            .collect();

        // Log the available directions
        info!(
            "Simple Explore: Ant {} at {:?} has {} passable neighbors",
            &ant.id[0..8],
            ant.position,
            passable_neighbors.len()
        );

        if passable_neighbors.is_empty() {
            info!(
                "Simple Explore: Ant {} has no passable neighbors, staying put",
                &ant.id[0..8]
            );
            return Vec::new(); // No valid moves
        }

        // Try to avoid going back to the previous position
        let mut valid_moves: Vec<HexCoord>;
        if !ant.last_move.is_empty() {
            let previous_pos = ant.last_move[0];
            valid_moves = passable_neighbors
                .iter()
                .filter(|pos| **pos != previous_pos)
                .cloned() // Use cloned() to avoid ownership issues
                .collect();

            // If filtering out the previous position leaves us with no options, use all passable neighbors
            if valid_moves.is_empty() {
                valid_moves = passable_neighbors.clone(); // Clone to avoid ownership issues
            }
        } else {
            valid_moves = passable_neighbors.clone(); // Clone to avoid ownership issues
        }

        // Pick a direction based on the ant's ID for deterministic but seemingly random behavior
        let index = (ant.id.chars().next().unwrap_or('a') as usize) % valid_moves.len();
        let chosen_move = valid_moves[index];

        info!(
            "Simple Explore: Ant {} moving from {:?} to {:?}",
            &ant.id[0..8],
            ant.position,
            chosen_move
        );

        vec![chosen_move]
    }
}

// Implementation for Gather strategy
impl Strategy for GatherStrategy {
    fn name(&self) -> &'static str {
        "Gather"
    }

    fn base_priority(&self, ant_type: AntType) -> f32 {
        match ant_type {
            AntType::Worker => 9.0,
            AntType::Scout => 4.0,
            AntType::Soldier => 2.0,
        }
    }

    fn global_priority_modifier(&self, game_state: &GameState) -> f32 {
        // Higher priority if we have food on the map
        let food_count = game_state.food_on_map.len();

        if food_count > 5 {
            3.0
        } else if food_count > 0 {
            1.0
        } else {
            0.0
        }
    }

    fn individual_priority_modifier(&self, ant: &Ant, game_state: &GameState) -> f32 {
        // If ant is already carrying food, prioritize returning to base
        if ant.food.is_some() {
            return 5.0;
        }

        // If ant is near food, prioritize gathering it
        let near_food = game_state
            .food_on_map
            .values()
            .any(|food| ant.position.distance_to(&food.position) < 3);

        if near_food { 4.0 } else { 0.0 }
    }

    fn execute(&self, _ant: &Ant, _game_state: &GameState) -> Vec<HexCoord> {
        // For now, just return an empty path
        // A real implementation would either go to food or return to base
        Vec::new()
    }
}

// Implementation for Defend strategy
impl Strategy for DefendStrategy {
    fn name(&self) -> &'static str {
        "Defend"
    }

    fn base_priority(&self, ant_type: AntType) -> f32 {
        match ant_type {
            AntType::Soldier => 8.0,
            AntType::Worker => 3.0,
            AntType::Scout => 2.0,
        }
    }

    fn global_priority_modifier(&self, game_state: &GameState) -> f32 {
        // Example: If enemies are near home, increase defense priority
        let enemies_near_home = game_state.enemy_ants.values().any(|enemy| {
            game_state
                .home_tiles
                .iter()
                .any(|home| enemy.position.distance_to(home) < 3)
        });

        if enemies_near_home { 10.0 } else { 0.0 }
    }

    fn individual_priority_modifier(&self, ant: &Ant, game_state: &GameState) -> f32 {
        // Example: Higher priority for ants closer to home
        0.0
    }

    fn execute(&self, _ant: &Ant, _game_state: &GameState) -> Vec<HexCoord> {
        // Implementation for defending
        Vec::new()
    }
}

// Implementation for Attack strategy
impl Strategy for AttackStrategy {
    fn name(&self) -> &'static str {
        "Attack"
    }

    fn base_priority(&self, ant_type: AntType) -> f32 {
        match ant_type {
            AntType::Soldier => 7.0,
            AntType::Scout => 3.0,
            AntType::Worker => 1.0,
        }
    }

    fn global_priority_modifier(&self, game_state: &GameState) -> f32 {
        // Example: If we have many soldiers, increase attack priority
        let soldier_count = game_state
            .my_ants
            .values()
            .filter(|ant| ant.ant_type == AntType::Soldier)
            .count();

        if soldier_count > 3 { 5.0 } else { 0.0 }
    }

    fn individual_priority_modifier(&self, ant: &Ant, game_state: &GameState) -> f32 {
        // Example: Higher priority if ant is near an enemy
        0.0
    }

    fn execute(&self, _ant: &Ant, _game_state: &GameState) -> Vec<HexCoord> {
        // Implementation for attacking
        Vec::new()
    }
}
