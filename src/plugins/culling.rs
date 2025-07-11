use crate::culling::*;
use bevy::prelude::*;

pub struct OcclusionCullingPlugin;

impl Plugin for OcclusionCullingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OcclusionCullingSettings>()
            .add_systems(Startup, setup_occlusion_culling)
            .add_systems(Update, toggle_occlusion_culling);
    }
}
