use bevy::{
    core_pipeline::{
        experimental::taa::TemporalAntiAliasing,
        prepass::{DepthPrepass, MotionVectorPrepass},
        smaa::{Smaa, SmaaPreset},
    },
    pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel},
    prelude::*,
    render::camera::{MipBias, TemporalJitter},
    window::PresentMode,
};

use crate::{WireframeConfig, config::AppConfig, types::GameCamera};

#[derive(Resource, Clone)]
pub struct RendererSettings {
    pub current_aa: AntiAliasingMode,
    pub current_ssao: bool,
    pub target_fps: u32,
    pub anisotropic_filtering: u32,
    pub wireframe_enabled: bool,
    pub settings_changed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AntiAliasingMode {
    None,
    Msaa2,
    Msaa4,
    Msaa8,
    Fxaa,
    Smaa,
    Taa,
}

impl From<&str> for AntiAliasingMode {
    fn from(s: &str) -> Self {
        match s {
            "msaa2" => AntiAliasingMode::Msaa2,
            "msaa4" => AntiAliasingMode::Msaa4,
            "msaa8" => AntiAliasingMode::Msaa8,
            "fxaa" => AntiAliasingMode::Fxaa,
            "smaa" => AntiAliasingMode::Smaa,
            "taa" => AntiAliasingMode::Taa,
            _ => AntiAliasingMode::None,
        }
    }
}

impl Default for RendererSettings {
    fn default() -> Self {
        Self {
            current_aa: AntiAliasingMode::Msaa4,
            current_ssao: false,
            target_fps: 60,
            anisotropic_filtering: 16,
            wireframe_enabled: false,
            settings_changed: false,
        }
    }
}

pub fn setup_renderer(mut commands: Commands, app_config: Res<AppConfig>) {
    let renderer_settings = RendererSettings {
        current_aa: AntiAliasingMode::from(app_config.renderer.anti_aliasing.as_str()),
        current_ssao: app_config.renderer.ssao_enabled,
        target_fps: app_config.renderer.target_fps,
        anisotropic_filtering: app_config.renderer.anisotropic_filtering,
        wireframe_enabled: app_config.renderer.wireframe_enabled,
        settings_changed: false,
    };

    commands.insert_resource(renderer_settings.clone());

    info!(
        "Renderer settings initialized: AA={:?}, SSAO={}, FPS={}, AF={}x, Wireframe={}",
        renderer_settings.current_aa,
        renderer_settings.current_ssao,
        renderer_settings.target_fps,
        renderer_settings.anisotropic_filtering,
        renderer_settings.wireframe_enabled
    );
}

pub fn apply_anti_aliasing(
    mut commands: Commands,
    camera_query: Query<Entity, With<GameCamera>>,
    renderer_settings: Res<RendererSettings>,
) {
    if !renderer_settings.settings_changed {
        return;
    }

    for camera_entity in camera_query.iter() {
        let mut camera_commands = commands.entity(camera_entity);

        // Remove all existing AA components
        camera_commands
            .remove::<Msaa>()
            .remove::<Smaa>()
            .remove::<TemporalAntiAliasing>()
            .remove::<TemporalJitter>()
            .remove::<MipBias>()
            .remove::<DepthPrepass>()
            .remove::<MotionVectorPrepass>();

        // Apply the selected anti-aliasing
        match renderer_settings.current_aa {
            AntiAliasingMode::None => {
                camera_commands.insert(Msaa::Off);
            }
            AntiAliasingMode::Msaa2 => {
                camera_commands.insert(Msaa::Sample2);
            }
            AntiAliasingMode::Msaa4 => {
                camera_commands.insert(Msaa::Sample4);
            }
            AntiAliasingMode::Msaa8 => {
                camera_commands.insert(Msaa::Sample8);
            }
            AntiAliasingMode::Fxaa => {
                // FXAA is built into the default pipeline in Bevy 0.16
                // Just disable MSAA and it will use FXAA automatically
                camera_commands.insert(Msaa::Off);
            }
            AntiAliasingMode::Smaa => {
                camera_commands.insert(Msaa::Off).insert(Smaa {
                    preset: SmaaPreset::High,
                });
            }
            AntiAliasingMode::Taa => {
                camera_commands
                    .insert(Msaa::Off)
                    .insert(TemporalAntiAliasing::default());
            }
        }

        info!("Applied anti-aliasing: {:?}", renderer_settings.current_aa);
    }
}

pub fn apply_ssao(
    mut commands: Commands,
    camera_query: Query<Entity, With<GameCamera>>,
    renderer_settings: Res<RendererSettings>,
) {
    if !renderer_settings.settings_changed {
        return;
    }

    for camera_entity in camera_query.iter() {
        let mut camera_commands = commands.entity(camera_entity);

        if renderer_settings.current_ssao {
            camera_commands.insert(ScreenSpaceAmbientOcclusion {
                quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
                constant_object_thickness: 0.15,
            });
        } else {
            camera_commands.remove::<ScreenSpaceAmbientOcclusion>();
        }

        info!(
            "SSAO {}",
            if renderer_settings.current_ssao {
                "enabled"
            } else {
                "disabled"
            }
        );
    }
}

pub fn apply_framerate_limit(renderer_settings: Res<RendererSettings>, time: Res<Time>) {
    if renderer_settings.target_fps == 0 {
        return; // Unlimited FPS
    }

    let target_frame_time =
        std::time::Duration::from_secs_f64(1.0 / renderer_settings.target_fps as f64);
    let frame_time = time.delta();

    if frame_time < target_frame_time {
        let sleep_time = target_frame_time - frame_time;
        std::thread::sleep(sleep_time);
    }
}

pub fn apply_window_settings(
    mut windows: Query<&mut Window>,
    app_config: Res<AppConfig>,
    renderer_settings: Res<RendererSettings>,
) {
    if !renderer_settings.settings_changed {
        return;
    }

    for mut window in windows.iter_mut() {
        // Apply resolution
        window.resolution = bevy::window::WindowResolution::new(
            app_config.renderer.resolution.0 as f32,
            app_config.renderer.resolution.1 as f32,
        );

        // Apply VSync
        window.present_mode = if app_config.renderer.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };

        // Apply window mode
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

        info!(
            "Applied window settings: {}x{}, VSync: {}, Mode: {}",
            app_config.renderer.resolution.0,
            app_config.renderer.resolution.1,
            app_config.renderer.vsync,
            app_config.renderer.window_mode
        );
    }
}

