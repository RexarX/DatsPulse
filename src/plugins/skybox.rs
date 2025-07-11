use crate::skybox::*;
use bevy::prelude::*;

pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkyboxManager>()
            .add_systems(Startup, setup_skybox)
            .add_systems(Update, (update_skybox, toggle_skybox_type));
    }
}
