use bevy::prelude::*;
use rasterless::{Ellipsoid, RasterlessRenderPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, RasterlessRenderPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let test = Vec::<u32>::new();

    /*commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10., 10., 10.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn((SpatialBundle::default(), Ellipsoid { radius: 5. }));*/
}
