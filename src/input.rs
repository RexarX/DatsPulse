use crate::menu::MenuState;
use crate::types::*;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

#[derive(Resource)]
pub struct CameraController {
    pub movement_speed: f32,
    pub sprint_multiplier: f32,
    pub mouse_sensitivity: f32,
    pub pitch: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            movement_speed: 5.0,
            sprint_multiplier: 2.0,
            mouse_sensitivity: 0.003,
            pitch: 0.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct CameraMouseControl {
    pub enabled: bool,
}

pub fn setup_input(mut commands: Commands) {
    commands.insert_resource(CameraController::default());
    commands.insert_resource(CameraMouseControl::default());
}

pub fn camera_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
    controller: Res<CameraController>,
    time: Res<Time>,
    menu_state: Res<MenuState>,
    mouse_control: Res<CameraMouseControl>,
) {
    // Don't process camera movement if menu is open or mouse control is disabled
    if menu_state.show_menu || !mouse_control.enabled {
        return;
    }

    if let Ok(mut camera_transform) = camera_query.single_mut() {
        let mut movement = Vec3::ZERO;
        let speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
            controller.movement_speed * controller.sprint_multiplier
        } else {
            controller.movement_speed
        };

        // Forward/Backward (W = forward, S = backward)
        if keyboard_input.pressed(KeyCode::KeyW) {
            movement += camera_transform.forward() * speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            movement -= camera_transform.forward() * speed * time.delta_secs();
        }

        // Left/Right
        if keyboard_input.pressed(KeyCode::KeyA) {
            movement -= camera_transform.right() * speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            movement += camera_transform.right() * speed * time.delta_secs();
        }

        // Up/Down
        if keyboard_input.pressed(KeyCode::Space) {
            movement += Vec3::Y * speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::ControlLeft) {
            movement -= Vec3::Y * speed * time.delta_secs();
        }

        camera_transform.translation += movement;
    }
}

pub fn camera_mouse_toggle_system(
    mut windows: Query<&mut Window>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_control: ResMut<CameraMouseControl>,
    menu_state: Res<MenuState>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        // Don't toggle mouse control if menu is open
        if menu_state.show_menu {
            return;
        }

        mouse_control.enabled = !mouse_control.enabled;

        if let Ok(mut window) = windows.single_mut() {
            window.cursor_options.visible = !mouse_control.enabled;
            window.cursor_options.grab_mode = if mouse_control.enabled {
                CursorGrabMode::Locked
            } else {
                CursorGrabMode::None
            };
        }
    }
}

pub fn camera_mouse_look_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
    mut controller: ResMut<CameraController>,
    mouse_control: Res<CameraMouseControl>,
    menu_state: Res<MenuState>,
) {
    // Don't process mouse look if menu is open or mouse control is disabled
    if !mouse_control.enabled || menu_state.show_menu {
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }
    if delta == Vec2::ZERO {
        return;
    }

    if let Ok(mut transform) = camera_query.single_mut() {
        // Yaw (around global Y)
        let yaw = -delta.x * controller.mouse_sensitivity;
        // Pitch (around local X)
        let pitch_delta = -delta.y * controller.mouse_sensitivity;

        // Update and clamp pitch
        controller.pitch = (controller.pitch + pitch_delta)
            .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

        // Apply yaw
        transform.rotate_y(yaw);

        // Set pitch by constructing a new rotation
        let yaw_rotation = Quat::from_rotation_y(transform.rotation.to_euler(EulerRot::YXZ).0);
        let pitch_rotation = Quat::from_rotation_x(controller.pitch);
        transform.rotation = yaw_rotation * pitch_rotation;
    }
}

pub fn input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reconnect_events: EventWriter<ReconnectRequestEvent>,
    mut menu_state: ResMut<MenuState>,
) {
    // Don't process game inputs if menu is open
    if menu_state.show_menu {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        reconnect_events.write(ReconnectRequestEvent);
    }

    if keyboard_input.just_pressed(KeyCode::F1) {
        menu_state.debug_mode = !menu_state.debug_mode;
        info!("Debug mode: {}", menu_state.debug_mode);
    }
}
