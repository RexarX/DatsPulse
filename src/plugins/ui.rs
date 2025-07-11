use crate::ui::*;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui).add_systems(
            Update,
            (
                update_fps_text,
                update_connection_text,
                update_debug_text,
                update_game_state_text,
            ),
        );
    }
}
