use crate::types::*;
use std::collections::HashMap;

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
