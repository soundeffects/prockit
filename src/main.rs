use bevy::prelude::*;

mod voxel_store;
use bevy_console::{AddConsoleCommand, ConsolePlugin};
use voxel_store::{store_info_command, StoreInfoCommand, VoxelStore};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ConsolePlugin))
        .add_systems(Startup, setup)
        .add_console_command::<StoreInfoCommand, _>(store_info_command)
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
        material: materials.add(Color::RED),
        ..default()
    });

    let mut voxel_store = VoxelStore::new();
    voxel_store.generate_disc(256, 32);
    commands.spawn((voxel_store, Name::new("Main World")));
}
