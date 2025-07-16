use crate::input::CameraController;
use crate::menu::MenuState;
use crate::types::*;
use bevy::color::palettes;
use bevy::math::prelude::*;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use std::collections::{HashMap, HashSet};

#[derive(Resource)]
pub struct RenderingAssets {
    pub food_materials: HashMap<FoodType, Handle<StandardMaterial>>,
    pub tile_materials: HashMap<TileType, Handle<StandardMaterial>>,
    pub home_material: Handle<StandardMaterial>,
    pub ground_material: Handle<StandardMaterial>,
    pub ant_model: Handle<Scene>,
    pub food_mesh: Handle<Mesh>,
    pub hex_mesh: Handle<Mesh>,
    pub home_mesh: Handle<Mesh>,
}

#[derive(Component)]
pub struct PersistentHex;

#[derive(Component)]
pub struct ColorOverride(Color);

pub fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 25.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        GameCamera,
    ));

    // Directional Light (brighter)
    commands.spawn((
        DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 0.7, -0.8)),
    ));

    // Ambient light (brighter)
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 2000.0,
        ..default()
    });

    // Create meshes
    let hex_mesh = create_proper_hexagon_mesh();
    let hex_mesh_handle = meshes.add(hex_mesh);
    let food_mesh = meshes.add(Sphere::new(0.15));
    let home_mesh = meshes.add(Cylinder::new(0.8, 0.2));

    // Load ant glTF model
    let ant_model = asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/ant/scene.gltf"));
    // Food materials
    let mut food_materials = HashMap::new();
    food_materials.insert(
        FoodType::Apple,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.1, 0.1),
            ..default()
        }),
    );
    food_materials.insert(
        FoodType::Bread,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.7, 0.3),
            ..default()
        }),
    );
    food_materials.insert(
        FoodType::Nectar,
        materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.9, 0.1),
            ..default()
        }),
    );

    // Enhanced tile materials with additional types
    let mut tile_materials = HashMap::new();

    // Visible tile types
    tile_materials.insert(
        TileType::Plain,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.7, 0.4),
            metallic: 0.0,
            perceptual_roughness: 0.9,
            ..default()
        }),
    );
    tile_materials.insert(
        TileType::Dirt,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.4, 0.2),
            metallic: 0.0,
            perceptual_roughness: 0.8,
            ..default()
        }),
    );
    tile_materials.insert(
        TileType::Acid,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.3, 0.8),
            emissive: LinearRgba::new(0.2, 0.1, 0.3, 1.0),
            metallic: 0.0,
            perceptual_roughness: 0.3,
            ..default()
        }),
    );
    tile_materials.insert(
        TileType::Rock,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5),
            metallic: 0.3,
            perceptual_roughness: 0.4,
            ..default()
        }),
    );
    tile_materials.insert(
        TileType::Anthill,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.3, 0.8),
            metallic: 0.0,
            perceptual_roughness: 0.8,
            ..default()
        }),
    );

    // Special materials for non-visible tiles
    tile_materials.insert(
        TileType::Unknown,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.3), // Gray for unknown tiles
            metallic: 0.0,
            perceptual_roughness: 0.95,
            ..default()
        }),
    );

    let home_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.9),
        emissive: LinearRgba::new(0.0, 0.0, 0.3, 1.0),
        metallic: 0.2,
        perceptual_roughness: 0.6,
        ..default()
    });

    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.3, 0.2),
        metallic: 0.0,
        perceptual_roughness: 0.95,
        ..default()
    });

    commands.insert_resource(RenderingAssets {
        food_materials,
        tile_materials,
        home_material,
        ground_material,
        ant_model,
        food_mesh,
        hex_mesh: hex_mesh_handle,
        home_mesh,
    });
}

