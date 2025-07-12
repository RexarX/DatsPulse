use crate::input::CameraController;
use crate::menu::MenuState;
use crate::types::*;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource)]
pub struct RenderingAssets {
    pub ant_materials: HashMap<AntType, Handle<StandardMaterial>>,
    pub food_materials: HashMap<FoodType, Handle<StandardMaterial>>,
    pub tile_materials: HashMap<TileType, Handle<StandardMaterial>>,
    pub home_material: Handle<StandardMaterial>,
    pub enemy_material: Handle<StandardMaterial>,
    pub ground_material: Handle<StandardMaterial>,
    pub ant_mesh: Handle<Mesh>,
    pub food_mesh: Handle<Mesh>,
    pub hex_mesh: Handle<Mesh>,
    pub home_mesh: Handle<Mesh>,
    pub ground_plane_mesh: Handle<Mesh>,
}

#[derive(Component)]
pub struct GroundPlane;

#[derive(Component)]
pub struct PersistentHex;

pub fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera positioned like Civilization - above and angled down
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 25.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        GameCamera,
    ));

    // Directional Light from above-side
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 0.7, -0.8)),
    ));

    // Ambient light for better visibility
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
        affects_lightmapped_meshes: false,
    });

    // Create proper hexagon mesh with correct proportions
    let hex_mesh = create_proper_hexagon_mesh();
    let hex_mesh_handle = meshes.add(hex_mesh);

    // Create ground plane mesh (large plane for the entire game field)
    let ground_plane = create_ground_plane_mesh(200.0);
    let ground_plane_handle = meshes.add(ground_plane);

    // Other meshes
    let ant_mesh = meshes.add(Sphere::new(0.25));
    let food_mesh = meshes.add(Sphere::new(0.15));
    let home_mesh = meshes.add(Cylinder::new(0.8, 0.2));

    // Ground material
    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.3), // Green
        metallic: 0.0,
        perceptual_roughness: 0.9,
        ..default()
    });

    // Ant materials with better colors
    let mut ant_materials = HashMap::new();
    ant_materials.insert(
        AntType::Worker,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.6, 0.2), // Golden
            metallic: 0.1,
            perceptual_roughness: 0.8,
            ..default()
        }),
    );
    ant_materials.insert(
        AntType::Soldier,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2), // Red
            metallic: 0.2,
            perceptual_roughness: 0.7,
            ..default()
        }),
    );
    ant_materials.insert(
        AntType::Scout,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 0.2), // Green
            metallic: 0.1,
            perceptual_roughness: 0.8,
            ..default()
        }),
    );

    // Food materials
    let mut food_materials = HashMap::new();
    food_materials.insert(
        FoodType::Apple,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.1, 0.1), // Red
            metallic: 0.0,
            perceptual_roughness: 0.9,
            ..default()
        }),
    );
    food_materials.insert(
        FoodType::Bread,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.7, 0.3), // Golden brown
            metallic: 0.0,
            perceptual_roughness: 0.9,
            ..default()
        }),
    );
    food_materials.insert(
        FoodType::Nectar,
        materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.9, 0.1), // Bright yellow
            metallic: 0.0,
            perceptual_roughness: 0.8,
            ..default()
        }),
    );

    // Tile materials with better visibility
    let mut tile_materials = HashMap::new();

    tile_materials.insert(
        TileType::Plain,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.8, 0.6), // Light green
            metallic: 0.0,
            perceptual_roughness: 0.9,
            ..default()
        }),
    );

    tile_materials.insert(
        TileType::Dirt,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.4, 0.2), // Brown
            metallic: 0.0,
            perceptual_roughness: 0.8,
            ..default()
        }),
    );

    tile_materials.insert(
        TileType::Acid,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.3, 0.8), // Purple
            metallic: 0.0,
            perceptual_roughness: 0.3,
            emissive: LinearRgba::new(0.2, 0.1, 0.3, 1.0),
            ..default()
        }),
    );

    tile_materials.insert(
        TileType::Rock,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5), // Gray
            metallic: 0.1,
            perceptual_roughness: 0.8,
            ..default()
        }),
    );

    tile_materials.insert(
        TileType::Anthill,
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.3, 0.8), // Blue
            metallic: 0.0,
            perceptual_roughness: 0.7,
            ..default()
        }),
    );

    let home_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.9), // Bright blue
        metallic: 0.2,
        perceptual_roughness: 0.6,
        emissive: LinearRgba::new(0.0, 0.0, 0.3, 1.0),
        ..default()
    });

    let enemy_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.1, 0.9), // Magenta
        metallic: 0.1,
        perceptual_roughness: 0.7,
        ..default()
    });

    // Spawn ground plane
    commands.spawn((
        Mesh3d(ground_plane_handle.clone()),
        MeshMaterial3d(ground_material.clone()),
        Transform::from_xyz(0.0, -0.05, 0.0), // Slightly below hex level
        GroundPlane,
    ));

    commands.insert_resource(RenderingAssets {
        ant_materials,
        food_materials,
        tile_materials,
        home_material,
        enemy_material,
        ground_material,
        ant_mesh,
        food_mesh,
        hex_mesh: hex_mesh_handle,
        home_mesh,
        ground_plane_mesh: ground_plane_handle,
    });
}

