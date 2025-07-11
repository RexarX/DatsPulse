use crate::menu::MenuState;
use crate::types::*;
use bevy::prelude::*;

pub fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        GameCamera,
    ));

    // Light
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

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.3))),
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));

    // Player representation
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.1, 0.1))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Player,
    ));

    // Origin marker
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.1))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        DebugMarker,
    ));
}

pub fn update_player_visual(
    game_state: Res<GameState>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<GameCamera>)>,
) {
    if let Ok(mut transform) = player_query.single_mut() {
        transform.translation = game_state.player_position;
    }
}

pub fn debug_rendering_system(mut gizmos: Gizmos, menu_state: Res<MenuState>) {
    if menu_state.debug_mode {
        // Draw coordinate axes (make them longer)
        gizmos.line(Vec3::ZERO, Vec3::X * 10.0, Color::srgb(1.0, 0.0, 0.0));
        gizmos.line(Vec3::ZERO, Vec3::Y * 10.0, Color::srgb(0.0, 1.0, 0.0));
        gizmos.line(Vec3::ZERO, Vec3::Z * 10.0, Color::srgb(0.0, 0.0, 1.0));

        // Draw larger grid
        let grid_size = 50;
        let grid_spacing = 2.0;
        let grid_extent = grid_size as f32 * grid_spacing;

        // Grid lines
        for i in -grid_size..=grid_size {
            let i = i as f32 * grid_spacing;
            // X direction lines
            gizmos.line(
                Vec3::new(-grid_extent, 0.0, i),
                Vec3::new(grid_extent, 0.0, i),
                Color::srgb(0.3, 0.3, 0.3),
            );
            // Z direction lines
            gizmos.line(
                Vec3::new(i, 0.0, -grid_extent),
                Vec3::new(i, 0.0, grid_extent),
                Color::srgb(0.3, 0.3, 0.3),
            );
        }

        // Highlight major grid lines (every 10 units)
        for i in (-grid_size..=grid_size).step_by(5) {
            let i = i as f32 * grid_spacing;
            if i % 10.0 == 0.0 {
                // Major X lines
                gizmos.line(
                    Vec3::new(-grid_extent, 0.0, i),
                    Vec3::new(grid_extent, 0.0, i),
                    Color::srgb(0.6, 0.6, 0.6),
                );
                // Major Z lines
                gizmos.line(
                    Vec3::new(i, 0.0, -grid_extent),
                    Vec3::new(i, 0.0, grid_extent),
                    Color::srgb(0.6, 0.6, 0.6),
                );
            }
        }

        // Draw circles at different distances
        gizmos.circle(Vec3::ZERO, 5.0, Color::srgb(1.0, 1.0, 0.0));
        gizmos.circle(Vec3::ZERO, 10.0, Color::srgb(0.0, 1.0, 1.0));
        gizmos.circle(Vec3::ZERO, 20.0, Color::srgb(1.0, 0.0, 1.0));
    }
}