pub fn update_world_rendering(
    mut commands: Commands,
    game_state: Res<GameState>,
    rendering_assets: Res<RenderingAssets>,
    ant_query: Query<Entity, (With<AntMarker>, Without<PersistentHex>)>,
    food_query: Query<Entity, (With<FoodMarker>, Without<PersistentHex>)>,
    home_query: Query<Entity, (With<HomeMarker>, Without<PersistentHex>)>,
    existing_hex_query: Query<(Entity, &TileMarker), With<PersistentHex>>,
) {
    // Clear dynamic entities
    for entity in ant_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in food_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in home_query.iter() {
        commands.entity(entity).despawn();
    }

    // Create a comprehensive hex grid
    let grid_size = 50; // Adjust as needed
    let existing_hexes: HashMap<HexCoord, Entity> = existing_hex_query
        .iter()
        .map(|(entity, marker)| (marker.position, entity))
        .collect();

    // Generate grid in odd-r layout
    for r in -grid_size..=grid_size {
        for q in -grid_size..=grid_size {
            let hex_pos = HexCoord::new(q, r);
            let world_pos = hex_pos_to_world_oddr(hex_pos);

            // Determine hex type and material
            let (tile_type, material) =
                determine_hex_appearance(&hex_pos, &game_state, &rendering_assets);

            // Update existing hex or create new one
            if let Some(entity) = existing_hexes.get(&hex_pos) {
                // Update existing hex with new material
                commands.entity(*entity).insert(MeshMaterial3d(material));
            } else {
                // Create new hex
                commands.spawn((
                    Mesh3d(rendering_assets.hex_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(world_pos).with_scale(Vec3::splat(0.95)),
                    TileMarker {
                        tile_type,
                        position: hex_pos,
                    },
                    PersistentHex,
                ));
            }
        }
    }

    // Continue with rendering other entities
    render_home_tiles(&mut commands, &game_state, &rendering_assets);
    render_ants(&mut commands, &game_state, &rendering_assets);
    render_food(&mut commands, &game_state, &rendering_assets);
}

pub fn render_default_hex_grid(
    mut commands: Commands,
    rendering_assets: Res<RenderingAssets>,
    game_state: Res<GameState>,
) {
    // Only render default grid if we don't have game data yet
    if game_state.connected && !game_state.visible_tiles.is_empty() {
        return;
    }

    // Create a simple grid for visualization when not connected
    if let Some(default_material) = rendering_assets.tile_materials.get(&TileType::Plain) {
        for q in -8..=8 {
            for r in -8..=8 {
                let hex_pos = HexCoord::new(q, r);
                let world_pos = hex_pos.to_vec3();

                commands.spawn((
                    Mesh3d(rendering_assets.hex_mesh.clone()),
                    MeshMaterial3d(default_material.clone()),
                    Transform::from_translation(world_pos).with_scale(Vec3::splat(0.95)),
                    TileMarker {
                        tile_type: TileType::Plain,
                        position: hex_pos,
                    },
                    PersistentHex,
                ));
            }
        }
    }
}

pub fn debug_rendering_system(
    mut gizmos: Gizmos,
    menu_state: Res<MenuState>,
    game_state: Res<GameState>,
) {
    if !menu_state.debug_mode {
        return;
    }

    // Draw coordinate axes
    gizmos.line(Vec3::ZERO, Vec3::X * 10.0, Color::srgb(1.0, 0.0, 0.0));
    gizmos.line(Vec3::ZERO, Vec3::Y * 10.0, Color::srgb(0.0, 1.0, 0.0));
    gizmos.line(Vec3::ZERO, Vec3::Z * 10.0, Color::srgb(0.0, 0.0, 1.0));

    // Draw hex grid outlines
    for (pos, _tile) in &game_state.visible_tiles {
        let world_pos = hex_pos_to_world_oddr(*pos);
        let hex_corners = get_hex_corners(world_pos);

        // Draw hex outline
        for i in 0..6 {
            let start = hex_corners[i] + Vec3::Y * 0.01;
            let end = hex_corners[(i + 1) % 6] + Vec3::Y * 0.01;
            gizmos.line(start, end, Color::srgb(0.8, 0.8, 0.8));
        }
    }

    // Draw ant movement paths - use actual ant position, not displaced
    for ant in game_state.my_ants.values() {
        if !ant.current_move.is_empty() {
            let mut prev_pos = hex_pos_to_world_oddr(ant.position) + Vec3::Y * 0.5;

            for hex_pos in &ant.current_move {
                let world_pos = hex_pos_to_world_oddr(*hex_pos) + Vec3::Y * 0.5;
                gizmos.line(prev_pos, world_pos, Color::srgb(0.0, 1.0, 1.0));
                prev_pos = world_pos;
            }
        }
    }

    // Draw vision ranges for scouts
    for ant in game_state.my_ants.values() {
        if ant.ant_type == AntType::Scout {
            let center = hex_pos_to_world_oddr(ant.position) + Vec3::Y * 0.1;
            let radius = ant.ant_type.view_range() as f32 * 1.73; // sqrt(3) for proper hex radius

            gizmos.circle(
                Isometry3d::from_translation(center)
                    * Isometry3d::from_rotation(Quat::from_rotation_x(
                        -std::f32::consts::FRAC_PI_2,
                    )),
                radius,
                Color::srgba(0.0, 1.0, 0.0, 0.3),
            );
        }
    }
}

pub fn update_camera_focus(
    game_state: Res<GameState>,
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<AntMarker>)>,
    input: Res<ButtonInput<KeyCode>>,
    controller: Res<CameraController>,
) {
    if !game_state.connected {
        return;
    }

    // Focus camera on main spot when F is pressed
    if input.just_pressed(KeyCode::KeyF) {
        let center = hex_pos_to_world_oddr(game_state.main_spot);

        if let Ok(mut camera_transform) = camera_query.single_mut() {
            camera_transform.translation = Vec3::new(center.x, controller.current_zoom, center.z);
            camera_transform.look_at(center, Vec3::Y);
        }
    }
}

