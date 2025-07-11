use crate::types::GameCamera;
use bevy::{
    core_pipeline::prepass::DepthPrepass, prelude::*,
    render::experimental::occlusion_culling::OcclusionCulling,
};

#[derive(Resource)]
pub struct OcclusionCullingSettings {
    pub enabled: bool,
    pub debug_mode: bool,
}

impl Default for OcclusionCullingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            debug_mode: false,
        }
    }
}

pub fn setup_occlusion_culling(
    mut commands: Commands,
    camera_query: Query<Entity, With<GameCamera>>,
    settings: Res<OcclusionCullingSettings>,
) {
    if settings.enabled {
        for camera_entity in camera_query.iter() {
            commands
                .entity(camera_entity)
                .insert(DepthPrepass)
                .insert(OcclusionCulling);
        }
        info!("Occlusion culling enabled");
    }
}

pub fn toggle_occlusion_culling(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<OcclusionCullingSettings>,
    camera_query: Query<Entity, With<GameCamera>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyO) {
        settings.enabled = !settings.enabled;

        for camera_entity in camera_query.iter() {
            if settings.enabled {
                commands
                    .entity(camera_entity)
                    .insert(DepthPrepass)
                    .insert(OcclusionCulling);
            } else {
                commands
                    .entity(camera_entity)
                    .remove::<DepthPrepass>()
                    .remove::<OcclusionCulling>();
            }
        }

        info!(
            "Occlusion culling: {}",
            if settings.enabled { "ON" } else { "OFF" }
        );
    }
}
