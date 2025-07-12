use crate::menu::*;
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Menu plugin doesn't need specific events currently
            .add_systems(Startup, setup_menu)
            .add_systems(
                Update,
                (
                    menu_toggle_system,
                    update_ui_visibility,
                    framerate_limiter_system,
                    sync_fov_from_camera,
                ),
            )
            .add_systems(EguiPrimaryContextPass, menu_ui_system);
    }
}
