//! This module defines the `ChunkPosition` type in its own file, so that access to its internals
//! is restricted and accessor methods must be used instead. These accessor methods do debug
//! assertion checking which double-checks correctness of assumptions when using a `ChunkPosition`.

use crate::chunk::Octant;

use super::{CHUNK_LENGTH, ChunkIndex, CubicDirection, PLATE_SIZE};
use bevy::prelude::{IVec3, UVec3};

/// Conceptually, a chunk stores data of voxels that have three-dimensional positions. This type
/// addresses those positions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct ChunkPosition {
    x: usize,
    y: usize,
    z: usize,
}

impl ChunkPosition {
    /// Creates a new `ChunkPosition` with the given coordinates. Debug assertion to ensure that
    /// coordinates are in the range of the constant chunk size.
    #[inline]
    pub(super) fn new(x: usize, y: usize, z: usize) -> Self {
        let position = Self { x, y, z };
        assert!(position.in_bounds());
        position
    }

    /// Applies a closure to all elements of this vector, returning a tuple of the results.
    pub(super) fn fieldwise<T, U>(&self, closure: U) -> (T, T, T)
    where
        U: Fn(usize) -> T,
    {
        (closure(self.x), closure(self.y), closure(self.z))
    }

    /// Ensures the `ChunkPosition` is within the limits of a constant chunk length along each
    /// dimension
    #[inline]
    pub(super) fn in_bounds(&self) -> bool {
        self.x < CHUNK_LENGTH && self.y < CHUNK_LENGTH && self.z < CHUNK_LENGTH
    }

    /// Converts this three-dimensional position to a linear index which represents the
    /// corresponding voxel data in the linear backing array of the `Chunk`.
    #[inline]
    pub(super) fn as_index(&self) -> ChunkIndex {
        ChunkIndex::new(self.x + self.y * CHUNK_LENGTH + self.z * PLATE_SIZE)
    }

    /// Converts this vector to a `UVec3` by converting each respective field from `usize` to
    /// `u32`.
    #[inline]
    pub(super) fn as_uvec3(&self) -> UVec3 {
        UVec3::new(self.x as u32, self.y as u32, self.z as u32)
    }

    /// Iterates through all positions in a chunk, counting through X, Y, and Z axes (in that
    /// order) with a step size of one along the X direction.
    pub(super) fn iter() -> impl Iterator<Item = Self> {
        (0..CHUNK_LENGTH).flat_map(|z| {
            (0..CHUNK_LENGTH).flat_map(move |y| (0..CHUNK_LENGTH).map(move |x| Self { x, y, z }))
        })
    }

    /// Provides an iterator with `ChunkIndex` and corresponding `ChunkPosition` for every voxel in
    /// the `Chunk`.
    pub(super) fn enumerate() -> impl Iterator<Item = (ChunkIndex, Self)> {
        ChunkIndex::iter().zip(Self::iter())
    }

    /// We compute the position of a subvoxel of our current position (given by the `octant`
    /// parameter) relative to an octant subchunk of this position's chunk.
    pub(super) fn subvoxel_in_subchunk(&self, octant: Octant) -> Self {
        let subchunk_offset = Octant::containing(*self).unit_offset();
        let offset = octant.unit_offset() - subchunk_offset * CHUNK_LENGTH as u32;
        Self {
            x: self.x * 2 + offset.x as usize,
            y: self.y * 2 + offset.y as usize,
            z: self.z * 2 + offset.z as usize,
        }
    }

    /// Bounded addition in the direction provided.
    pub(super) fn increment(&self, direction: CubicDirection) -> Option<Self> {
        let position = IVec3::new(self.x as i32, self.y as i32, self.z as i32);
        let new_position = position + direction.axis_basis();
        if new_position.min_element() >= 0 && new_position.max_element() < CHUNK_LENGTH as i32 {
            Some(Self {
                x: new_position.x as usize,
                y: new_position.y as usize,
                z: new_position.z as usize,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tests() {
        todo!()
    }
}
