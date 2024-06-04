use bevy::prelude::*;
use triless::{Ellipsoid, TrilessRenderPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TrilessRenderPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    /*commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10., 10., 10.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn((SpatialBundle::default(), Ellipsoid { radius: 5. }));*/
}
