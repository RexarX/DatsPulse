use crate::renderer::*;
use bevy::prelude::*;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_renderer)
            .add_systems(
                Update,
                (
                    update_renderer_settings,
                    apply_anti_aliasing,
                    apply_ssao,
                    apply_framerate_limit,
                    apply_window_settings,
                    apply_clear_color,
                    apply_wireframe_settings,
                )
                    .chain()
                    .before(reset_renderer_settings_changed),
            )
            .add_systems(PostUpdate, reset_renderer_settings_changed);
    }
}
