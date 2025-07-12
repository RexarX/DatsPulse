use crate::config::AppConfig;
use crate::input::CameraController;
use crate::renderer::RendererSettings;
use crate::types::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::pbr::wireframe::WireframeConfig;
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, WindowMode, WindowResolution};
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
            show_fps: false,        // Hidden by default
            show_connection: false, // Hidden by default
            show_debug_text: false, // Hidden by default
            show_game_state: false, // Hidden by default
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

pub fn setup_menu(mut commands: Commands, app_config: Res<AppConfig>) {
    let mut menu_state = MenuState::default();

    // Initialize FOV from camera or use default
    menu_state.fov = 75.0; // Default FOV in degrees

    commands.insert_resource(menu_state);
    commands.insert_resource(ResolutionOptions::default());
}

pub fn menu_toggle_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<MenuState>,
) {
    if keyboard_input.just_pressed(KeyCode::Insert) {
        menu_state.show_menu = !menu_state.show_menu;
    }
}

pub fn menu_ui_system(
    mut contexts: EguiContexts,
    mut menu_state: ResMut<MenuState>,
    mut camera_controller: ResMut<CameraController>,
    mut camera_transform_query: Query<&mut Transform, With<GameCamera>>,
    mut projection_query: Query<&mut Projection, With<GameCamera>>,
    mut windows: Query<&mut Window>,
    resolution_options: Res<ResolutionOptions>,
    mut reconnect_events: EventWriter<ReconnectRequestEvent>,
    mut app_config: ResMut<crate::config::AppConfig>,
    mut renderer_settings: ResMut<RendererSettings>,
    mut clear_color: ResMut<ClearColor>,
    mut wireframe_config: ResMut<WireframeConfig>,
    game_state: Res<GameState>,
    connection_state: Res<ConnectionState>,
    diagnostics: Res<DiagnosticsStore>,
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
        .default_width(600.0)
        .default_height(800.0)
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.heading("Game Settings");
            ui.separator();

            // Game Status Section
            ui.collapsing("Game Status", |ui| {
                // FPS
                if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                    if let Some(average) = fps.average() {
                        ui.label(format!("FPS: {:.1}", average));
                    }
                }

                // Connection Status
                if connection_state.connected {
                    ui.colored_label(egui::Color32::GREEN, "Status: Connected & Registered");
                } else {
                    ui.colored_label(
                        egui::Color32::RED,
                        format!("Status: {}", connection_state.connection_message),
                    );
                }

                // Game State Info
                if game_state.connected {
                    let ant_count = game_state.my_ants.len();
                    let enemy_count = game_state.enemy_ants.len();
                    let food_count = game_state.food_on_map.len();
                    let visible_tiles = game_state.visible_tiles.len();

                    // Calculate ant type distribution
                    let mut worker_count = 0;
                    let mut soldier_count = 0;
                    let mut scout_count = 0;
                    let mut carrying_food = 0;

                    for ant in game_state.my_ants.values() {
                        match ant.ant_type {
                            AntType::Worker => worker_count += 1,
                            AntType::Soldier => soldier_count += 1,
                            AntType::Scout => scout_count += 1,
                        }
                        carrying_food += ant.food.amount;
                    }

                    ui.separator();
                    ui.label(format!(
                        "Turn: {} | Score: {}",
                        game_state.turn_number, game_state.score
                    ));
                    ui.label(format!("Next turn: {:.1}s", game_state.next_turn_in));
                    ui.label(format!(
                        "Ants: {} (W:{} S:{} Sc:{})",
                        ant_count, worker_count, soldier_count, scout_count
                    ));
                    ui.label(format!("Enemies: {} | Food: {}", enemy_count, food_count));
                    ui.label(format!(
                        "Carrying: {} | Visible tiles: {}",
                        carrying_food, visible_tiles
                    ));
                    ui.label(format!(
                        "Home: ({}, {})",
                        game_state.main_spot.q, game_state.main_spot.r
                    ));
                } else {
                    ui.label("Game State: Disconnected");
                }
            });

            ui.separator();

            // Debug Settings
            ui.collapsing("Debug Settings", |ui| {
                ui.checkbox(&mut menu_state.debug_mode, "Debug Mode");
                ui.label("Toggle debug rendering and information");

                ui.separator();
                ui.label("Debug Rendering Options:");
                ui.checkbox(&mut menu_state.show_fps, "Show FPS Overlay");
                ui.checkbox(
                    &mut menu_state.show_connection,
                    "Show Connection Status Overlay",
                );
                ui.checkbox(&mut menu_state.show_debug_text, "Show Debug Text Overlay");
                ui.checkbox(&mut menu_state.show_game_state, "Show Game State Overlay");
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

                if ui.button("Apply Display Settings").clicked() {
                    apply_display_settings(&mut windows, &resolution_options, &menu_state);
                }
            });

            ui.separator();

            // Renderer Settings
            ui.collapsing("Renderer", |ui| {
                ui.label("Target FPS:");
                ui.horizontal(|ui| {
                    if ui
                        .radio_value(&mut app_config.renderer.target_fps, 30, "30")
                        .changed()
                    {
                        renderer_settings.target_fps = 30;
                    }
                    if ui
                        .radio_value(&mut app_config.renderer.target_fps, 60, "60")
                        .changed()
                    {
                        renderer_settings.target_fps = 60;
                    }
                    if ui
                        .radio_value(&mut app_config.renderer.target_fps, 120, "120")
                        .changed()
                    {
                        renderer_settings.target_fps = 120;
                    }
                    if ui
                        .radio_value(&mut app_config.renderer.target_fps, 144, "144")
                        .changed()
                    {
                        renderer_settings.target_fps = 144;
                    }
                    if ui
                        .radio_value(&mut app_config.renderer.target_fps, 0, "Unlimited")
                        .changed()
                    {
                        renderer_settings.target_fps = 0;
                    }
                });

                ui.separator();

                ui.label("Anisotropic Filtering:");
                ui.horizontal(|ui| {
                    if ui
                        .radio_value(&mut app_config.renderer.anisotropic_filtering, 1, "Off")
                        .changed()
                    {
                        renderer_settings.anisotropic_filtering = 1;
                    }
                    if ui
                        .radio_value(&mut app_config.renderer.anisotropic_filtering, 2, "2x")
                        .changed()
                    {
                        renderer_settings.anisotropic_filtering = 2;
                    }
                    if ui
                        .radio_value(&mut app_config.renderer.anisotropic_filtering, 4, "4x")
                        .changed()
                    {
                        renderer_settings.anisotropic_filtering = 4;
                    }
                    if ui
                        .radio_value(&mut app_config.renderer.anisotropic_filtering, 8, "8x")
                        .changed()
                    {
                        renderer_settings.anisotropic_filtering = 8;
                    }
                    if ui
                        .radio_value(&mut app_config.renderer.anisotropic_filtering, 16, "16x")
                        .changed()
                    {
                        renderer_settings.anisotropic_filtering = 16;
                    }
                });

                ui.separator();

                ui.label("Anti-Aliasing:");
                let mut aa_changed = false;
                ui.horizontal(|ui| {
                    if ui
                        .radio_value(
                            &mut app_config.renderer.anti_aliasing,
                            "none".to_string(),
                            "None",
                        )
                        .changed()
                    {
                        aa_changed = true;
                    }
                    if ui
                        .radio_value(
                            &mut app_config.renderer.anti_aliasing,
                            "msaa2".to_string(),
                            "MSAA 2x",
                        )
                        .changed()
                    {
                        aa_changed = true;
                    }
                    if ui
                        .radio_value(
                            &mut app_config.renderer.anti_aliasing,
                            "msaa4".to_string(),
                            "MSAA 4x",
                        )
                        .changed()
                    {
                        aa_changed = true;
                    }
                    if ui
                        .radio_value(
                            &mut app_config.renderer.anti_aliasing,
                            "msaa8".to_string(),
                            "MSAA 8x",
                        )
                        .changed()
                    {
                        aa_changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    if ui
                        .radio_value(
                            &mut app_config.renderer.anti_aliasing,
                            "fxaa".to_string(),
                            "FXAA",
                        )
                        .changed()
                    {
                        aa_changed = true;
                    }
                    if ui
                        .radio_value(
                            &mut app_config.renderer.anti_aliasing,
                            "smaa".to_string(),
                            "SMAA",
                        )
                        .changed()
                    {
                        aa_changed = true;
                    }
                    if ui
                        .radio_value(
                            &mut app_config.renderer.anti_aliasing,
                            "taa".to_string(),
                            "TAA",
                        )
                        .changed()
                    {
                        aa_changed = true;
                    }
                });

                if aa_changed {
                    renderer_settings.current_aa = crate::renderer::AntiAliasingMode::from(
                        app_config.renderer.anti_aliasing.as_str(),
                    );
                }

                ui.separator();

                if ui
                    .checkbox(
                        &mut app_config.renderer.ssao_enabled,
                        "Screen Space Ambient Occlusion (SSAO)",
                    )
                    .changed()
                {
                    renderer_settings.current_ssao = app_config.renderer.ssao_enabled;
                }

                ui.separator();

                ui.label("Clear Color:");
                let mut color_changed = false;
                ui.horizontal(|ui| {
                    ui.label("R:");
                    if ui
                        .add(egui::Slider::new(
                            &mut app_config.renderer.clear_color.0,
                            0.0..=1.0,
                        ))
                        .changed()
                    {
                        color_changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("G:");
                    if ui
                        .add(egui::Slider::new(
                            &mut app_config.renderer.clear_color.1,
                            0.0..=1.0,
                        ))
                        .changed()
                    {
                        color_changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("B:");
                    if ui
                        .add(egui::Slider::new(
                            &mut app_config.renderer.clear_color.2,
                            0.0..=1.0,
                        ))
                        .changed()
                    {
                        color_changed = true;
                    }
                });

                if color_changed {
                    clear_color.0 = Color::srgb(
                        app_config.renderer.clear_color.0,
                        app_config.renderer.clear_color.1,
                        app_config.renderer.clear_color.2,
                    );
                }

                if ui.button("Apply All Renderer Settings").clicked() {
                    apply_renderer_settings(
                        &mut windows,
                        &app_config,
                        &mut renderer_settings,
                        &mut clear_color,
                        &mut wireframe_config,
                    );
                }

                ui.label("Wireframe:");
                if ui
                    .checkbox(
                        &mut app_config.renderer.wireframe_enabled,
                        "Enable Wireframe",
                    )
                    .changed()
                {
                    wireframe_config.global = app_config.renderer.wireframe_enabled;
                }

                if app_config.renderer.wireframe_enabled {
                    ui.label("Wireframe Color:");
                    ui.horizontal(|ui| {
                        ui.label("R:");
                        if ui
                            .add(egui::Slider::new(
                                &mut app_config.renderer.wireframe_color.0,
                                0.0..=1.0,
                            ))
                            .changed()
                        {
                            wireframe_config.default_color = Color::srgb(
                                app_config.renderer.wireframe_color.0,
                                app_config.renderer.wireframe_color.1,
                                app_config.renderer.wireframe_color.2,
                            )
                            .into();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("G:");
                        if ui
                            .add(egui::Slider::new(
                                &mut app_config.renderer.wireframe_color.1,
                                0.0..=1.0,
                            ))
                            .changed()
                        {
                            wireframe_config.default_color = Color::srgb(
                                app_config.renderer.wireframe_color.0,
                                app_config.renderer.wireframe_color.1,
                                app_config.renderer.wireframe_color.2,
                            )
                            .into();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("B:");
                        if ui
                            .add(egui::Slider::new(
                                &mut app_config.renderer.wireframe_color.2,
                                0.0..=1.0,
                            ))
                            .changed()
                        {
                            wireframe_config.default_color = Color::srgb(
                                app_config.renderer.wireframe_color.0,
                                app_config.renderer.wireframe_color.1,
                                app_config.renderer.wireframe_color.2,
                            )
                            .into();
                        }
                    });
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
                    update_camera_fov(&mut projection_query, menu_state.fov);
                }

                ui.separator();

                ui.label("Movement Speed:");
                if ui
                    .add(
                        egui::Slider::new(&mut camera_controller.movement_speed, 1.0..=30.0)
                            .suffix(" units/s"),
                    )
                    .changed()
                {
                    // Update the app config when changed
                    app_config.camera.movement_speed = camera_controller.movement_speed;
                }

                ui.label("Sprint Multiplier:");
                if ui
                    .add(
                        egui::Slider::new(&mut camera_controller.sprint_multiplier, 1.0..=5.0)
                            .suffix("x"),
                    )
                    .changed()
                {
                    app_config.camera.sprint_multiplier = camera_controller.sprint_multiplier;
                }

                ui.label("Mouse Sensitivity:");
                if ui
                    .add(
                        egui::Slider::new(&mut camera_controller.mouse_sensitivity, 0.001..=2.0)
                            .step_by(0.001),
                    )
                    .changed()
                {
                    app_config.camera.mouse_sensitivity = camera_controller.mouse_sensitivity;
                }

                ui.separator();

                ui.label("Zoom Settings:");
                ui.label("Zoom Speed:");
                if ui
                    .add(
                        egui::Slider::new(&mut camera_controller.zoom_speed, 10.0..=100.0)
                            .suffix(" units/s"),
                    )
                    .changed()
                {
                    app_config.camera.zoom_speed = camera_controller.zoom_speed;
                }

                ui.label("Min Zoom:");
                if ui
                    .add(
                        egui::Slider::new(&mut camera_controller.min_zoom, 3.0..=20.0)
                            .suffix(" units"),
                    )
                    .changed()
                {
                    app_config.camera.min_zoom = camera_controller.min_zoom;
                    // Clamp current zoom if it's below the new minimum
                    if camera_controller.current_zoom < camera_controller.min_zoom {
                        camera_controller.current_zoom = camera_controller.min_zoom;
                        app_config.camera.current_zoom = camera_controller.current_zoom;
                    }
                }

                ui.label("Max Zoom:");
                if ui
                    .add(
                        egui::Slider::new(&mut camera_controller.max_zoom, 20.0..=100.0)
                            .suffix(" units"),
                    )
                    .changed()
                {
                    app_config.camera.max_zoom = camera_controller.max_zoom;
                    // Clamp current zoom if it's above the new maximum
                    if camera_controller.current_zoom > camera_controller.max_zoom {
                        camera_controller.current_zoom = camera_controller.max_zoom;
                        app_config.camera.current_zoom = camera_controller.current_zoom;
                    }
                }

                ui.label("Current Zoom:");
                let min_zoom = camera_controller.min_zoom;
                let max_zoom = camera_controller.max_zoom;
                if ui
                    .add(
                        egui::Slider::new(&mut camera_controller.current_zoom, min_zoom..=max_zoom)
                            .suffix(" units"),
                    )
                    .changed()
                {
                    app_config.camera.current_zoom = camera_controller.current_zoom;
                    // Apply the zoom change immediately to the camera
                    if let Ok(mut camera_transform) = camera_transform_query.single_mut() {
                        let current_pos = camera_transform.translation;
                        let target_pos =
                            Vec3::new(current_pos.x, camera_controller.current_zoom, current_pos.z);
                        camera_transform.translation = target_pos;
                    }
                }

                ui.separator();

                ui.label("Mouse Drag Settings:");
                ui.label("Drag Sensitivity:");
                if ui
                    .add(
                        egui::Slider::new(&mut app_config.camera.drag_sensitivity, 0.001..=0.1)
                            .step_by(0.001),
                    )
                    .changed()
                {
                    // Note: This will be applied to MouseDragState in the next frame
                    // We could add a system to sync this if needed
                }

                ui.separator();

                // Camera position display (read-only)
                if let Ok(camera_transform) = camera_transform_query.single() {
                    ui.label("Current Position:");
                    ui.label(format!(
                        "X: {:.1}, Y: {:.1}, Z: {:.1}",
                        camera_transform.translation.x,
                        camera_transform.translation.y,
                        camera_transform.translation.z
                    ));
                }

                // Reset camera button
                if ui.button("Reset Camera to Default").clicked() {
                    // Reset to default values
                    camera_controller.movement_speed = 15.0;
                    camera_controller.sprint_multiplier = 2.0;
                    camera_controller.mouse_sensitivity = 0.5;
                    camera_controller.zoom_speed = 50.0;
                    camera_controller.min_zoom = 5.0;
                    camera_controller.max_zoom = 50.0;
                    camera_controller.current_zoom = 20.0;

                    // Update app config
                    app_config.camera.movement_speed = camera_controller.movement_speed;
                    app_config.camera.sprint_multiplier = camera_controller.sprint_multiplier;
                    app_config.camera.mouse_sensitivity = camera_controller.mouse_sensitivity;
                    app_config.camera.zoom_speed = camera_controller.zoom_speed;
                    app_config.camera.min_zoom = camera_controller.min_zoom;
                    app_config.camera.max_zoom = camera_controller.max_zoom;
                    app_config.camera.current_zoom = camera_controller.current_zoom;
                    app_config.camera.drag_sensitivity = 0.01;

                    // Apply zoom change to camera
                    if let Ok(mut camera_transform) = camera_transform_query.single_mut() {
                        let current_pos = camera_transform.translation;
                        let target_pos =
                            Vec3::new(current_pos.x, camera_controller.current_zoom, current_pos.z);
                        camera_transform.translation = target_pos;
                    }
                }
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

            // Configuration
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

            // Controls Info
            ui.collapsing("Controls", |ui| {
                ui.label("Camera Controls:");
                ui.label("  WASD: Move camera");
                ui.label("  Space/Ctrl: Up/Down");
                ui.label("  Mouse: Look around (when enabled)");
                ui.label("  Shift: Sprint");
                ui.separator();
                ui.label("Game Controls:");
                ui.label("  F: Focus on home");
                ui.label("  R: Reconnect to server");
                ui.label("  L: Request game logs");
                ui.label("  M: Send test move commands");
                ui.separator();
                ui.label("UI Controls:");
                ui.label("  Insert: Toggle this menu");
                ui.label("  Escape: Toggle mouse control");
                ui.label("  F1: Toggle debug mode");
                ui.label("  O: Toggle occlusion culling");
                ui.label("  K: Show current skybox type");
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

fn apply_renderer_settings(
    windows: &mut Query<&mut Window>,
    app_config: &AppConfig,
    renderer_settings: &mut RendererSettings,
    clear_color: &mut ClearColor,
    wireframe_config: &mut WireframeConfig,
) {
    // Update renderer settings
    renderer_settings.current_aa =
        crate::renderer::AntiAliasingMode::from(app_config.renderer.anti_aliasing.as_str());
    renderer_settings.current_ssao = app_config.renderer.ssao_enabled;
    renderer_settings.target_fps = app_config.renderer.target_fps;
    renderer_settings.anisotropic_filtering = app_config.renderer.anisotropic_filtering;

    // Update clear color
    clear_color.0 = Color::srgb(
        app_config.renderer.clear_color.0,
        app_config.renderer.clear_color.1,
        app_config.renderer.clear_color.2,
    );

    // Update wireframe settings
    wireframe_config.global = app_config.renderer.wireframe_enabled;
    wireframe_config.default_color = Color::srgb(
        app_config.renderer.wireframe_color.0,
        app_config.renderer.wireframe_color.1,
        app_config.renderer.wireframe_color.2,
    )
    .into();

    // Update window settings
    if let Ok(mut window) = windows.single_mut() {
        window.resolution = bevy::window::WindowResolution::new(
            app_config.renderer.resolution.0 as f32,
            app_config.renderer.resolution.1 as f32,
        );

        window.present_mode = if app_config.renderer.vsync {
            bevy::window::PresentMode::AutoVsync
        } else {
            bevy::window::PresentMode::AutoNoVsync
        };

        window.mode = match app_config.renderer.window_mode.as_str() {
            "borderless" => bevy::window::WindowMode::BorderlessFullscreen(
                bevy::window::MonitorSelection::Primary,
            ),
            "fullscreen" => bevy::window::WindowMode::Fullscreen(
                bevy::window::MonitorSelection::Primary,
                bevy::window::VideoModeSelection::Current,
            ),
            _ => bevy::window::WindowMode::Windowed,
        };

        info!("Applied all renderer settings");
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

pub fn sync_fov_from_camera(
    mut menu_state: ResMut<MenuState>,
    camera_query: Query<&Projection, (With<GameCamera>, Changed<Projection>)>,
) {
    if let Ok(projection) = camera_query.single() {
        if let Projection::Perspective(perspective) = projection {
            menu_state.fov = perspective.fov.to_degrees();
        }
    }
}
