use crate::input::CameraController;
use crate::types::*;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, MonitorSelection, PresentMode, WindowMode, WindowResolution};
use bevy_egui::{EguiContexts, egui};

#[derive(Resource)]
pub struct MenuState {
    pub show_menu: bool,
    pub show_fps: bool,
    pub show_connection: bool,
    pub show_debug_text: bool,
    pub show_game_state: bool,
    pub debug_mode: bool,
    pub fov: f32,
    pub selected_resolution: usize,
    pub selected_window_mode: WindowModeWrapper,
    pub selected_present_mode: PresentModeWrapper,
    pub framerate_limit: FramerateLimit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowModeWrapper {
    Windowed,
    BorderlessFullscreen,
    Fullscreen,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PresentModeWrapper {
    Fifo,      // VSync on
    Immediate, // VSync off
    Mailbox,   // Adaptive VSync
    AutoVsync, // Let the system decide
}

#[derive(Debug, Clone, PartialEq)]
pub enum FramerateLimit {
    Unlimited,
    Limit30,
    Limit60,
    Limit120,
    Limit144,
    Limit240,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            show_menu: false,
            show_fps: true,
            show_connection: true,
            show_debug_text: true,
            show_game_state: true,
            debug_mode: false,
            fov: 75.0,
            selected_resolution: 2,
            selected_window_mode: WindowModeWrapper::Windowed,
            selected_present_mode: PresentModeWrapper::Fifo,
            framerate_limit: FramerateLimit::Unlimited,
        }
    }
}

#[derive(Resource)]
pub struct ResolutionOptions {
    pub resolutions: Vec<(u32, u32)>,
    pub labels: Vec<String>,
}

impl Default for ResolutionOptions {
    fn default() -> Self {
        let resolutions = vec![
            (800, 600),
            (1024, 768),
            (1280, 720),
            (1366, 768),
            (1920, 1080),
            (2560, 1440),
            (3840, 2160),
        ];

        let labels = resolutions
            .iter()
            .map(|(w, h)| format!("{}x{}", w, h))
            .collect();

        Self {
            resolutions,
            labels,
        }
    }
}

pub fn setup_menu(mut commands: Commands) {
    commands.insert_resource(MenuState::default());
    commands.insert_resource(ResolutionOptions::default());
}

pub fn menu_toggle_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<MenuState>,
    mut mouse_control: ResMut<crate::input::CameraMouseControl>,
    mut windows: Query<&mut Window>,
) {
    if keyboard_input.just_pressed(KeyCode::Insert) {
        menu_state.show_menu = !menu_state.show_menu;

        // When menu opens, disable mouse control and show cursor
        if menu_state.show_menu {
            mouse_control.enabled = false;
            if let Ok(mut window) = windows.single_mut() {
                window.cursor_options.visible = true;
                window.cursor_options.grab_mode = CursorGrabMode::None;
            }
        }
    }
}

