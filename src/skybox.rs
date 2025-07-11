use crate::types::GameCamera;
use bevy::{
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

#[derive(Resource)]
pub struct SkyboxManager {
    pub current_skybox: SkyboxType,
    pub skybox_handles: Vec<Handle<Image>>,
    pub is_loaded: bool,
    pub fallback_applied: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkyboxType {
    Cubemap,
    Fallback,
}

impl Default for SkyboxManager {
    fn default() -> Self {
        Self {
            current_skybox: SkyboxType::Cubemap,
            skybox_handles: Vec::new(),
            is_loaded: false,
            fallback_applied: false,
        }
    }
}

pub fn setup_skybox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut skybox_manager: ResMut<SkyboxManager>,
    mut camera_query: Query<Entity, With<GameCamera>>,
) {
    // Try to load the vertical strip cubemap
    let cubemap_path = "textures/skybox/cubemap_strip.png";
    let cubemap_handle = asset_server.load(cubemap_path);
    skybox_manager.skybox_handles.clear();
    skybox_manager.skybox_handles.push(cubemap_handle.clone());

    // Attach Skybox component to your camera(s)
    for camera_entity in camera_query.iter_mut() {
        commands.entity(camera_entity).insert(Skybox {
            image: cubemap_handle.clone(),
            brightness: 1000.0,
            ..default()
        });
    }
}

pub fn update_skybox(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut skybox_manager: ResMut<SkyboxManager>,
    mut camera_query: Query<&mut Skybox, With<GameCamera>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if skybox_manager.is_loaded {
        return;
    }

    if skybox_manager.skybox_handles.is_empty() {
        apply_fallback_skybox(&mut skybox_manager, &mut camera_query, &mut images);
        return;
    }

    let skybox_handle = &skybox_manager.skybox_handles[0];

    match asset_server.load_state(skybox_handle) {
        bevy::asset::LoadState::Loaded => {
            info!("Loading skybox texture...");

            if let Some(image) = images.get_mut(skybox_handle) {
                // Only reinterpret if it's a vertical strip (array_layer_count == 1)
                if image.texture_descriptor.array_layer_count() == 1 {
                    let layers = image.height() / image.width();
                    if layers == 6 {
                        image.reinterpret_stacked_2d_as_array(6);
                        image.texture_view_descriptor = Some(TextureViewDescriptor {
                            dimension: Some(TextureViewDimension::Cube),
                            ..default()
                        });

                        // Apply skybox to all cameras
                        for mut skybox in camera_query.iter_mut() {
                            skybox.image = skybox_handle.clone();
                            skybox.brightness = 1000.0;
                        }

                        skybox_manager.is_loaded = true;
                        skybox_manager.current_skybox = SkyboxType::Cubemap;
                        info!("Skybox loaded successfully!");
                    } else {
                        error!(
                            "Cubemap strip must have height = 6 * width! Got {}x{}",
                            image.width(),
                            image.height()
                        );
                        apply_fallback_skybox(&mut skybox_manager, &mut camera_query, &mut images);
                    }
                } else {
                    error!("Image already processed or invalid format");
                    apply_fallback_skybox(&mut skybox_manager, &mut camera_query, &mut images);
                }
            } else {
                error!("Failed to get image from handle");
                apply_fallback_skybox(&mut skybox_manager, &mut camera_query, &mut images);
            }
        }
        bevy::asset::LoadState::Failed(_) => {
            error!("Failed to load skybox texture, using fallback");
            apply_fallback_skybox(&mut skybox_manager, &mut camera_query, &mut images);
        }
        _ => {
            // Still loading, check if we should timeout and use fallback
            if !skybox_manager.fallback_applied {
                // You could add a timeout here if needed
            }
        }
    }
}

fn apply_fallback_skybox(
    skybox_manager: &mut SkyboxManager,
    camera_query: &mut Query<&mut Skybox, With<GameCamera>>,
    images: &mut ResMut<Assets<Image>>,
) {
    if skybox_manager.fallback_applied {
        return;
    }

    info!("Applying fallback skybox (solid black)");

    // Create a simple black cubemap
    let black_image = create_black_cubemap(images);

    for mut skybox in camera_query.iter_mut() {
        skybox.image = black_image.clone();
        skybox.brightness = 0.0; // Black skybox
    }

    skybox_manager.fallback_applied = true;
    skybox_manager.is_loaded = true;
    skybox_manager.current_skybox = SkyboxType::Fallback;
}

fn create_black_cubemap(images: &mut ResMut<Assets<Image>>) -> Handle<Image> {
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    let size = 64;
    let mut data = vec![0u8; (size * size * 4 * 6) as usize]; // RGBA, 6 faces

    // Fill with black (already 0, but being explicit)
    for chunk in data.chunks_mut(4) {
        chunk[0] = 0; // R
        chunk[1] = 0; // G
        chunk[2] = 0; // B
        chunk[3] = 255; // A
    }

    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });

    images.add(image)
}

pub fn toggle_skybox_type(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    skybox_manager: Res<SkyboxManager>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyK) {
        match skybox_manager.current_skybox {
            SkyboxType::Cubemap => info!("Current skybox: Cubemap"),
            SkyboxType::Fallback => info!("Current skybox: Fallback (Black)"),
        }
    }
}
