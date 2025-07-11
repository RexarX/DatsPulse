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
    pub ant_mesh: Handle<Mesh>,
    pub food_mesh: Handle<Mesh>,
    pub tile_mesh: Handle<Mesh>,
    pub home_mesh: Handle<Mesh>,
}

pub fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        GameCamera,
    ));

    // Directional Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -std::f32::consts::FRAC_PI_4,
        )),
    ));

    // Setup rendering assets
    let ant_mesh = meshes.add(Sphere::new(0.3));
    let food_mesh = meshes.add(Sphere::new(0.2));
    let tile_mesh = meshes.add(Cylinder::new(0.8, 0.1));
    let home_mesh = meshes.add(Cylinder::new(0.9, 0.2));

    // Ant materials
    let mut ant_materials = HashMap::new();
    ant_materials.insert(
        AntType::Worker,
        materials.add(Color::srgb(0.8, 0.6, 0.2)), // Brown
    );
    ant_materials.insert(
        AntType::Soldier,
        materials.add(Color::srgb(0.8, 0.2, 0.2)), // Red
    );
    ant_materials.insert(
        AntType::Scout,
        materials.add(Color::srgb(0.2, 0.8, 0.2)), // Green
    );

    // Food materials
    let mut food_materials = HashMap::new();
    food_materials.insert(
        FoodType::Apple,
        materials.add(Color::srgb(0.8, 0.1, 0.1)), // Red
    );
    food_materials.insert(
        FoodType::Bread,
        materials.add(Color::srgb(0.8, 0.6, 0.2)), // Orange
    );
    food_materials.insert(
        FoodType::Nectar,
        materials.add(Color::srgb(0.9, 0.8, 0.1)), // Yellow
    );

    // Tile materials
    let mut tile_materials = HashMap::new();
    tile_materials.insert(
        TileType::Plain,
        materials.add(Color::srgb(0.4, 0.6, 0.3)), // Green
    );
    tile_materials.insert(
        TileType::Dirt,
        materials.add(Color::srgb(0.6, 0.4, 0.2)), // Brown
    );
    tile_materials.insert(
        TileType::Acid,
        materials.add(Color::srgb(0.6, 0.2, 0.8)), // Purple
    );
    tile_materials.insert(
        TileType::Rock,
        materials.add(Color::srgb(0.5, 0.5, 0.5)), // Gray
    );
    tile_materials.insert(
        TileType::Anthill,
        materials.add(Color::srgb(0.3, 0.3, 0.8)), // Blue
    );

    let home_material = materials.add(Color::srgb(0.1, 0.1, 0.8)); // Blue
    let enemy_material = materials.add(Color::srgb(0.8, 0.1, 0.8)); // Magenta

    commands.insert_resource(RenderingAssets {
        ant_materials,
        food_materials,
        tile_materials,
        home_material,
        enemy_material,
        ant_mesh,
        food_mesh,
        tile_mesh,
        home_mesh,
    });
}

