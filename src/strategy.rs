use crate::types::*;
use crate::utils::*;
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

// Updated ExploreStrategy using the new movement system
impl Strategy for ExploreStrategy {
    fn name(&self) -> &'static str {
        "Explore"
    }

    fn base_priority(&self, ant_type: AntType) -> f32 {
        match ant_type {
            AntType::Scout => 8.0,
            AntType::Worker => 5.0,
            AntType::Soldier => 3.0,
        }
    }

    fn global_priority_modifier(&self, game_state: &GameState) -> f32 {
        let visible_tile_count = game_state.visible_tiles.len();
        match visible_tile_count {
            0..=30 => 7.0,
            31..=80 => 4.0,
            81..=150 => 2.0,
            _ => 0.0,
        }
    }

    fn individual_priority_modifier(&self, ant: &Ant, game_state: &GameState) -> f32 {
        let has_unexplored_neighbors = ant
            .position
            .neighbors()
            .iter()
            .any(|pos| !game_state.visible_tiles.contains_key(pos));

        let frontier_bonus = if has_unexplored_neighbors { 4.0 } else { 0.0 };
        let movement_bonus = if !ant.current_move.is_empty() {
            1.0
        } else {
            0.0
        };

        frontier_bonus + movement_bonus
    }

    fn execute(&self, ant: &Ant, game_state: &GameState) -> Vec<HexCoord> {
        // Use the centralized movement system for exploration
        let path = MovementManager::explore_move(ant, game_state);

        info!(
            "Explore: Ant {} (speed: {}) at {:?} planning {} moves: {:?}",
            &ant.id[0..8],
            ant.ant_type.speed(),
            ant.position,
            path.len(),
            path
        );

        path
    }
}

// Updated GatherStrategy
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
        if ant.food.is_some() {
            return 5.0; // High priority to return home
        }

        let near_food = game_state
            .food_on_map
            .values()
            .any(|food| ant.position.distance_to(&food.position) < 3);

        if near_food { 4.0 } else { 0.0 }
    }

    fn execute(&self, ant: &Ant, game_state: &GameState) -> Vec<HexCoord> {
        if ant.food.is_some() {
            // Return to home if carrying food
            MovementManager::return_to_home(ant, game_state)
        } else {
            // Go to nearest food
            MovementManager::move_to_nearest_food(ant, game_state)
        }
    }
}

// Updated DefendStrategy
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
        let enemies_near_home = game_state.enemy_ants.values().any(|enemy| {
            game_state
                .home_tiles
                .iter()
                .any(|home| enemy.position.distance_to(home) < 3)
        });

        if enemies_near_home { 10.0 } else { 0.0 }
    }

    fn individual_priority_modifier(&self, _ant: &Ant, _game_state: &GameState) -> f32 {
        0.0
    }

    fn execute(&self, ant: &Ant, game_state: &GameState) -> Vec<HexCoord> {
        // Find the most threatened home tile and move to defend it
        if let Some(threatened_home) = game_state.home_tiles.iter().min_by_key(|home| {
            game_state
                .enemy_ants
                .values()
                .map(|enemy| enemy.position.distance_to(home))
                .min()
                .unwrap_or(i32::MAX)
        }) {
            MovementManager::move_to_defend(ant, *threatened_home, game_state)
        } else {
            // No specific threat, stay near main spot
            MovementManager::move_to_defend(ant, game_state.main_spot, game_state)
        }
    }
}

// Updated AttackStrategy
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
        let soldier_count = game_state
            .my_ants
            .values()
            .filter(|ant| ant.ant_type == AntType::Soldier)
            .count();

        if soldier_count > 3 { 5.0 } else { 0.0 }
    }

    fn individual_priority_modifier(&self, ant: &Ant, game_state: &GameState) -> f32 {
        // Higher priority if ant is near an enemy
        let near_enemy = game_state
            .enemy_ants
            .values()
            .any(|enemy| ant.position.distance_to(&enemy.position) < 4);

        if near_enemy { 3.0 } else { 0.0 }
    }

    fn execute(&self, ant: &Ant, game_state: &GameState) -> Vec<HexCoord> {
        // Find nearest enemy and attack
        if let Some(nearest_enemy) = game_state
            .enemy_ants
            .values()
            .min_by_key(|enemy| ant.position.distance_to(&enemy.position))
        {
            MovementManager::move_to_attack(ant, nearest_enemy, game_state)
        } else {
            // No enemies visible, explore to find them
            MovementManager::explore_move(ant, game_state)
        }
    }
}