pub fn change_material(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    color_override: Query<&ColorOverride>,
    mesh_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    mut asset_materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get the `ColorOverride` of the entity, if it does not have a color override, skip
    let Ok(color_override) = color_override.get(trigger.target()) else {
        return;
    };

    // Iterate over all children recursively
    for descendants in children.iter_descendants(trigger.target()) {
        // Get the material of the descendant
        if let Some(material) = mesh_materials
            .get(descendants)
            .ok()
            .and_then(|id| asset_materials.get_mut(id.id()))
        {
            // Create a copy of the material and override base color
            // If you intend on creating multiple models with the same tint, it
            // is best to cache the handle somewhere, as having multiple materials
            // that are identical is expensive
            let mut new_material = material.clone();
            new_material.base_color = color_override.0;

            // Override `MeshMaterial3d` with new material
            commands
                .entity(descendants)
                .insert(MeshMaterial3d(asset_materials.add(new_material)));
        }
    }
}

// Separate function for rendering home tiles
fn render_home_tiles(
    commands: &mut Commands,
    game_state: &GameState,
    rendering_assets: &RenderingAssets,
) {
    for home_pos in &game_state.home_tiles {
        let position = hex_pos_to_world_oddr(*home_pos) + Vec3::Y * 0.15;
        let is_main = *home_pos == game_state.main_spot;
        let scale = if is_main { 1.3 } else { 1.1 };

        commands.spawn((
            Mesh3d(rendering_assets.home_mesh.clone()),
            MeshMaterial3d(rendering_assets.home_material.clone()),
            Transform::from_translation(position).with_scale(Vec3::splat(scale)),
            HomeMarker {
                is_main_spot: is_main,
            },
        ));
    }
}

