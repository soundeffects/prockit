//! This module defines the `ChunkIndex` type in its own file, so that access to its internals is
//! restricted and accessor methods must be used instead. These accessor methods do debug assertion
//! checking which double-checks correctness of assumptions when using a `ChunkIndex`.

use super::{CHUNK_LENGTH, CHUNK_SIZE, ChunkPosition, PLATE_SIZE};

/// Internally, a chunk uses a linear array for voxel data. This type addresses the linear position
/// of voxel data.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct ChunkIndex(usize);

impl ChunkIndex {
    /// Creates a new `ChunkIndex` with the given index. Debug assertion to ensure that the index
    /// is within the bounds of the `Chunk` voxel array size.
    #[inline]
    pub(super) fn new(index: usize) -> Self {
        assert!(index < CHUNK_SIZE);
        Self(index)
    }

    /// Iterates over each possible index in a `Chunk`, allowing debug assertions on indices by
    /// default.
    pub(super) fn iter() -> impl Iterator<Item = Self> {
        (0..CHUNK_SIZE).map(|index| Self(index))
    }

    /// Converts this index to a three-dimensional `ChunkPosition`.
    pub(super) fn as_position(&self) -> ChunkPosition {
        ChunkPosition::new(
            self.0 % CHUNK_LENGTH,
            self.0 / CHUNK_LENGTH % CHUNK_LENGTH,
            self.0 / PLATE_SIZE,
        )
    }

    /// Converts this index into a `usize`, which is the primitive that can be used to index into
    /// an array.
    pub(super) fn as_usize(&self) -> usize {
        self.0
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
