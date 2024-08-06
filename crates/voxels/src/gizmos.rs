use crate::MeshingChunk;
use bevy::color::palettes::css::RED;
pub use bevy::prelude::*;
#[cfg(feature = "bevy_console")]
use bevy_console::{AddConsoleCommand, ConsoleCommand};
#[cfg(feature = "bevy_console")]
use clap::Parser;

/// The `ChunkGizmos` group includes markers for all loaded meshing chunks.
#[derive(Default, Reflect, GizmoConfigGroup)]
struct ChunkGizmos;

/// The `ChunkLoaderGizmos` group includes markers for all chunk loader entities.
#[derive(Default, Reflect, GizmoConfigGroup)]
struct ChunkLoaderGizmos;

/// The `ChunkGizmosPlugin` adds systems which draw gizmos for debugging Chunks during runtime.
pub struct ChunkGizmosPlugin;

impl Plugin for ChunkGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_chunk_gizmos, draw_chunk_loader_gizmos));

        #[cfg(feature = "bevy_console")]
        app.add_console_command::<ChunkVisibilityCommand, _>(toggle_chunk_visibility)
            .add_console_command::<ChunkLoaderVisibilityCommand, _>(toggle_chunk_loader_visibility);
    }
}

fn draw_chunk_gizmos(
    chunks: Query<&Transform, With<MeshingChunk>>,
    mut chunk_gizmos: Gizmos<ChunkGizmos>,
) {
    for transform in &chunks {
        chunk_gizmos.cuboid(transform.clone(), RED);
    }
}

fn draw_chunk_loader_gizmos(chunk_loaders) {
    todo!()
}

/// The `ChunkVisibilityCommand` is used to toggle the visibility of chunk gizmos. It's command name
/// is `chunk_gizmos` and it takes no arguments.
#[cfg(feature = "bevy_console")]
#[derive(Parser, ConsoleCommand)]
#[command(name = "chunk_gizmos")]
struct ChunkVisibilityCommand;

/// This system toggles the visibility of chunk gizmos by setting the `enabled` field of the
/// `ChunkGizmos` group. It is called by `bevy_console` when the `ChunkVisibilityCommand` is
/// sent.
#[cfg(feature = "bevy_console")]
fn toggle_chunk_visibility(
    mut gizmo_config_store: ResMut<GizmoConfigStore>,
    mut command: ConsoleCommand<ChunkVisibilityCommand>,
) {
    if let Some(Ok(ChunkVisibilityCommand)) = command.take() {
        let (chunk_gizmo_config, _) = gizmo_config_store.config_mut::<ChunkGizmos>();
        chunk_gizmo_config.enabled ^= true;
        command.ok();
    }
}

/// The `ChunkLoaderVisibilityCommand` is used to toggle the visibility of chunk loader gizmos. It's
/// command name is `chunk_loader_gizmos` and it takes no arguments.
#[cfg(feature = "bevy_console")]
#[derive(Parser, ConsoleCommand)]
#[command(name = "chunk_loader_gizmos")]
struct ChunkLoaderVisibilityCommand;

/// This system toggles the visibility of chunk loader gizmos by setting the `enabled` field of the
/// `ChunkLoaderGizmos` group. It is called by `bevy_console` when the
/// `ChunkLoaderVisibilityCommand` is sent.
#[cfg(feature = "bevy_console")]
fn toggle_chunk_loader_visibility(
    mut gizmo_config_store: ResMut<GizmoConfigStore>,
    mut command: ConsoleCommand<ChunkLoaderVisibilityCommand>,
) {
    if let Some(Ok(ChunkLoaderVisibilityCommand)) = command.take() {
        let (chunk_loader_gizmo_config, _) = gizmo_config_store.config_mut::<ChunkLoaderGizmos>();
        chunk_loader_gizmo_config.enabled ^= true;
        command.ok();
    }
}
