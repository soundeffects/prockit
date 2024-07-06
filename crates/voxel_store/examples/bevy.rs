// ██▀ ▀▄▀ ▄▀▄ █▄ ▄█ █▀▄ █   ██▀ ▄   ██▄ ██▀ █ █ ▀▄▀   █▀▄ ▄▀▀
// █▄▄ █ █ █▀█ █ ▀ █ █▀  █▄▄ █▄▄ ▄   █▄█ █▄▄ ▀▄▀  █  ▄ █▀▄ ▄██

//! This example demonstrates a Bevy app running with a [`VoxelStore`] spawned
//! as a component of an entity, and diagnostics plugins enabled to print how
//! many chunks are present in the [`VoxelStore`].

use bevy::{diagnostic::LogDiagnosticsPlugin, prelude::*};
use voxel_store::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // We add plugins which take care of diagnostics below.
            // This plugin is provided by Bevy:
            LogDiagnosticsPlugin::default(),
            // This plugin is provided by the voxel_store crate:
            VoxelStoreDiagnosticsPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // We create a new voxel store
    let mut voxel_store = VoxelStore::new();

    // The voxel store is empty, so we write to it. The new values are as
    // defined by the sampler function, and writes happen within the range of
    // -10 to 10 in all three spatial axes.
    voxel_store.write(-10..10, -10..10, -10..10, Sampler);

    // We spawn a new entity with a name and a voxel store component
    commands.spawn((Name::new("Main Voxel Store"), voxel_store));
}