pub fn apply_clear_color(
    mut clear_color: ResMut<ClearColor>,
    app_config: Res<AppConfig>,
    renderer_settings: Res<RendererSettings>,
) {
    if renderer_settings.settings_changed {
        clear_color.0 = Color::srgb(
            app_config.renderer.clear_color.0,
            app_config.renderer.clear_color.1,
            app_config.renderer.clear_color.2,
        );

        info!(
            "Applied clear color: RGB({}, {}, {})",
            app_config.renderer.clear_color.0,
            app_config.renderer.clear_color.1,
            app_config.renderer.clear_color.2
        );
    }
}

pub fn apply_wireframe_settings(
    mut commands: Commands,
    wireframe_config: Res<WireframeConfig>,
    app_config: Res<AppConfig>,
    renderer_settings: Res<RendererSettings>,
    // Query for mesh entities that could have wireframe
    mesh_query: Query<Entity, With<Mesh3d>>,
    // Query for entities that already have wireframe
    wireframe_query: Query<Entity, With<bevy::pbr::wireframe::Wireframe>>,
) {
    if !renderer_settings.settings_changed {
        return;
    }

    if app_config.renderer.wireframe_enabled {
        // Add wireframe to all mesh entities that don't have it
        for entity in mesh_query.iter() {
            if !wireframe_query.contains(entity) {
                commands
                    .entity(entity)
                    .insert(bevy::pbr::wireframe::Wireframe);
            }
        }
    } else {
        // Remove wireframe from all entities that have it
        for entity in wireframe_query.iter() {
            commands
                .entity(entity)
                .remove::<bevy::pbr::wireframe::Wireframe>();
        }
    }

    info!(
        "Wireframe settings applied: {}",
        app_config.renderer.wireframe_enabled
    );
}

// System to update renderer settings from config
pub fn update_renderer_settings(
    mut renderer_settings: ResMut<RendererSettings>,
    app_config: Res<AppConfig>,
) {
    if app_config.is_changed() {
        let new_aa = AntiAliasingMode::from(app_config.renderer.anti_aliasing.as_str());
        let new_ssao = app_config.renderer.ssao_enabled;
        let new_fps = app_config.renderer.target_fps;
        let new_af = app_config.renderer.anisotropic_filtering;

        // Only mark as changed if something actually changed
        if renderer_settings.current_aa != new_aa
            || renderer_settings.current_ssao != new_ssao
            || renderer_settings.target_fps != new_fps
            || renderer_settings.anisotropic_filtering != new_af
        {
            renderer_settings.current_aa = new_aa;
            renderer_settings.current_ssao = new_ssao;
            renderer_settings.target_fps = new_fps;
            renderer_settings.anisotropic_filtering = new_af;
            renderer_settings.settings_changed = true;
        }
    }
}

// System to reset the change flag after all systems have run
pub fn reset_renderer_settings_changed(mut renderer_settings: ResMut<RendererSettings>) {
    renderer_settings.settings_changed = false;
}