pub fn menu_ui_system(
    mut contexts: EguiContexts,
    mut menu_state: ResMut<MenuState>,
    mut camera_controller: ResMut<CameraController>,
    mut camera_query: Query<&mut Projection, With<GameCamera>>,
    mut windows: Query<&mut Window>,
    resolution_options: Res<ResolutionOptions>,
    mut reconnect_events: EventWriter<ReconnectRequestEvent>,
    mut app_config: ResMut<crate::config::AppConfig>,
) -> Result {
    if !menu_state.show_menu {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    // Apply font size scaling
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(app_config.ui.menu_font_size, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(app_config.ui.menu_font_size, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(
            app_config.ui.menu_font_size * 1.2,
            egui::FontFamily::Proportional,
        ),
    );
    ctx.set_style(style);

    egui::Window::new(&app_config.ui.menu_title)
        .default_width(500.0)
        .default_height(700.0)
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.heading("Game Settings");
            ui.separator();

            // UI Settings
            ui.collapsing("UI Settings", |ui| {
                ui.checkbox(&mut menu_state.show_fps, "Show FPS");
                ui.checkbox(&mut menu_state.show_connection, "Show Connection Status");
                ui.checkbox(&mut menu_state.show_debug_text, "Show Debug Text");
                ui.checkbox(&mut menu_state.show_game_state, "Show Game State");

                ui.separator();

                ui.label("Menu Font Size:");
                ui.add(
                    egui::Slider::new(&mut app_config.ui.menu_font_size, 10.0..=30.0).suffix("px"),
                );

                ui.label("UI Font Size:");
                ui.add(
                    egui::Slider::new(&mut app_config.ui.ui_font_size, 12.0..=32.0).suffix("px"),
                );

                ui.label("Menu Title:");
                ui.text_edit_singleline(&mut app_config.ui.menu_title);
            });

            ui.separator();

            // Debug Settings
            ui.collapsing("Debug Settings", |ui| {
                ui.checkbox(&mut menu_state.debug_mode, "Debug Mode");
                ui.label("Toggle debug rendering and information");
            });

            ui.separator();

            // Display Settings
            ui.collapsing("Display Settings", |ui| {
                // Window Mode
                ui.label("Window Mode:");
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut menu_state.selected_window_mode,
                        WindowModeWrapper::Windowed,
                        "Windowed",
                    );
                    ui.radio_value(
                        &mut menu_state.selected_window_mode,
                        WindowModeWrapper::BorderlessFullscreen,
                        "Borderless",
                    );
                    ui.radio_value(
                        &mut menu_state.selected_window_mode,
                        WindowModeWrapper::Fullscreen,
                        "Fullscreen",
                    );
                });

                // Resolution
                ui.label("Resolution:");
                egui::ComboBox::from_id_salt("resolution_combo")
                    .selected_text(&resolution_options.labels[menu_state.selected_resolution])
                    .show_ui(ui, |ui| {
                        for (i, label) in resolution_options.labels.iter().enumerate() {
                            ui.selectable_value(&mut menu_state.selected_resolution, i, label);
                        }
                    });

                ui.separator();

                // VSync Settings
                ui.label("VSync Mode:");
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut menu_state.selected_present_mode,
                        PresentModeWrapper::Fifo,
                        "VSync On",
                    );
                    ui.radio_value(
                        &mut menu_state.selected_present_mode,
                        PresentModeWrapper::Immediate,
                        "VSync Off",
                    );
                });
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut menu_state.selected_present_mode,
                        PresentModeWrapper::Mailbox,
                        "Adaptive",
                    );
                    ui.radio_value(
                        &mut menu_state.selected_present_mode,
                        PresentModeWrapper::AutoVsync,
                        "Auto",
                    );
                });

                // Framerate Limit
                ui.label("Framerate Limit:");
                egui::ComboBox::from_id_salt("framerate_combo")
                    .selected_text(match menu_state.framerate_limit {
                        FramerateLimit::Unlimited => "Unlimited",
                        FramerateLimit::Limit30 => "30 FPS",
                        FramerateLimit::Limit60 => "60 FPS",
                        FramerateLimit::Limit120 => "120 FPS",
                        FramerateLimit::Limit144 => "144 FPS",
                        FramerateLimit::Limit240 => "240 FPS",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut menu_state.framerate_limit,
                            FramerateLimit::Unlimited,
                            "Unlimited",
                        );
                        ui.selectable_value(
                            &mut menu_state.framerate_limit,
                            FramerateLimit::Limit30,
                            "30 FPS",
                        );
                        ui.selectable_value(
                            &mut menu_state.framerate_limit,
                            FramerateLimit::Limit60,
                            "60 FPS",
                        );
                        ui.selectable_value(
                            &mut menu_state.framerate_limit,
                            FramerateLimit::Limit120,
                            "120 FPS",
                        );
                        ui.selectable_value(
                            &mut menu_state.framerate_limit,
                            FramerateLimit::Limit144,
                            "144 FPS",
                        );
                        ui.selectable_value(
                            &mut menu_state.framerate_limit,
                            FramerateLimit::Limit240,
                            "240 FPS",
                        );
                    });

                if ui.button("Apply Display Settings").clicked() {
                    apply_display_settings(&mut windows, &resolution_options, &menu_state);
                }
            });

            ui.separator();

            // Camera Settings
            ui.collapsing("Camera Settings", |ui| {
                ui.label("Field of View:");
                if ui
                    .add(egui::Slider::new(&mut menu_state.fov, 45.0..=120.0).suffix("°"))
                    .changed()
                {
                    update_camera_fov(&mut camera_query, menu_state.fov);
                }

                ui.label("Mouse Sensitivity:");
                ui.add(
                    egui::Slider::new(&mut camera_controller.mouse_sensitivity, 0.001..=0.01)
                        .step_by(0.001),
                );

                ui.label("Movement Speed:");
                ui.add(
                    egui::Slider::new(&mut camera_controller.movement_speed, 1.0..=20.0)
                        .suffix(" units/s"),
                );

                ui.label("Sprint Multiplier:");
                ui.add(
                    egui::Slider::new(&mut camera_controller.sprint_multiplier, 1.0..=5.0)
                        .suffix("x"),
                );
            });

            ui.separator();

            // Server Settings
            ui.collapsing("Server", |ui| {
                if ui.button("Reconnect to Server").clicked() {
                    reconnect_events.write(ReconnectRequestEvent);
                }
                ui.label("Force reconnection to the game server");
            });

            ui.separator();

            // Save/Load Configuration
            ui.collapsing("Configuration", |ui| {
                if ui.button("Save Configuration").clicked() {
                    if let Err(e) = app_config.save(std::path::Path::new("config.toml")) {
                        error!("Failed to save configuration: {}", e);
                    } else {
                        info!("Configuration saved successfully");
                    }
                }
                ui.label("Save current settings to config.toml");
            });

            ui.separator();

            // Close button
            if ui.button("Close Menu").clicked() {
                menu_state.show_menu = false;
            }
        });

    Ok(())
}

