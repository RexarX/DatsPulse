use crate::rendering::*;
use bevy::prelude::*;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_3d_scene)
            .add_systems(Update, (update_player_visual, debug_rendering_system));
    }
}