// Separate function for rendering ants
fn render_ants(
    commands: &mut Commands,
    game_state: &GameState,
    rendering_assets: &RenderingAssets,
) {
    // Count units per hex for proper displacement
    let mut units_per_hex: HashMap<HexCoord, Vec<(String, UnitType)>> = HashMap::new();

    for (ant_id, ant) in &game_state.my_ants {
        units_per_hex
            .entry(ant.position)
            .or_insert_with(Vec::new)
            .push((ant_id.clone(), UnitType::Ant));
    }

    for (enemy_id, enemy) in &game_state.enemy_ants {
        units_per_hex
            .entry(enemy.position)
            .or_insert_with(Vec::new)
            .push((enemy_id.clone(), UnitType::Enemy));
    }

    // Render my ants
    for (ant_id, ant) in &game_state.my_ants {
        let units_on_hex = units_per_hex.get(&ant.position).unwrap();
        let ant_index = units_on_hex
            .iter()
            .position(|(id, t)| id == ant_id && *t == UnitType::Ant)
            .unwrap_or(0);

        let base_position = hex_pos_to_world_oddr(ant.position) + Vec3::Y * 0.3;
        let offset = get_unit_offset(ant_index, UnitType::Ant, units_on_hex.len());
        let position = base_position + offset;

        let health_ratio = ant.health as f32 / ant.ant_type.health() as f32;
        let scale = (0.8 + health_ratio * 0.5) * 0.005;

        let ant_marker = AntMarker {
            ant_id: ant_id.clone(),
            ant_type: ant.ant_type,
            is_enemy: false,
        };
        if let Some(color) = get_ant_color(&ant_marker) {
            commands.spawn((
                SceneRoot(rendering_assets.ant_model.clone()),
                Transform::from_translation(position).with_scale(Vec3::splat(scale)),
                ColorOverride(color),
                ant_marker,
            ));
        } else {
            commands.spawn((
                SceneRoot(rendering_assets.ant_model.clone()),
                Transform::from_translation(position).with_scale(Vec3::splat(scale)),
                ant_marker,
            ));
        }
    }

    // Render enemy ants
    for (enemy_id, enemy) in &game_state.enemy_ants {
        let units_on_hex = units_per_hex.get(&enemy.position).unwrap();
        let enemy_index = units_on_hex
            .iter()
            .position(|(id, t)| id == enemy_id && *t == UnitType::Enemy)
            .unwrap_or(0);

        let base_position = hex_pos_to_world_oddr(enemy.position) + Vec3::Y * 0.3;
        let offset = get_unit_offset(enemy_index, UnitType::Enemy, units_on_hex.len());
        let position = base_position + offset;

        let health_ratio = enemy.health as f32 / enemy.ant_type.health() as f32;
        let scale = (1.0 + health_ratio * 0.5) * 0.005;

        let ant_marker = AntMarker {
            ant_id: enemy_id.clone(),
            ant_type: enemy.ant_type,
            is_enemy: true,
        };
        if let Some(color) = get_ant_color(&ant_marker) {
            commands.spawn((
                SceneRoot(rendering_assets.ant_model.clone()),
                Transform::from_translation(position).with_scale(Vec3::splat(scale)),
                ColorOverride(color),
                ant_marker,
            ));
        } else {
            commands.spawn((
                SceneRoot(rendering_assets.ant_model.clone()),
                Transform::from_translation(position).with_scale(Vec3::splat(scale)),
                ant_marker,
            ));
        }
        commands.spawn((
            SceneRoot(rendering_assets.ant_model.clone()),
            Transform::from_translation(position).with_scale(Vec3::splat(scale)),
            AntMarker {
                ant_id: enemy_id.clone(),
                ant_type: enemy.ant_type,
                is_enemy: true,
            },
        ));
    }
}

// Separate function for rendering food
fn render_food(
    commands: &mut Commands,
    game_state: &GameState,
    rendering_assets: &RenderingAssets,
) {
    for (pos, food) in &game_state.food_on_map {
        let position = hex_pos_to_world_oddr(*pos) + Vec3::Y * 0.2;

        if let Some(material) = rendering_assets.food_materials.get(&food.food_type) {
            let mut transform = Transform::from_translation(position);
            let scale = 0.6 + (food.amount as f32 / 10.0).min(0.8);
            transform.scale = Vec3::splat(scale);

            commands.spawn((
                Mesh3d(rendering_assets.food_mesh.clone()),
                MeshMaterial3d(material.clone()),
                transform,
                FoodMarker {
                    food_type: food.food_type,
                    amount: food.amount,
                },
            ));
        }
    }
}