pub fn update_world_rendering(
    mut commands: Commands,
    game_state: Res<GameState>,
    rendering_assets: Res<RenderingAssets>,

    // Query existing entities to update or despawn
    ant_query: Query<Entity, With<AntMarker>>,
    food_query: Query<Entity, With<FoodMarker>>,
    tile_query: Query<Entity, With<TileMarker>>,
    home_query: Query<Entity, With<HomeMarker>>,
) {
    // Clear existing entities
    for entity in ant_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in food_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in tile_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in home_query.iter() {
        commands.entity(entity).despawn();
    }

    // Render tiles
    for (pos, tile) in &game_state.visible_tiles {
        let position = pos.to_vec3();

        if let Some(material) = rendering_assets.tile_materials.get(&tile.tile_type) {
            commands.spawn((
                Mesh3d(rendering_assets.tile_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(position),
                TileMarker {
                    tile_type: tile.tile_type,
                    position: *pos,
                },
            ));
        }
    }

    // Render home tiles
    for home_pos in &game_state.home_tiles {
        let position = home_pos.to_vec3() + Vec3::Y * 0.1;
        let is_main = *home_pos == game_state.main_spot;

        let scale = if is_main { 1.2 } else { 1.0 };

        commands.spawn((
            Mesh3d(rendering_assets.home_mesh.clone()),
            MeshMaterial3d(rendering_assets.home_material.clone()),
            Transform::from_translation(position).with_scale(Vec3::splat(scale)),
            HomeMarker {
                is_main_spot: is_main,
            },
        ));
    }

    // Render my ants
    for (ant_id, ant) in &game_state.my_ants {
        let position = ant.position.to_vec3() + Vec3::Y * 0.5;

        if let Some(material) = rendering_assets.ant_materials.get(&ant.ant_type) {
            let mut transform = Transform::from_translation(position);

            // Scale based on health
            let health_ratio = ant.health as f32 / ant.ant_type.health() as f32;
            transform.scale = Vec3::splat(0.5 + health_ratio * 0.5);

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

    // Render enemy ants
    for (enemy_id, enemy) in &game_state.enemy_ants {
        let position = enemy.position.to_vec3() + Vec3::Y * 0.5;

        let mut transform = Transform::from_translation(position);

        // Scale based on health and make slightly larger to distinguish
        let health_ratio = enemy.health as f32 / enemy.ant_type.health() as f32;
        transform.scale = Vec3::splat(0.6 + health_ratio * 0.4);

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

    // Render food
    for (pos, food) in &game_state.food_on_map {
        let position = pos.to_vec3() + Vec3::Y * 0.3;

        if let Some(material) = rendering_assets.food_materials.get(&food.food_type) {
            let mut transform = Transform::from_translation(position);

            // Scale based on amount
            let scale = 0.5 + (food.amount as f32 / 10.0).min(1.0);
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

pub fn debug_rendering_system(
    mut gizmos: Gizmos,
    menu_state: Res<MenuState>,
    game_state: Res<GameState>,
) {
    if !menu_state.debug_mode || !game_state.connected {
        return;
    }

    // Draw coordinate axes
    gizmos.line(Vec3::ZERO, Vec3::X * 5.0, Color::srgb(1.0, 0.0, 0.0));
    gizmos.line(Vec3::ZERO, Vec3::Y * 5.0, Color::srgb(0.0, 1.0, 0.0));
    gizmos.line(Vec3::ZERO, Vec3::Z * 5.0, Color::srgb(0.0, 0.0, 1.0));

    // Draw hex grid
    for (pos, _tile) in &game_state.visible_tiles {
        let world_pos = pos.to_vec3();

        // Draw hex outline
        let hex_points = get_hex_corners(world_pos);
        for i in 0..6 {
            let start = hex_points[i];
            let end = hex_points[(i + 1) % 6];
            gizmos.line(start, end, Color::srgb(0.5, 0.5, 0.5));
        }
    }

    // Draw ant movement paths
    for ant in game_state.my_ants.values() {
        if !ant.current_move.is_empty() {
            let mut prev_pos = ant.position.to_vec3() + Vec3::Y * 0.8;

            for hex_pos in &ant.current_move {
                let world_pos = hex_pos.to_vec3() + Vec3::Y * 0.8;
                gizmos.line(prev_pos, world_pos, Color::srgb(0.0, 1.0, 1.0));
                prev_pos = world_pos;
            }
        }
    }

    // Draw vision ranges for scouts
    for ant in game_state.my_ants.values() {
        if ant.ant_type == AntType::Scout {
            let center = ant.position.to_vec3() + Vec3::Y * 0.1;
            let radius = ant.ant_type.view_range() as f32 * 0.866; // Approximate hex radius
            gizmos.circle(center, radius, Color::srgb(0.0, 1.0, 0.0));
        }
    }
}

fn get_hex_corners(center: Vec3) -> [Vec3; 6] {
    let size = 0.8;
    let mut corners = [Vec3::ZERO; 6];

    for i in 0..6 {
        let angle = std::f32::consts::PI / 3.0 * i as f32;
        corners[i] = center + Vec3::new(size * angle.cos(), 0.0, size * angle.sin());
    }

    corners
}

pub fn update_camera_focus(
    game_state: Res<GameState>,
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<AntMarker>)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if !game_state.connected {
        return;
    }

    // Focus camera on main spot when F is pressed
    if input.just_pressed(KeyCode::KeyF) {
        let center = game_state.main_spot.to_vec3();

        if let Ok(mut camera_transform) = camera_query.single_mut() {
            camera_transform.translation = center + Vec3::new(0.0, 15.0, 15.0);
            camera_transform.look_at(center, Vec3::Y);
        }
    }
}
