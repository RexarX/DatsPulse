use crate::input::*;
use crate::types::*;
use bevy::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RegisterRequestEvent>()
            .add_systems(Startup, setup_input)
            .add_systems(
                Update,
                (
                    camera_mouse_toggle_system,
                    camera_movement_system,
                    input_system,
                    sync_camera_settings,
                ),
            );
    }
}
