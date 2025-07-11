use bevy::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Hex coordinate system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn s(&self) -> i32 {
        -self.q - self.r
    }

    pub fn distance_to(&self, other: &HexCoord) -> i32 {
        ((self.q - other.q).abs() + (self.r - other.r).abs() + (self.s() - other.s()).abs()) / 2
    }

    pub fn neighbors(&self) -> Vec<HexCoord> {
        vec![
            HexCoord::new(self.q + 1, self.r),
            HexCoord::new(self.q + 1, self.r - 1),
            HexCoord::new(self.q, self.r - 1),
            HexCoord::new(self.q - 1, self.r),
            HexCoord::new(self.q - 1, self.r + 1),
            HexCoord::new(self.q, self.r + 1),
        ]
    }

    pub fn to_vec3(&self) -> Vec3 {
        // Convert hex coordinates to 3D position for rendering
        let x = (3.0_f32.sqrt() * self.q as f32 + 3.0_f32.sqrt() / 2.0 * self.r as f32) * 0.5;
        let z = (3.0 / 2.0 * self.r as f32) * 0.5;
        Vec3::new(x, 0.0, z)
    }
}

// Ant types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AntType {
    Worker = 0,
    Soldier = 1,
    Scout = 2,
}

impl AntType {
    pub fn from_int(value: i32) -> Option<Self> {
        match value {
            0 => Some(AntType::Worker),
            1 => Some(AntType::Soldier),
            2 => Some(AntType::Scout),
            _ => None,
        }
    }

    pub fn health(&self) -> i32 {
        match self {
            AntType::Worker => 130,
            AntType::Soldier => 180,
            AntType::Scout => 80,
        }
    }

    pub fn attack(&self) -> i32 {
        match self {
            AntType::Worker => 30,
            AntType::Soldier => 70,
            AntType::Scout => 20,
        }
    }

    pub fn speed(&self) -> i32 {
        match self {
            AntType::Worker => 5,
            AntType::Soldier => 4,
            AntType::Scout => 7,
        }
    }

    pub fn view_radius(&self) -> i32 {
        match self {
            AntType::Worker => 1,
            AntType::Soldier => 1,
            AntType::Scout => 4,
        }
    }

    pub fn food_capacity(&self) -> i32 {
        match self {
            AntType::Worker => 8,
            AntType::Soldier => 2,
            AntType::Scout => 2,
        }
    }
}

// Food types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FoodType {
    Apple = 1,
    Bread = 2,
    Nectar = 3,
}

impl FoodType {
    pub fn from_int(value: i32) -> Option<Self> {
        match value {
            1 => Some(FoodType::Apple),
            2 => Some(FoodType::Bread),
            3 => Some(FoodType::Nectar),
            _ => None,
        }
    }

    pub fn calories(&self) -> i32 {
        match self {
            FoodType::Apple => 10,
            FoodType::Bread => 20,
            FoodType::Nectar => 60,
        }
    }
}

// Tile types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    AntSpot = 1,
    Plain = 2,
    Dirt = 3,
    Acid = 4,
    Rock = 5,
}

impl TileType {
    pub fn from_int(value: i32) -> Option<Self> {
        match value {
            1 => Some(TileType::AntSpot),
            2 => Some(TileType::Plain),
            3 => Some(TileType::Dirt),
            4 => Some(TileType::Acid),
            5 => Some(TileType::Rock),
            _ => None,
        }
    }

    pub fn movement_cost(&self) -> Option<i32> {
        match self {
            TileType::AntSpot => Some(1),
            TileType::Plain => Some(1),
            TileType::Dirt => Some(2),
            TileType::Acid => Some(1),
            TileType::Rock => None, // Impassable
        }
    }

    pub fn damage(&self) -> i32 {
        match self {
            TileType::Acid => 20,
            _ => 0,
        }
    }
}

