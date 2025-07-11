use crate::input::*;
use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_input).add_systems(
            Update,
            (
                camera_mouse_toggle_system,
                camera_mouse_look_system,
                camera_movement_system,
                input_system,
            ),
        );
    }
}
