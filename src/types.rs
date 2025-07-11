use bevy::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Game-specific types
#[derive(Debug, Clone, Resource)]
pub struct GameState {
    pub connected: bool,
    pub game_data: HashMap<String, serde_json::Value>,
    pub last_update: DateTime<Utc>,
    pub player_position: Vec3,
    pub score: i32,
    pub level: i32,
    pub time_remaining: f32,
    pub my_ants: HashMap<String, Ant>,
    pub enemy_ants: HashMap<String, Enemy>,
    pub food_on_map: HashMap<HexCoord, FoodOnMap>,
    pub visible_tiles: HashMap<HexCoord, Tile>,
    pub home_tiles: Vec<HexCoord>,
    pub main_spot: HexCoord,
    pub turn_number: i32,
    pub next_turn_in: f32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            connected: false,
            game_data: HashMap::new(),
            last_update: Utc::now(),
            player_position: Vec3::ZERO,
            score: 0,
            level: 1,
            time_remaining: 0.0,
            my_ants: HashMap::new(),
            enemy_ants: HashMap::new(),
            food_on_map: HashMap::new(),
            visible_tiles: HashMap::new(),
            home_tiles: Vec::new(),
            main_spot: HexCoord::new(0, 0),
            turn_number: 0,
            next_turn_in: 0.0,
        }
    }
}

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
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            connected: false,
            registered: false,
            last_connection_attempt: None,
            connection_message: "Waiting for server connection...".to_string(),
        }
    }
}

