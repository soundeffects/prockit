// ██▄ ▄▀▄ █▄ █ ██▀     █   ▄▀▄ █▀▄   █▀▄ ▄▀▀
// █▄█ ▀▄▀ █ ▀█ █▄▄ ▄▄▄ █▄▄ ▀▄▀ █▄▀ ▄ █▀▄ ▄██
// This example demonstrates how to spawn a skeleton using one of the provided generators from the
// crate, how to draw skeletons using the gizmo debug view, and it also illustrates how the LOD
// system for skeletons works by allowing the user to inspect at different zoom/LOD levels.

use bevy::{color::palettes::css::GRAY, prelude::*};
use prockit_skeletons::{stick_figure, Skeleton, SkeletonGizmosPlugin, SkeletonPlugin};
use std::f32::consts::PI;

fn main() {
    App::new()
        .init_gizmo_group::<WorldGizmos>()
        // The SkeletonPlugin allows us to construct skeletons using SkeletonDescriptors, and the
        // SkeletonGizmosPlugin will draw skeletons using gizmos automatically.
        .add_plugins((DefaultPlugins, SkeletonPlugin, SkeletonGizmosPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (draw_gizmos, move_camera))
        .run();
}

// WorldGizmos only contains the grid gizmo
#[derive(Default, Reflect, GizmoConfigGroup)]
struct WorldGizmos;

fn setup(mut commands: Commands) {
    let skeleton_center = Vec3::new(0.0, 1.0, 0.0);

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 2.0, 6.0).looking_at(skeleton_center, Vec3::Y),
        ..default()
    });

    // Skeleton (stick_figure creates a SkeletonDescriptor which will then be transformed into a
    // skeleton bone structure by the SkeletonPlugin)
    commands.spawn((Transform::from_translation(skeleton_center), stick_figure()));

    // Instruction text
    commands.spawn(TextBundle {
        text: Text::from_section(
            "Use the up arrow to zoom in, the down arrow to zoom out.",
            TextStyle::default(),
        ),
        style: Style {
            margin: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        ..default()
    });
}

// Draw the grid gizmo
fn draw_gizmos(mut gizmos: Gizmos<WorldGizmos>) {
    gizmos.grid(
        Vec3::ZERO,
        Quat::from_rotation_x(PI / 2.0),
        UVec2::splat(10),
        Vec2::splat(1.0),
        GRAY,
    );
}

// Rotate the camera around the central skeleton, and also allow the user to zoom in and out by
// pressing the up and down arrow keys
fn move_camera(
    mut camera: Query<&mut Transform, With<Camera3d>>,
    skeleton: Query<&Transform, (With<Skeleton>, Without<Camera3d>)>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if !skeleton.is_empty() {
        // Orbit
        let mut transform = camera.single_mut();
        let skeleton_transform = skeleton.single();
        transform.rotate_around(
            skeleton_transform.translation,
            Quat::from_rotation_y(time.delta_seconds() / 2.0),
        );

        // Zoom
        if keyboard.pressed(KeyCode::ArrowUp) {
            let difference = transform.translation - skeleton_transform.translation;
            let direction = difference.normalize();
            let magnitude = difference.length();
            transform.translation -= direction * magnitude * time.delta_seconds();
        }
        if keyboard.pressed(KeyCode::ArrowDown) {
            let difference = transform.translation - skeleton_transform.translation;
            let direction = difference.normalize();
            let magnitude = difference.length();
            transform.translation += direction * magnitude * time.delta_seconds();
        }
    }
}