fn create_proper_hexagon_mesh() -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Use exact hex proportions for proper alignment
    let hex_size = 1.0; // This will be our base unit

    // Center vertex
    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    // Create 6 vertices for pointy-top hexagon
    for i in 0..6 {
        let angle = std::f32::consts::PI / 3.0 * i as f32; // 60 degrees apart
        let x = hex_size * angle.cos();
        let z = hex_size * angle.sin();

        positions.push([x, 0.0, z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);
    }

    // Create triangles
    for i in 0..6 {
        let current = i + 1;
        let next = if i == 5 { 1 } else { i + 2 };
        indices.extend_from_slice(&[0, next as u32, current as u32]);
    }

    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
}

fn determine_hex_appearance(
    hex_pos: &HexCoord,
    game_state: &GameState,
    rendering_assets: &RenderingAssets,
) -> (TileType, Handle<StandardMaterial>) {
    // Check if hex is visible in game state
    if let Some(tile) = game_state.visible_tiles.get(hex_pos) {
        // Visible tile - use actual tile type
        let material = rendering_assets
            .tile_materials
            .get(&tile.tile_type)
            .unwrap_or(&rendering_assets.tile_materials[&TileType::Plain])
            .clone();
        (tile.tile_type, material)
    } else {
        // Not visible - use gray material for unknown tiles
        let material = rendering_assets
            .tile_materials
            .get(&TileType::Unknown)
            .unwrap_or(&rendering_assets.tile_materials[&TileType::Plain])
            .clone();
        (TileType::Unknown, material)
    }
}

fn get_ant_color(ant_marker: &AntMarker) -> Option<Color> {
    if ant_marker.is_enemy {
        // Red tint for enemies with slight glow
        return Some(Color::srgb(1.0, 0.0, 0.0));
    }
    // Class-based colors for friendly ants
    match ant_marker.ant_type {
        AntType::Worker => None,
        AntType::Soldier => Some(Color::srgb(0.0, 1.0, 0.0)),
        AntType::Scout => Some(Color::srgb(0.0, 0.0, 1.0)),
    }
}

fn get_hex_corners(center: Vec3) -> [Vec3; 6] {
    let size = 0.866;
    let mut corners = [Vec3::ZERO; 6];

    for i in 0..6 {
        let angle = std::f32::consts::PI / 3.0 * i as f32;
        corners[i] = center + Vec3::new(size * angle.cos(), 0.0, size * angle.sin());
    }

    corners
}

fn hex_pos_to_world_oddr(hex: HexCoord) -> Vec3 {
    let size = 1.0;
    let width = size * 2.0;
    let height = size * 1.732050808; // sqrt(3)

    let x = size * (3.0 / 2.0 * hex.q as f32);
    let z = height * (hex.r as f32 + 0.5 * (hex.q & 1) as f32);

    Vec3::new(x, 0.0, z)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum UnitType {
    Ant,
    Enemy,
    Food,
}

fn get_unit_offset(index: usize, unit_type: UnitType, total_units_on_hex: usize) -> Vec3 {
    if total_units_on_hex <= 1 {
        return Vec3::ZERO;
    }
    let base_radius = match unit_type {
        UnitType::Ant => 0.2,
        UnitType::Enemy => 0.25,
        UnitType::Food => 0.15,
    };
    let angle = (index as f32 * 2.0 * std::f32::consts::PI / total_units_on_hex as f32)
        % (2.0 * std::f32::consts::PI);
    let radius = base_radius + (index as f32 * 0.05).min(0.3);
    Vec3::new(radius * angle.cos(), 0.0, radius * angle.sin())
}