fn apply_display_settings(
    windows: &mut Query<&mut Window>,
    resolution_options: &ResolutionOptions,
    menu_state: &MenuState,
) {
    if let Ok(mut window) = windows.single_mut() {
        let (width, height) = resolution_options.resolutions[menu_state.selected_resolution];

        window.resolution = WindowResolution::new(width as f32, height as f32);
        window.mode = match menu_state.selected_window_mode {
            WindowModeWrapper::Windowed => WindowMode::Windowed,
            WindowModeWrapper::BorderlessFullscreen => {
                WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
            }
            WindowModeWrapper::Fullscreen => {
                WindowMode::Fullscreen(MonitorSelection::Primary, VideoModeSelection::Current)
            }
        };

        // Apply VSync settings
        window.present_mode = match menu_state.selected_present_mode {
            PresentModeWrapper::Fifo => PresentMode::Fifo,
            PresentModeWrapper::Immediate => PresentMode::Immediate,
            PresentModeWrapper::Mailbox => PresentMode::Mailbox,
            PresentModeWrapper::AutoVsync => PresentMode::AutoVsync,
        };

        info!(
            "Applied display settings: {}x{}, VSync: {:?}",
            width, height, window.present_mode
        );
    }
}

fn update_camera_fov(camera_query: &mut Query<&mut Projection, With<GameCamera>>, fov: f32) {
    if let Ok(mut projection) = camera_query.single_mut() {
        if let Projection::Perspective(perspective) = projection.as_mut() {
            perspective.fov = fov.to_radians();
        }
    }
}

// System to update UI visibility based on menu state
pub fn update_ui_visibility(
    menu_state: Res<MenuState>,
    mut fps_query: Query<
        &mut Visibility,
        (
            With<crate::ui::FpsText>,
            Without<crate::ui::ConnectionText>,
            Without<crate::ui::DebugText>,
            Without<crate::ui::GameStateText>,
        ),
    >,
    mut connection_query: Query<
        &mut Visibility,
        (
            With<crate::ui::ConnectionText>,
            Without<crate::ui::FpsText>,
            Without<crate::ui::DebugText>,
            Without<crate::ui::GameStateText>,
        ),
    >,
    mut debug_query: Query<
        &mut Visibility,
        (
            With<crate::ui::DebugText>,
            Without<crate::ui::FpsText>,
            Without<crate::ui::ConnectionText>,
            Without<crate::ui::GameStateText>,
        ),
    >,
    mut game_state_query: Query<
        &mut Visibility,
        (
            With<crate::ui::GameStateText>,
            Without<crate::ui::FpsText>,
            Without<crate::ui::ConnectionText>,
            Without<crate::ui::DebugText>,
        ),
    >,
) {
    // Update FPS visibility
    if let Ok(mut visibility) = fps_query.single_mut() {
        *visibility = if menu_state.show_fps {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Update connection status visibility
    if let Ok(mut visibility) = connection_query.single_mut() {
        *visibility = if menu_state.show_connection {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Update debug text visibility
    if let Ok(mut visibility) = debug_query.single_mut() {
        *visibility = if menu_state.show_debug_text {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Update game state visibility
    if let Ok(mut visibility) = game_state_query.single_mut() {
        *visibility = if menu_state.show_game_state {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

// System to handle framerate limiting
pub fn framerate_limiter_system(menu_state: Res<MenuState>, time: Res<Time>) {
    use std::thread;
    use std::time::Duration;

    let target_frame_time = match menu_state.framerate_limit {
        FramerateLimit::Unlimited => return,
        FramerateLimit::Limit30 => Duration::from_secs_f64(1.0 / 30.0),
        FramerateLimit::Limit60 => Duration::from_secs_f64(1.0 / 60.0),
        FramerateLimit::Limit120 => Duration::from_secs_f64(1.0 / 120.0),
        FramerateLimit::Limit144 => Duration::from_secs_f64(1.0 / 144.0),
        FramerateLimit::Limit240 => Duration::from_secs_f64(1.0 / 240.0),
    };

    let frame_time = time.delta();
    if frame_time < target_frame_time {
        let sleep_time = target_frame_time - frame_time;
        thread::sleep(sleep_time);
    }
}
