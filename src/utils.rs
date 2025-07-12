use crate::types::*;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

// Enhanced movement system that respects speed limits and provides common movement patterns
pub struct MovementManager;

impl MovementManager {
    /// Find a path to target, respecting the ant's speed limit
    pub fn find_path_to_target(
        ant: &Ant,
        target: HexCoord,
        game_state: &GameState,
    ) -> Vec<HexCoord> {
        let max_moves = ant.ant_type.speed() as usize;

        if let Some(path) = Self::pathfind(ant.position, target, &game_state.visible_tiles) {
            // Return only the moves the ant can make this turn (excluding current position)
            path.into_iter()
                .skip(1) // Skip current position
                .take(max_moves)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all valid adjacent moves for an ant
    pub fn get_valid_moves(ant: &Ant, game_state: &GameState) -> Vec<HexCoord> {
        ant.position
            .neighbors()
            .into_iter()
            .filter(|pos| Self::is_valid_move(pos, game_state))
            .collect()
    }

    /// Move towards a target, respecting speed limits
    pub fn move_towards(ant: &Ant, target: HexCoord, game_state: &GameState) -> Vec<HexCoord> {
        Self::find_path_to_target(ant, target, game_state)
    }

    /// Find a good exploration move (prioritizes unexplored areas)
    pub fn explore_move(ant: &Ant, game_state: &GameState) -> Vec<HexCoord> {
        let max_moves = ant.ant_type.speed() as usize;
        let valid_moves = Self::get_valid_moves(ant, game_state);

        if valid_moves.is_empty() {
            return Vec::new();
        }

        // Score moves based on exploration value
        let mut scored_moves: Vec<(HexCoord, f32)> = valid_moves
            .iter()
            .map(|pos| (*pos, Self::exploration_score(*pos, game_state)))
            .collect();

        // Sort by score (highest first)
        scored_moves.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Try to plan a multi-step exploration path
        if let Some((best_move, _)) = scored_moves.first() {
            Self::plan_exploration_path(ant.position, *best_move, max_moves, game_state)
        } else {
            Vec::new()
        }
    }

    /// Find the nearest food and return a path to it
    pub fn move_to_nearest_food(ant: &Ant, game_state: &GameState) -> Vec<HexCoord> {
        let nearest_food = game_state
            .food_on_map
            .values()
            .min_by_key(|food| ant.position.distance_to(&food.position));

        if let Some(food) = nearest_food {
            Self::find_path_to_target(ant, food.position, game_state)
        } else {
            Vec::new()
        }
    }

    /// Return to the nearest home tile
    pub fn return_to_home(ant: &Ant, game_state: &GameState) -> Vec<HexCoord> {
        let nearest_home = game_state
            .home_tiles
            .iter()
            .min_by_key(|home| ant.position.distance_to(home));

        if let Some(home) = nearest_home {
            Self::find_path_to_target(ant, *home, game_state)
        } else {
            Vec::new()
        }
    }

    /// Move to intercept or attack an enemy
    pub fn move_to_attack(
        ant: &Ant,
        target_enemy: &Enemy,
        game_state: &GameState,
    ) -> Vec<HexCoord> {
        Self::find_path_to_target(ant, target_enemy.position, game_state)
    }

    /// Move to defend a specific position
    pub fn move_to_defend(
        ant: &Ant,
        defend_position: HexCoord,
        game_state: &GameState,
    ) -> Vec<HexCoord> {
        Self::find_path_to_target(ant, defend_position, game_state)
    }

    // Private helper methods
    fn is_valid_move(pos: &HexCoord, game_state: &GameState) -> bool {
        // Check if position is passable
        match game_state.visible_tiles.get(pos) {
            Some(tile) => tile.tile_type.is_passable(),
            None => true, // Assume unexplored tiles are passable
        }
    }

    fn exploration_score(pos: HexCoord, game_state: &GameState) -> f32 {
        let mut score = 0.0;

        // Count unexplored neighbors
        let unexplored_neighbors = pos
            .neighbors()
            .iter()
            .filter(|neighbor| !game_state.visible_tiles.contains_key(neighbor))
            .count();

        score += unexplored_neighbors as f32 * 2.0;

        // Prefer positions that are not too close to other ants
        let nearby_ants = game_state
            .my_ants
            .values()
            .filter(|ant| ant.position.distance_to(&pos) < 3)
            .count();

        score -= nearby_ants as f32 * 0.5;

        score
    }

    fn plan_exploration_path(
        start: HexCoord,
        first_move: HexCoord,
        max_moves: usize,
        game_state: &GameState,
    ) -> Vec<HexCoord> {
        let mut path = vec![first_move];
        let mut current = first_move;

        for _ in 1..max_moves {
            let valid_moves = current
                .neighbors()
                .into_iter()
                .filter(|pos| Self::is_valid_move(pos, game_state))
                .collect::<Vec<_>>();

            if valid_moves.is_empty() {
                break;
            }

            // Pick the move with the highest exploration score
            let next_move = valid_moves
                .iter()
                .max_by(|a, b| {
                    Self::exploration_score(**a, game_state)
                        .partial_cmp(&Self::exploration_score(**b, game_state))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .cloned();

            if let Some(next) = next_move {
                path.push(next);
                current = next;
            } else {
                break;
            }
        }

        path
    }

    fn pathfind(
        start: HexCoord,
        target: HexCoord,
        tiles: &HashMap<HexCoord, Tile>,
    ) -> Option<Vec<HexCoord>> {
        // Simple BFS pathfinding
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut came_from = HashMap::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(current) = queue.pop_front() {
            if current == target {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current = target;

                while current != start {
                    path.push(current);
                    if let Some(parent) = came_from.get(&current) {
                        current = *parent;
                    } else {
                        break;
                    }
                }

                path.push(start);
                path.reverse();
                return Some(path);
            }

            for neighbor in current.neighbors() {
                if !visited.contains(&neighbor) && Self::is_tile_passable(&neighbor, tiles) {
                    visited.insert(neighbor);
                    came_from.insert(neighbor, current);
                    queue.push_back(neighbor);
                }
            }
        }

        None
    }

    fn is_tile_passable(pos: &HexCoord, tiles: &HashMap<HexCoord, Tile>) -> bool {
        match tiles.get(pos) {
            Some(tile) => tile.tile_type.is_passable(),
            None => true, // Assume unexplored tiles are passable
        }
    }
}
pub struct PathFinder;

impl PathFinder {
    pub fn find_path(
        start: HexCoord,
        target: HexCoord,
        tiles: &HashMap<HexCoord, Tile>,
        max_distance: i32,
    ) -> Option<Vec<HexCoord>> {
        // Simple pathfinding - just direct line for now
        let distance = start.distance(&target);
        if distance > max_distance {
            return None;
        }

        // Check if target is reachable
        if let Some(tile) = tiles.get(&target) {
            if !tile.tile_type.is_passable() {
                return None;
            }
        }

        // For now, return a simple path (just the target)
        // A proper A* implementation would go here
        Some(vec![start, target])
    }
}
