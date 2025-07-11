use crate::game::*;
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_game_logic)
            .add_systems(Update, game_logic_system);
    }
}