// API Data Transfer Objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiFood {
    pub amount: i32,
    #[serde(rename = "type")]
    pub food_type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAnt {
    pub id: String,
    #[serde(rename = "type")]
    pub ant_type: i32,
    pub q: i32,
    pub r: i32,
    pub health: i32,
    pub food: ApiFood,
    #[serde(rename = "lastMove")]
    pub last_move: Vec<HexCoord>,
    #[serde(rename = "move")]
    pub current_move: Vec<HexCoord>,
    #[serde(rename = "lastAttack")]
    pub last_attack: Option<HexCoord>,
    #[serde(rename = "lastEnemyAnt")]
    pub last_enemy_ant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEnemy {
    #[serde(rename = "type")]
    pub ant_type: i32,
    pub q: i32,
    pub r: i32,
    pub health: i32,
    pub food: ApiFood,
    pub attack: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiFoodOnMap {
    pub q: i32,
    pub r: i32,
    #[serde(rename = "type")]
    pub food_type: i32,
    pub amount: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTile {
    pub q: i32,
    pub r: i32,
    #[serde(rename = "type")]
    pub tile_type: i32,
    pub cost: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiArenaResponse {
    pub ants: Vec<ApiAnt>,
    pub enemies: Vec<ApiEnemy>,
    pub food: Vec<ApiFoodOnMap>,
    pub home: Vec<HexCoord>,
    pub map: Vec<ApiTile>,
    #[serde(rename = "nextTurnIn")]
    pub next_turn_in: f64,
    pub score: i32,
    pub spot: HexCoord,
    #[serde(rename = "turnNo")]
    pub turn_no: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMoveCommand {
    pub ant: String,
    pub path: Vec<HexCoord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMoveRequest {
    pub moves: Vec<ApiMoveCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMoveResponse {
    pub ants: Vec<ApiAnt>,
    pub enemies: Vec<ApiEnemy>,
    pub food: Vec<ApiFoodOnMap>,
    pub home: Vec<HexCoord>,
    pub map: Vec<ApiTile>,
    #[serde(rename = "nextTurnIn")]
    pub next_turn_in: f64,
    pub score: i32,
    pub spot: HexCoord,
    #[serde(rename = "turnNo")]
    pub turn_no: i32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRegistrationResponse {
    #[serde(rename = "lobbyEndsIn")]
    pub lobby_ends_in: i32,
    pub name: String,
    #[serde(rename = "nextTurn")]
    pub next_turn: f64,
    pub realm: String,
}

// Game state for internal use
#[derive(Debug, Clone, Resource)]
pub struct GameState {
    pub connected: bool,
    pub my_ants: HashMap<String, Ant>,
    pub enemy_ants: HashMap<String, EnemyAnt>,
    pub food_on_map: HashMap<HexCoord, FoodOnMap>,
    pub visible_tiles: HashMap<HexCoord, Tile>,
    pub home_tiles: Vec<HexCoord>,
    pub main_spot: HexCoord,
    pub score: i32,
    pub turn_number: i32,
    pub next_turn_in: f64,
    pub last_update: DateTime<Utc>,
    pub pending_moves: HashMap<String, Vec<HexCoord>>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            connected: false,
            my_ants: HashMap::new(),
            enemy_ants: HashMap::new(),
            food_on_map: HashMap::new(),
            visible_tiles: HashMap::new(),
            home_tiles: Vec::new(),
            main_spot: HexCoord::new(0, 0),
            score: 0,
            turn_number: 0,
            next_turn_in: 0.0,
            last_update: Utc::now(),
            pending_moves: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ant {
    pub id: String,
    pub ant_type: AntType,
    pub position: HexCoord,
    pub health: i32,
    pub food: Option<(FoodType, i32)>,
    pub last_move: Vec<HexCoord>,
    pub current_move: Vec<HexCoord>,
    pub last_attack: Option<HexCoord>,
    pub last_enemy_ant: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EnemyAnt {
    pub ant_type: AntType,
    pub position: HexCoord,
    pub health: i32,
    pub food: Option<(FoodType, i32)>,
    pub attack: i32,
}

#[derive(Debug, Clone)]
pub struct FoodOnMap {
    pub position: HexCoord,
    pub food_type: FoodType,
    pub amount: i32,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub position: HexCoord,
    pub tile_type: TileType,
    pub movement_cost: Option<i32>,
}

#[derive(Debug, Clone, Resource)]
pub struct ServerConfig {
    pub url: String,
    pub token: String,
    pub tick_rate: std::time::Duration,
    pub auto_reconnect: bool,
}

#[derive(Debug, Clone, Resource)]
pub struct ConnectionState {
    pub connected: bool,
    pub registered: bool,
    pub last_connection_attempt: Option<DateTime<Utc>>,
    pub connection_message: String,
    pub current_realm: Option<String>,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            connected: false,
            registered: false,
            last_connection_attempt: None,
            connection_message: "Waiting for server connection...".to_string(),
            current_realm: None,
        }
    }
}

// Events
#[derive(Event)]
pub struct GameActionEvent(pub GameAction);

#[derive(Event)]
pub struct ConnectionEvent {
    pub connected: bool,
    pub message: String,
}

#[derive(Event)]
pub struct ReconnectRequestEvent;

#[derive(Event)]
pub struct RegisterRequestEvent;

#[derive(Event)]
pub struct MoveCommandEvent {
    pub ant_id: String,
    pub path: Vec<HexCoord>,
}

// Game actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAction {
    pub action_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

// Components for rendering
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct DebugMarker;

#[derive(Component)]
pub struct AntMarker {
    pub ant_id: String,
    pub ant_type: AntType,
    pub is_enemy: bool,
}

#[derive(Component)]
pub struct FoodMarker {
    pub food_type: FoodType,
    pub amount: i32,
}

#[derive(Component)]
pub struct TileMarker {
    pub tile_type: TileType,
    pub position: HexCoord,
}

#[derive(Component)]
pub struct HomeMarker {
    pub is_main_spot: bool,
}

// Pathfinding utilities
pub struct PathFinder;

impl PathFinder {
    pub fn find_path(
        start: HexCoord,
        goal: HexCoord,
        tiles: &HashMap<HexCoord, Tile>,
        max_cost: i32,
    ) -> Option<Vec<HexCoord>> {
        use std::cmp::Ordering;
        use std::collections::{BinaryHeap, HashMap};

        #[derive(Debug, Clone)]
        struct Node {
            position: HexCoord,
            cost: i32,
            heuristic: i32,
        }

        impl PartialEq for Node {
            fn eq(&self, other: &Self) -> bool {
                self.cost + self.heuristic == other.cost + other.heuristic
            }
        }

        impl Eq for Node {}

        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for Node {
            fn cmp(&self, other: &Self) -> Ordering {
                (other.cost + other.heuristic).cmp(&(self.cost + self.heuristic))
            }
        }

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<HexCoord, HexCoord> = HashMap::new();
        let mut g_score: HashMap<HexCoord, i32> = HashMap::new();

        g_score.insert(start, 0);
        open_set.push(Node {
            position: start,
            cost: 0,
            heuristic: start.distance_to(&goal),
        });

        while let Some(current) = open_set.pop() {
            if current.position == goal {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current_pos = goal;

                while current_pos != start {
                    path.push(current_pos);
                    current_pos = came_from[&current_pos];
                }
                path.push(start);
                path.reverse();
                return Some(path);
            }

            for neighbor in current.position.neighbors() {
                if let Some(tile) = tiles.get(&neighbor) {
                    if let Some(move_cost) = tile.movement_cost {
                        let tentative_g_score = g_score[&current.position] + move_cost;

                        if tentative_g_score > max_cost {
                            continue;
                        }
                        if tentative_g_score >= *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                            continue;
                        }
                        came_from.insert(neighbor, current.position);
                        g_score.insert(neighbor, tentative_g_score);
                        open_set.push(Node {
                            position: neighbor,
                            cost: tentative_g_score,
                            heuristic: neighbor.distance_to(&goal),
                        });
                    }
                }
            }
        }

        None
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GameError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Server error: {message}")]
    Server { message: String },

    #[error("Connection error: {message}")]
    Connection { message: String },

    #[error("Invalid coordinate: {0}")]
    InvalidCoordinate(String),

    #[error("Invalid ant type: {0}")]
    InvalidAntType(i32),

    #[error("Invalid food type: {0}")]
    InvalidFoodType(i32),

    #[error("Invalid tile type: {0}")]
    InvalidTileType(i32),
}

pub type GameResult<T> = Result<T, GameError>;