pub fn update_world_rendering(
    mut commands: Commands,
    game_state: Res<GameState>,
    rendering_assets: Res<RenderingAssets>,
    ant_query: Query<Entity, (With<AntMarker>, Without<PersistentHex>)>,
    food_query: Query<Entity, (With<FoodMarker>, Without<PersistentHex>)>,
    home_query: Query<Entity, (With<HomeMarker>, Without<PersistentHex>)>,
    existing_hex_query: Query<&TileMarker, With<PersistentHex>>,
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

    let mut existing_hexes: std::collections::HashSet<HexCoord> = existing_hex_query
        .iter()
        .map(|marker| marker.position)
        .collect();

    // Always render some hexes for visibility
    if existing_hexes.is_empty() {
        // Render default grid
        if let Some(default_material) = rendering_assets.tile_materials.get(&TileType::Plain) {
            for q in -10..=10 {
                for r in -10..=10 {
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
                    existing_hexes.insert(hex_pos);
                }
            }
        }
    }

    // If we have game data, render game-specific hexes
    if game_state.connected && !game_state.visible_tiles.is_empty() {
        for (pos, tile) in &game_state.visible_tiles {
            if !existing_hexes.contains(pos) {
                if let Some(material) = rendering_assets.tile_materials.get(&tile.tile_type) {
                    let position = pos.to_vec3();
                    let mut transform = Transform::from_translation(position);
                    transform.scale = Vec3::splat(0.95);

                    commands.spawn((
                        Mesh3d(rendering_assets.hex_mesh.clone()),
                        MeshMaterial3d(material.clone()),
                        transform,
                        TileMarker {
                            tile_type: tile.tile_type,
                            position: *pos,
                        },
                        PersistentHex,
                    ));
                    existing_hexes.insert(*pos);
                }
            }
        }
    }

    // Render home tiles with special highlighting
    for (index, home_pos) in game_state.home_tiles.iter().enumerate() {
        let position = home_pos.to_vec3() + Vec3::Y * 0.15;
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

    // Count units per hex for proper displacement
    let mut units_per_hex: HashMap<HexCoord, Vec<(String, UnitType)>> = HashMap::new();

    // Count my ants
    for (ant_id, ant) in &game_state.my_ants {
        units_per_hex
            .entry(ant.position)
            .or_insert_with(Vec::new)
            .push((ant_id.clone(), UnitType::Ant));
    }

    // Count enemy ants
    for (enemy_id, enemy) in &game_state.enemy_ants {
        units_per_hex
            .entry(enemy.position)
            .or_insert_with(Vec::new)
            .push((enemy_id.clone(), UnitType::Enemy));
    }

    // Count food
    for (pos, _food) in &game_state.food_on_map {
        units_per_hex
            .entry(*pos)
            .or_insert_with(Vec::new)
            .push((format!("food_{}", pos.q), UnitType::Food));
    }

    // Render my ants with position offsets to avoid overlap
    for (ant_id, ant) in &game_state.my_ants {
        let units_on_hex = units_per_hex.get(&ant.position).unwrap();
        let ant_index = units_on_hex
            .iter()
            .position(|(id, t)| id == ant_id && *t == UnitType::Ant)
            .unwrap_or(0);

        let base_position = ant.position.to_vec3() + Vec3::Y * 0.3;
        let offset = get_unit_offset(ant_index, UnitType::Ant, units_on_hex.len());
        let position = base_position + offset;

        if let Some(material) = rendering_assets.ant_materials.get(&ant.ant_type) {
            let mut transform = Transform::from_translation(position);
            let health_ratio = ant.health as f32 / ant.ant_type.health() as f32;
            transform.scale = Vec3::splat(0.8 + health_ratio * 0.4);

            commands.spawn((
                Mesh3d(rendering_assets.ant_mesh.clone()),
                MeshMaterial3d(material.clone()),
                transform,
                AntMarker {
                    ant_id: ant_id.clone(),
                    ant_type: ant.ant_type,
                    is_enemy: false,
                },
            ));
        }
    }

    // Render enemy ants with position offsets
    for (enemy_id, enemy) in &game_state.enemy_ants {
        let units_on_hex = units_per_hex.get(&enemy.position).unwrap();
        let enemy_index = units_on_hex
            .iter()
            .position(|(id, t)| id == enemy_id && *t == UnitType::Enemy)
            .unwrap_or(0);

        let base_position = enemy.position.to_vec3() + Vec3::Y * 0.3;
        let offset = get_unit_offset(enemy_index, UnitType::Enemy, units_on_hex.len());
        let position = base_position + offset;

        let mut transform = Transform::from_translation(position);

        // Scale based on health and make slightly larger to distinguish
        let health_ratio = enemy.health as f32 / enemy.ant_type.health() as f32;
        transform.scale = Vec3::splat(0.9 + health_ratio * 0.3);

        commands.spawn((
            Mesh3d(rendering_assets.ant_mesh.clone()),
            MeshMaterial3d(rendering_assets.enemy_material.clone()),
            transform,
            AntMarker {
                ant_id: enemy_id.clone(),
                ant_type: enemy.ant_type,
                is_enemy: true,
            },
        ));
    }

    // Render food with position offsets
    for (pos, food) in &game_state.food_on_map {
        let units_on_hex = units_per_hex.get(pos).unwrap();
        let food_id = format!("food_{}", pos.q);
        let food_index = units_on_hex
            .iter()
            .position(|(id, t)| id == &food_id && *t == UnitType::Food)
            .unwrap_or(0);

        let base_position = pos.to_vec3() + Vec3::Y * 0.2;
        let offset = get_unit_offset(food_index, UnitType::Food, units_on_hex.len());
        let position = base_position + offset;

        if let Some(material) = rendering_assets.food_materials.get(&food.food_type) {
            let mut transform = Transform::from_translation(position);

            // Scale based on amount
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
        let world_pos = pos.to_vec3();
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
            let mut prev_pos = ant.position.to_vec3() + Vec3::Y * 0.5;

            for hex_pos in &ant.current_move {
                let world_pos = hex_pos.to_vec3() + Vec3::Y * 0.5;
                gizmos.line(prev_pos, world_pos, Color::srgb(0.0, 1.0, 1.0));
                prev_pos = world_pos;
            }
        }
    }

    // Draw vision ranges for scouts
    for ant in game_state.my_ants.values() {
        if ant.ant_type == AntType::Scout {
            let center = ant.position.to_vec3() + Vec3::Y * 0.1;
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
        let center = game_state.main_spot.to_vec3();

        if let Ok(mut camera_transform) = camera_query.single_mut() {
            camera_transform.translation = Vec3::new(center.x, controller.current_zoom, center.z);
            camera_transform.look_at(center, Vec3::Y);
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
        indices.extend_from_slice(&[0, current as u32, next as u32]);
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

fn create_ground_plane_mesh(size: f32) -> Mesh {
    let half_size = size / 2.0;

    let positions = vec![
        [-half_size, 0.0, -half_size],
        [half_size, 0.0, -half_size],
        [half_size, 0.0, half_size],
        [-half_size, 0.0, half_size],
    ];

    let normals = vec![
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];

    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    let indices = vec![0, 1, 2, 0, 2, 3];

    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
}

fn get_hex_corners(center: Vec3) -> [Vec3; 6] {
    let size = 0.866;
    let mut corners = [Vec3::ZERO; 6];

    for i in 0..6 {
        let angle = std::f32::consts::PI / 3.0 * i as f32 + std::f32::consts::PI / 6.0;
        corners[i] = center + Vec3::new(size * angle.cos(), 0.0, size * angle.sin());
    }

    corners
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum UnitType {
    Ant,
    Enemy,
    Food,
}

fn get_unit_offset(index: usize, unit_type: UnitType, total_units_on_hex: usize) -> Vec3 {
    // Only displace if there are multiple units on the same hex
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
