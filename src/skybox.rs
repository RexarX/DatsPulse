use crate::types::GameCamera;
use bevy::{
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

#[derive(Resource)]
pub struct SkyboxManager {
    pub current_skybox: SkyboxType,
    pub skybox_handle: Option<Handle<Image>>,
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
            skybox_handle: None,
            is_loaded: false,
            fallback_applied: false,
        }
    }
}

pub fn setup_skybox(asset_server: Res<AssetServer>, mut skybox_manager: ResMut<SkyboxManager>) {
    // Try to load the vertical strip cubemap
    let cubemap_handle = asset_server.load("textures/skybox/cubemap_strip.png");
    skybox_manager.skybox_handle = Some(cubemap_handle);

    info!("Attempting to load skybox from: textures/skybox/cubemap_strip.png");
}

pub fn update_skybox(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut skybox_manager: ResMut<SkyboxManager>,
    mut camera_query: Query<Entity, (With<GameCamera>, Without<Skybox>)>,
    mut commands: Commands,
) {
    if skybox_manager.is_loaded {
        return;
    }

    let Some(skybox_handle) = &skybox_manager.skybox_handle else {
        apply_fallback_skybox(
            &mut skybox_manager,
            &mut camera_query,
            &mut images,
            &mut commands,
        );
        return;
    };

    match asset_server.load_state(skybox_handle) {
        bevy::asset::LoadState::Loaded => {
            info!("Skybox texture loaded, processing...");

            if let Some(image) = images.get_mut(skybox_handle) {
                // Only reinterpret if it's a 2D texture that needs to be converted to cubemap
                if image.texture_descriptor.array_layer_count() == 1 {
                    let width = image.width();
                    let height = image.height();

                    // Check if this is a vertical strip (6:1 aspect ratio)
                    if height == width * 6 {
                        info!("Converting vertical strip to cubemap array");
                        image.reinterpret_stacked_2d_as_array(6);
                        image.texture_view_descriptor = Some(TextureViewDescriptor {
                            dimension: Some(TextureViewDimension::Cube),
                            ..default()
                        });
                    } else {
                        error!(
                            "Skybox image dimensions incorrect. Expected height = 6 * width, got {}x{}",
                            width, height
                        );
                        apply_fallback_skybox(
                            &mut skybox_manager,
                            &mut camera_query,
                            &mut images,
                            &mut commands,
                        );
                        return;
                    }
                }

                // Apply skybox to all cameras without skybox
                for camera_entity in camera_query.iter() {
                    commands.entity(camera_entity).insert(Skybox {
                        image: skybox_handle.clone(),
                        brightness: 1000.0,
                        ..default()
                    });
                }

                skybox_manager.is_loaded = true;
                skybox_manager.current_skybox = SkyboxType::Cubemap;
                info!("Skybox applied successfully!");
            } else {
                error!("Failed to get skybox image from asset handle");
                apply_fallback_skybox(
                    &mut skybox_manager,
                    &mut camera_query,
                    &mut images,
                    &mut commands,
                );
            }
        }
        bevy::asset::LoadState::Failed(err) => {
            error!("Failed to load skybox texture: {:?}", err);
            apply_fallback_skybox(
                &mut skybox_manager,
                &mut camera_query,
                &mut images,
                &mut commands,
            );
        }
        _ => {
            // Still loading - we could add a timeout here if needed
        }
    }
}

fn apply_fallback_skybox(
    skybox_manager: &mut SkyboxManager,
    camera_query: &mut Query<Entity, (With<GameCamera>, Without<Skybox>)>,
    images: &mut ResMut<Assets<Image>>,
    commands: &mut Commands,
) {
    if skybox_manager.fallback_applied {
        return;
    }

    info!("Applying fallback skybox (solid black)");

    // Create a simple black cubemap
    let black_image = create_black_cubemap(images);

    for camera_entity in camera_query.iter() {
        commands.entity(camera_entity).insert(Skybox {
            image: black_image.clone(),
            brightness: 0.0,
            ..default()
        });
    }

    skybox_manager.fallback_applied = true;
    skybox_manager.is_loaded = true;
    skybox_manager.current_skybox = SkyboxType::Fallback;
}

fn create_black_cubemap(images: &mut ResMut<Assets<Image>>) -> Handle<Image> {
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    let size = 64;
    let mut data = vec![0u8; (size * size * 4 * 6) as usize]; // RGBA, 6 faces

    // Fill with black
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
