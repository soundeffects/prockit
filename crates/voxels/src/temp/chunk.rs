// ▄▀▀ █▄█ █ █ █▄ █ █▄▀   █▀▄ ▄▀▀
// ▀▄▄ █ █ ▀▄█ █ ▀█ █ █ ▄ █▀▄ ▄██

//! The `chunk` module encapsulates the [`Chunk`] data type and all associated
//! methods. [`Chunk`]s are groupings of [`Voxel`]s, and they are the primary
//! data element stored in a [`VoxelStore`]. They provide sparse hierarchical
//! storage of voxel data and raymarching acceleration.

// Until functionality gets filled out in this file, we are allowing unused
// variables
#![allow(dead_code)]

use crate::voxel::Voxel;
use ndshape::ConstPow2Shape3u32;

/// The `Chunk` struct is built as a compact storage element for a grid of 4^3
/// voxels. Several `Chunk`s make up the space of voxel store. A `Chunk` may
/// have children, such that a space that would normally be occupied by a voxel
/// may be subdivided and occupied by another `Chunk`.
///
/// Users of this crate should not interact with the `Chunk` type directly--this
/// struct should only be for internal use.
///
/// A chunk is limited to 64 (4^3) voxels because this allows for a `u64` type
/// to act as a bitmask for those values which are considered active or children
/// in the chunk. With a single memory lookup and bitwise or operations, a ray
/// marching through this chunk can isolate only those solid voxels that it may
/// collide with, or could determine whether it needs to step into a child
/// chunk.
///
/// Chunks do not own their children, they simply store a boolean telling
/// whether a child exists. The application must query the voxel store again for
/// the child chunk, at the position and level of detail expected given the
/// index of the child indicator.
#[derive(Clone, Copy)]
pub(crate) struct Chunk {
    active: u64,
    values: [Voxel; 64],
}

impl Chunk {
    // This shape type lets us linearize and delinearize indices for the chunk.
    pub(crate) const SHAPE: ConstPow2Shape3u32<2, 2, 2> = ConstPow2Shape3u32::<2, 2, 2>;

    pub(crate) fn new() -> Self {
        Self {
            active: 0,
            values: [Voxel::default(); 64],
        }
    }
}
