// ██▀ ▀▄▀ ▄▀▄ █▄ ▄█ █▀▄ █   ██▀ ▄   ▄▀▀ █ █▄ ▄█ █▀▄ █   ██▀   █▀▄ ▄▀▀
// █▄▄ █ █ █▀█ █ ▀ █ █▀  █▄▄ █▄▄ ▄   ▄██ █ █ ▀ █ █▀  █▄▄ █▄▄ ▄ █▀▄ ▄██

//! This example demonstrates instantiating and writing to a [`VoxelStore`].

use voxel_store::prelude::*;

fn main() {
    // Create an empty voxel store
    let mut voxel_store = VoxelStore::new();

    // The voxel store is empty, so we write to it. The new values are as
    // defined by the sampler function, and writes happen within the range of
    // -10 to 10 in all three spatial axes.
    voxel_store.write(-10..10, -10..10, -10..10, Sampler);
}
