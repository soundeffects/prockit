// ▄▀  █ ▀█▀ █▄ ▄█ ▄▀▄ ▄▀▀   █▀▄ ▄▀▀
// ▀▄█ █ █▄▄ █ ▀ █ ▀▄▀ ▄██ ▄ █▀▄ ▄██
//! This module defines systems for visualizing `Skeleton`s and `Bone`s, meant for debug purposes.
//! These systems utilize Bevy's gizmos. The systems are collected into the `SkeletonGizmosPlugin`,
//! which can easily be added to the app for the vizualization functionality. When the
//! `bevy_console` feature is enabled, these visualizations can also be toggled on/off by using the
//! `skeleton_gizmos` command in the console.
use crate::{Bone, Skeleton};
use bevy::{color::palettes::css::RED, prelude::*};
#[cfg(feature = "bevy_console")]
use bevy_console::{AddConsoleCommand, ConsoleCommand};
#[cfg(feature = "bevy_console")]
use clap::Parser;

/// The `SkeletonGizmos` group includes all gizmos rendered by the `SkeletonGizmosPlugin`.
#[derive(Default, Reflect, GizmoConfigGroup)]
struct SkeletonGizmos;

/// This plugin adds systems for visualizing `Skeleton`s and `Bone`s to the app, meant for debug
/// purposes. These systems utilize Bevy's gizmos. When the `bevy_console` feature is enabled, these
/// visualizations can also be toggled on/off by using the `skeleton_gizmos` command in the console.
pub struct SkeletonGizmosPlugin;

impl Plugin for SkeletonGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<SkeletonGizmos>()
            .add_systems(Update, draw_skeletons);

        #[cfg(feature = "bevy_console")]
        app.add_console_command::<VisibilityCommand, _>(toggle_visibility);
    }
}

/// The `draw_skeletons` system iterates through all `Bone` entities, positions them in world space
/// using the angles and lengths defined by each `Bone` and its parent, and then draws a line
/// segment for each `Bone` once positioned.
fn draw_skeletons(
    camera: Query<&Transform, (With<Camera3d>, Without<Skeleton>)>,
    skeletons: Query<(&Transform, &Children), With<Skeleton>>,
    bones: Query<(&Bone, Option<&Children>)>,
    mut gizmos: Gizmos<SkeletonGizmos>,
) {
    let _camera_position = camera.single().translation;

    // Start iteration with the roots (skeletons) and recurse to the leaves
    for (transform, children) in &skeletons {
        // Iterate through all bones by using a stack. Necessary because bones are ordered
        // hierarchically.
        let mut stack = Vec::new();

        // Push all bones that are direct children of the skeleton component
        for child in children {
            stack.push((Vec3::ZERO, (Vec3::X, Vec3::Y), child));
        }

        while !stack.is_empty() {
            // We can unwrap because we know the stack is not empty
            let (parent_position, parent_context, id) = stack.pop().unwrap();
            let (bone, potential_children) = bones.get(id.clone()).unwrap();

            // Position bone in space
            let new_context = bone.derive(parent_context);
            let new_position = parent_position + (new_context.0 * bone.length());

            // Draw a line for the bone, while also transforming by the skeleton's transform
            gizmos.line(
                transform.transform_point(parent_position),
                transform.transform_point(new_position),
                RED,
            );

            // Add children to the stack
            if let Some(children) = potential_children {
                for child in children {
                    stack.push((new_position, new_context, child));
                }
            }
        }
    }
}

/// The `VisibilityCommand` is used to toggle the visibility of skeleton gizmos. It's command name
/// is `skeleton_gizmos` and it takes no arguments.
#[cfg(feature = "bevy_console")]
#[derive(Parser, ConsoleCommand)]
#[command(name = "skeleton_gizmos")]
struct VisibilityCommand;

/// This system toggles the visibility of skeleton gizmos by setting the `enabled` field of the
/// `SkeletonGizmos` group. It is called by `bevy_console` when the `VisibilityCommand` is executed.
#[cfg(feature = "bevy_console")]
fn toggle_visibility(
    mut gizmo_config_store: ResMut<GizmoConfigStore>,
    mut command: ConsoleCommand<VisibilityCommand>,
) {
    if let Some(Ok(VisibilityCommand)) = command.take() {
        let (skeleton_gizmo_config, _) = gizmo_config_store.config_mut::<SkeletonGizmos>();
        skeleton_gizmo_config.enabled ^= true;
        command.ok();
    }
}