// API Request/Response types for DatsPulse (with Api prefix)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiArenaResponse {
    pub ants: Vec<ApiAnt>,
    pub enemies: Vec<ApiEnemy>,
    pub food: Vec<ApiFoodOnMap>,
    pub home: Vec<ApiHex>,
    pub map: Vec<ApiTile>,
    #[serde(rename = "nextTurnIn")]
    pub next_turn_in: f64,
    pub score: i32,
    pub spot: ApiHex,
    #[serde(rename = "turnNo")]
    pub turn_no: i32,
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
    pub last_move: Vec<ApiHex>,
    #[serde(rename = "move")]
    pub current_move: Vec<ApiHex>,
    #[serde(rename = "lastAttack")]
    pub last_attack: Option<ApiHex>,
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
pub struct ApiFood {
    pub amount: i32,
    #[serde(rename = "type")]
    pub food_type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiFoodOnMap {
    pub q: i32,
    pub r: i32,
    pub amount: i32,
    #[serde(rename = "type")]
    pub food_type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiHex {
    pub q: i32,
    pub r: i32,
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
pub struct ApiMoveCommand {
    pub ant: String,
    pub path: Vec<ApiHex>,
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
    pub home: Vec<ApiHex>,
    pub map: Vec<ApiTile>,
    pub errors: Vec<String>,
    #[serde(rename = "nextTurnIn")]
    pub next_turn_in: f64,
    pub score: i32,
    pub spot: ApiHex,
    #[serde(rename = "turnNo")]
    pub turn_no: i32,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLogMessage {
    pub message: String,
    pub time: String,
}

// Game logic types
#[derive(Debug, Clone)]
pub struct Ant {
    pub id: String,
    pub ant_type: AntType,
    pub position: HexCoord,
    pub health: i32,
    pub max_health: i32,
    pub food: Food,
    pub last_move: Vec<HexCoord>,
    pub current_move: Vec<HexCoord>,
    pub last_attack: Option<HexCoord>,
    pub last_enemy_ant: Option<String>,
}

impl Ant {
    pub fn food(&self) -> Option<(FoodType, i32)> {
        if self.food.amount > 0 {
            Some((self.food.food_type, self.food.amount))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub ant_type: AntType,
    pub position: HexCoord,
    pub health: i32,
    pub food: Food,
    pub attack: i32,
}

#[derive(Debug, Clone)]
pub struct Food {
    pub amount: i32,
    pub food_type: FoodType,
}

impl Food {
    pub fn is_some(&self) -> bool {
        self.amount > 0
    }
}

#[derive(Debug, Clone)]
pub struct FoodOnMap {
    pub position: HexCoord,
    pub amount: i32,
    pub food_type: FoodType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn distance(&self, other: &HexCoord) -> i32 {
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

    pub fn distance_to(&self, other: &HexCoord) -> i32 {
        self.distance(other)
    }

    pub fn to_vec3(&self) -> Vec3 {
        hex_to_world_pos(self)
    }
}

impl From<ApiHex> for HexCoord {
    fn from(hex: ApiHex) -> Self {
        Self::new(hex.q, hex.r)
    }
}

impl From<HexCoord> for ApiHex {
    fn from(coord: HexCoord) -> Self {
        Self {
            q: coord.q,
            r: coord.r,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub position: HexCoord,
    pub tile_type: TileType,
    pub cost: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AntType {
    Worker = 0,
    Soldier = 1,
    Scout = 2,
}

impl AntType {
    pub fn from_api(value: i32) -> Option<Self> {
        match value {
            0 => Some(AntType::Worker),
            1 => Some(AntType::Soldier),
            2 => Some(AntType::Scout),
            _ => None,
        }
    }

    pub fn to_api(&self) -> i32 {
        *self as i32
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

    pub fn capacity(&self) -> i32 {
        match self {
            AntType::Worker => 8,
            AntType::Soldier => 2,
            AntType::Scout => 2,
        }
    }

    pub fn view_range(&self) -> i32 {
        match self {
            AntType::Worker => 1,
            AntType::Soldier => 1,
            AntType::Scout => 4,
        }
    }

    pub fn speed(&self) -> i32 {
        match self {
            AntType::Worker => 5,
            AntType::Soldier => 4,
            AntType::Scout => 7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FoodType {
    Apple = 1,
    Bread = 2,
    Nectar = 3,
}

impl FoodType {
    pub fn from_api(value: i32) -> Option<Self> {
        match value {
            1 => Some(FoodType::Apple),
            2 => Some(FoodType::Bread),
            3 => Some(FoodType::Nectar),
            _ => None,
        }
    }

    pub fn to_api(&self) -> i32 {
        *self as i32
    }

    pub fn calories(&self) -> i32 {
        match self {
            FoodType::Apple => 10,
            FoodType::Bread => 20,
            FoodType::Nectar => 60,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileType {
    Anthill = 1,
    Plain = 2,
    Dirt = 3,
    Acid = 4,
    Rock = 5,
}

impl TileType {
    pub fn from_api(value: i32) -> Option<Self> {
        match value {
            1 => Some(TileType::Anthill),
            2 => Some(TileType::Plain),
            3 => Some(TileType::Dirt),
            4 => Some(TileType::Acid),
            5 => Some(TileType::Rock),
            _ => None,
        }
    }

    pub fn to_api(&self) -> i32 {
        *self as i32
    }

    pub fn is_passable(&self) -> bool {
        match self {
            TileType::Rock => false,
            _ => true,
        }
    }

    pub fn movement_cost(&self) -> Option<i32> {
        match self {
            TileType::Anthill => Some(1),
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

// Conversion implementations
impl From<ApiAnt> for Ant {
    fn from(api_ant: ApiAnt) -> Self {
        let ant_type = AntType::from_api(api_ant.ant_type).unwrap_or(AntType::Worker);
        Self {
            id: api_ant.id,
            ant_type,
            position: HexCoord::from(ApiHex {
                q: api_ant.q,
                r: api_ant.r,
            }),
            health: api_ant.health,
            max_health: ant_type.health(),
            food: Food::from(api_ant.food),
            last_move: api_ant.last_move.into_iter().map(HexCoord::from).collect(),
            current_move: api_ant
                .current_move
                .into_iter()
                .map(HexCoord::from)
                .collect(),
            last_attack: api_ant.last_attack.map(HexCoord::from),
            last_enemy_ant: api_ant.last_enemy_ant,
        }
    }
}

impl From<ApiEnemy> for Enemy {
    fn from(api_enemy: ApiEnemy) -> Self {
        Self {
            ant_type: AntType::from_api(api_enemy.ant_type).unwrap_or(AntType::Worker),
            position: HexCoord::from(ApiHex {
                q: api_enemy.q,
                r: api_enemy.r,
            }),
            health: api_enemy.health,
            food: Food::from(api_enemy.food),
            attack: api_enemy.attack,
        }
    }
}

impl From<ApiFood> for Food {
    fn from(api_food: ApiFood) -> Self {
        Self {
            amount: api_food.amount,
            food_type: FoodType::from_api(api_food.food_type).unwrap_or(FoodType::Apple),
        }
    }
}

impl From<ApiFoodOnMap> for FoodOnMap {
    fn from(api_food: ApiFoodOnMap) -> Self {
        Self {
            position: HexCoord::from(ApiHex {
                q: api_food.q,
                r: api_food.r,
            }),
            amount: api_food.amount,
            food_type: FoodType::from_api(api_food.food_type).unwrap_or(FoodType::Apple),
        }
    }
}

impl From<ApiTile> for Tile {
    fn from(api_tile: ApiTile) -> Self {
        Self {
            position: HexCoord::from(ApiHex {
                q: api_tile.q,
                r: api_tile.r,
            }),
            tile_type: TileType::from_api(api_tile.tile_type).unwrap_or(TileType::Plain),
            cost: api_tile.cost,
        }
    }
}

// Game state conversion
impl GameState {
    pub fn from_api_response(response: &ApiArenaResponse) -> Self {
        let mut game_state = GameState {
            connected: true,
            score: response.score,
            level: response.turn_no,
            time_remaining: response.next_turn_in as f32,
            last_update: chrono::Utc::now(),
            turn_number: response.turn_no,
            next_turn_in: response.next_turn_in as f32,
            main_spot: HexCoord::from(response.spot.clone()),
            ..Default::default()
        };

        // Convert API data to internal format
        for api_ant in &response.ants {
            let ant = Ant::from(api_ant.clone());
            game_state.my_ants.insert(ant.id.clone(), ant);
        }

        for api_enemy in &response.enemies {
            let enemy = Enemy::from(api_enemy.clone());
            // Generate a unique ID for enemy since API doesn't provide one
            let enemy_id = format!("enemy_{}_{}", enemy.position.q, enemy.position.r);
            game_state.enemy_ants.insert(enemy_id, enemy);
        }

        for api_food in &response.food {
            let food = FoodOnMap::from(api_food.clone());
            game_state.food_on_map.insert(food.position, food);
        }

        for api_tile in &response.map {
            let tile = Tile::from(api_tile.clone());
            game_state.visible_tiles.insert(tile.position, tile);
        }

        for api_home in &response.home {
            game_state.home_tiles.push(HexCoord::from(api_home.clone()));
        }

        // Set player position to the spot (main hex of anthill)
        game_state.player_position = game_state.main_spot.to_vec3();

        // Add additional game data
        game_state.game_data.insert(
            "ants_count".to_string(),
            serde_json::json!(response.ants.len()),
        );
        game_state.game_data.insert(
            "enemies_count".to_string(),
            serde_json::json!(response.enemies.len()),
        );
        game_state.game_data.insert(
            "food_count".to_string(),
            serde_json::json!(response.food.len()),
        );

        game_state
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
pub struct RegisterRequestEvent;

#[derive(Event)]
pub struct ReconnectRequestEvent;

#[derive(Event)]
pub struct MoveCommandEvent {
    pub ant_id: String,
    pub path: Vec<HexCoord>,
}

// API Events
#[derive(Event)]
pub struct ApiArenaEvent(pub ApiArenaResponse);

#[derive(Event)]
pub struct ApiMoveEvent(pub ApiMoveRequest);

#[derive(Event)]
pub struct ApiRegistrationEvent(pub ApiRegistrationResponse);

// Components
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct DebugMarker;

#[derive(Component)]
pub struct AntEntity {
    pub ant_id: String,
    pub ant_type: AntType,
}

#[derive(Component)]
pub struct EnemyEntity {
    pub ant_type: AntType,
}

#[derive(Component)]
pub struct FoodEntity {
    pub food_type: FoodType,
    pub amount: i32,
}

#[derive(Component)]
pub struct TileEntity {
    pub tile_type: TileType,
    pub position: HexCoord,
}

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

// Error types
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

    #[error("API error: {message}")]
    Api { message: String },

    #[error("Invalid ant type: {value}")]
    InvalidAntType { value: i32 },

    #[error("Invalid food type: {value}")]
    InvalidFoodType { value: i32 },

    #[error("Invalid tile type: {value}")]
    InvalidTileType { value: i32 },

    #[error("Pathfinding error: {message}")]
    Pathfinding { message: String },
}

pub type GameResult<T> = Result<T, GameError>;

// Utility functions
pub fn hex_to_world_pos(hex: &HexCoord) -> Vec3 {
    // Convert hex coordinates to world position for 3D rendering
    let size = 1.0;
    let x = size * (3.0_f32.sqrt() * hex.q as f32 + 3.0_f32.sqrt() / 2.0 * hex.r as f32);
    let z = size * (3.0 / 2.0 * hex.r as f32);
    Vec3::new(x, 0.0, z)
}

pub fn world_pos_to_hex(pos: &Vec3) -> HexCoord {
    // Convert world position back to hex coordinates
    let size = 1.0;
    let q = (3.0_f32.sqrt() / 3.0 * pos.x - 1.0 / 3.0 * pos.z) / size;
    let r = (2.0 / 3.0 * pos.z) / size;

    // Round to nearest hex
    let q_round = q.round() as i32;
    let r_round = r.round() as i32;

    HexCoord::new(q_round, r_round)
}

// Constants
pub const MAX_ANTS: i32 = 100;
pub const ANTHILL_ATTACK_RADIUS: i32 = 2;
pub const ANTHILL_DAMAGE: i32 = 20;
pub const SUPPORT_BONUS: f32 = 0.5; // 50% bonus
pub const ANTHILL_BONUS: f32 = 0.25; // 25% bonus
