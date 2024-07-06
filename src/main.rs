use bevy::{diagnostic::LogDiagnosticsPlugin, prelude::*};
use voxel_store::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            LogDiagnosticsPlugin::default(),
            VoxelStoreDiagnosticsPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(5., 5., 5.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Directional Light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(5., 5., 1.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Central Cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::from_size(Vec3::ONE)),
        material: materials.add(StandardMaterial {
            base_color: Srgba::rgb(1.0, 0.0, 0.0).into(),
            ..default()
        }),
        ..default()
    });

    let mut voxel_store = VoxelStore::new();
    voxel_store.write(-10..10, -10..10, -10..10, Sampler);
    commands.spawn((voxel_store, Name::new("Main World")));
}
