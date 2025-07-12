use crate::AppConfig;
use crate::menu::MenuState;
use crate::types::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

#[derive(Resource)]
pub struct CameraController {
    pub movement_speed: f32,
    pub sprint_multiplier: f32,
    pub mouse_sensitivity: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub current_zoom: f32,
    pub target_zoom: f32,
    pub target_position: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            movement_speed: 15.0,
            sprint_multiplier: 2.0,
            mouse_sensitivity: 0.5,
            zoom_speed: 50.0,
            min_zoom: 5.0,
            max_zoom: 50.0,
            current_zoom: 20.0,
            target_zoom: 20.0,
            target_position: Vec3::default(),
        }
    }
}

#[derive(Resource)]
pub struct MouseDragState {
    pub is_dragging: bool,
    pub last_mouse_pos: Vec2,
    pub drag_sensitivity: f32,
}

impl Default for MouseDragState {
    fn default() -> Self {
        Self {
            is_dragging: false,
            last_mouse_pos: Vec2::ZERO,
            drag_sensitivity: 0.01,
        }
    }
}

#[derive(Resource, Default)]
pub struct CameraMouseControl {
    pub enabled: bool,
}

pub fn setup_input(mut commands: Commands, app_config: Res<AppConfig>) {
    let controller = CameraController {
        movement_speed: app_config.camera.movement_speed,
        sprint_multiplier: app_config.camera.sprint_multiplier,
        mouse_sensitivity: app_config.camera.mouse_sensitivity,
        zoom_speed: app_config.camera.zoom_speed,
        min_zoom: app_config.camera.min_zoom,
        max_zoom: app_config.camera.max_zoom,
        current_zoom: app_config.camera.current_zoom,
        target_zoom: app_config.camera.current_zoom,
        target_position: Vec3::new(0.0, 0.0, 0.0),
    };

    let drag_state = MouseDragState {
        is_dragging: false,
        last_mouse_pos: Vec2::ZERO,
        drag_sensitivity: app_config.camera.drag_sensitivity,
    };

    commands.insert_resource(controller);
    commands.insert_resource(drag_state);
    commands.insert_resource(CameraMouseControl::default());
}

pub fn camera_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
    mut controller: ResMut<CameraController>,
    mut drag_state: ResMut<MouseDragState>,
    time: Res<Time>,
    menu_state: Res<MenuState>,
    windows: Query<&Window>,
) {
    if menu_state.show_menu {
        return;
    }

    if let Ok(mut camera_transform) = camera_query.single_mut() {
        let mut movement = Vec3::ZERO;
        let speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
            controller.movement_speed * controller.sprint_multiplier
        } else {
            controller.movement_speed
        };

        if keyboard_input.pressed(KeyCode::KeyW) {
            movement.z -= speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            movement.z += speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            movement.x -= speed * time.delta_secs();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            movement.x += speed * time.delta_secs();
        }

        if mouse_button_input.pressed(MouseButton::Right) {
            if !drag_state.is_dragging {
                drag_state.is_dragging = true;
                if let Ok(window) = windows.single() {
                    if let Some(cursor_pos) = window.cursor_position() {
                        drag_state.last_mouse_pos = cursor_pos;
                    }
                }
            } else {
                for event in mouse_motion_events.read() {
                    // Zoom scaling
                    let zoom_ratio = (controller.current_zoom - controller.min_zoom)
                        / (controller.max_zoom - controller.min_zoom);
                    let min_drag_multiplier = 0.1;
                    let max_drag_multiplier = 2.0;
                    let drag_multiplier = min_drag_multiplier
                        + (max_drag_multiplier - min_drag_multiplier) * zoom_ratio;
                    let zoom_adjusted_sensitivity = drag_state.drag_sensitivity * drag_multiplier;

                    let drag_delta = event.delta * zoom_adjusted_sensitivity;

                    // Only move in XZ plane (ignore Y)
                    controller.target_position.z += drag_delta.x;
                    controller.target_position.x -= drag_delta.y;
                }
            }
        } else {
            drag_state.is_dragging = false;
        }

        for event in scroll_events.read() {
            let zoom_delta = event.y * controller.zoom_speed * 0.15;
            controller.target_zoom = (controller.target_zoom - zoom_delta)
                .clamp(controller.min_zoom, controller.max_zoom);
        }

        controller.target_position += movement;

        let lerp_speed = 10.0;
        camera_transform.translation = camera_transform.translation.lerp(
            Vec3::new(
                controller.target_position.x,
                controller.current_zoom,
                controller.target_position.z,
            ),
            lerp_speed * time.delta_secs(),
        );

        controller.current_zoom = controller
            .current_zoom
            .lerp(controller.target_zoom, lerp_speed * time.delta_secs());

        let look_at_target = Vec3::new(
            camera_transform.translation.x,
            0.0,
            camera_transform.translation.z,
        );
        camera_transform.look_at(look_at_target, Vec3::Y);
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

pub fn sync_camera_settings(
    app_config: Res<AppConfig>,
    mut drag_state: ResMut<MouseDragState>,
    mut controller: ResMut<CameraController>,
) {
    // Only sync if config has changed
    if app_config.is_changed() {
        drag_state.drag_sensitivity = app_config.camera.drag_sensitivity;

        // Sync other settings that might have been changed externally
        controller.movement_speed = app_config.camera.movement_speed;
        controller.sprint_multiplier = app_config.camera.sprint_multiplier;
        controller.mouse_sensitivity = app_config.camera.mouse_sensitivity;
        controller.zoom_speed = app_config.camera.zoom_speed;
        controller.min_zoom = app_config.camera.min_zoom;
        controller.max_zoom = app_config.camera.max_zoom;
        controller.current_zoom = app_config.camera.current_zoom;
    }
}

pub fn input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut reconnect_events: EventWriter<ReconnectRequestEvent>,
    mut register_events: EventWriter<RegisterRequestEvent>,
    mut menu_state: ResMut<MenuState>,
) {
    // Don't process game inputs if menu is open
    if menu_state.show_menu {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        reconnect_events.write(ReconnectRequestEvent);
    }

    if keyboard_input.just_pressed(KeyCode::KeyG) {
        register_events.write(RegisterRequestEvent);
    }

    if keyboard_input.just_pressed(KeyCode::F1) {
        menu_state.debug_mode = !menu_state.debug_mode;
        info!("Debug mode: {}", menu_state.debug_mode);
    }
}
