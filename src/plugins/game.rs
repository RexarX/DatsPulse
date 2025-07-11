use crate::game::*;
use crate::types::*;
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Add game-specific events
            .add_event::<GameActionEvent>()
            .add_event::<MoveCommandEvent>()
            // Add game systems
            .add_systems(Startup, setup_game_logic)
            .add_systems(Update, game_logic_system);
    }
}
