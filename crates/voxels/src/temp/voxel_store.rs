// █ █ ▄▀▄ ▀▄▀ ██▀ █       ▄▀▀ ▀█▀ ▄▀▄ █▀▄ ██▀   █▀▄ ▄▀▀
// ▀▄▀ ▀▄▀ █ █ █▄▄ █▄▄ ▄▄▄ ▄██  █  ▀▄▀ █▀▄ █▄▄ ▄ █▀▄ ▄██

//! The `voxel_store` module encapsulates the [`VoxelStore`] struct and all
//! associated methods. [`VoxelStore`]s are the primary exported struct of this
//! crate, and act as high-level spatial region which can be written to and read
//! from, as well as including level-of-detail and raymarching functionality.

use crate::{chunk::Chunk, sampler::Sampler};
use std::{collections::HashMap, ops::Range};

// Use bevy's version of glam if present
#[cfg(feature = "bevy")]
use bevy::prelude::IVec3;

// Otherwise default to the most recent glam
#[cfg(not(feature = "bevy"))]
use glam::IVec3;

/// The `VoxelStore` is the primary exported struct of this crate. It can be
/// interacted with as a generic spatial region where voxels can be read from
/// and written to. It also provides utilites to accelerate tasks like
/// transfering data to the GPU, ray marching through the volume, and storing a
/// pointer to a location in the voxel space for later.
///
/// Note that as of yet, much of this functionality is unwritten.
///
/// The data structure is designed as a collection of [`Chunk`]s, each indexed
/// to by a 3D vector representing its location.
pub struct VoxelStore {
    levels: Vec<HashMap<IVec3, Chunk>>,
}

impl VoxelStore {
    /// Creates a new, empty `VoxelStore`.
    pub fn new() -> Self {
        Self { levels: Vec::new() }
    }

    /// Returns the total number of populated [`Chunk`]s present in this
    /// `VoxelStore`. Each chunk contains 4^3 voxel values, and may span
    /// differing region sizes depending on the level-of-detail the chunk
    /// represents.
    pub fn len(&self) -> usize {
        let mut total_len = 0;
        for level in &self.levels {
            total_len += level.len();
        }
        total_len
    }

    /// This function will provide an interface for writing to a `VoxelStore`,
    /// but is currently not implmented. It will take a range to edit, as well
    /// as a [`Sampler`] as a function for setting voxel values within that
    /// range.
    pub fn write(&mut self, _x: Range<i64>, _y: Range<i64>, _z: Range<i64>, _sampler: Sampler) {
        unimplemented!()
    }
}
