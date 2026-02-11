//! This module defines the `ChunkPosition` type in its own file, so that access to its internals
//! is restricted and accessor methods must be used instead. These accessor methods do debug
//! assertion checking which double-checks correctness of assumptions when using a `ChunkPosition`.

use super::{CubicDirection, Octant};
use bevy::prelude::{IVec3, UVec3, Vec3};

/// Conceptually, a chunk stores data of voxels that have three-dimensional positions. This type
/// addresses those positions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Position<const N: usize> {
    x: usize,
    y: usize,
    z: usize,
}

impl<const N: usize> Position<N> {
    pub(super) fn new(x: usize, y: usize, z: usize) -> Self {
        let position = Self { x, y, z };
        assert!(position.in_bounds());
        position
    }

    pub(super) fn x(&self) -> usize {
        self.x
    }

    pub(super) fn y(&self) -> usize {
        self.y
    }

    pub(super) fn z(&self) -> usize {
        self.z
    }

    pub(super) fn fieldwise<T, U>(&self, closure: T) -> (U, U, U)
    where
        T: Fn(usize) -> U,
    {
        (closure(self.x), closure(self.y), closure(self.z))
    }

    pub(super) fn transform(&self, closure: impl Fn(usize) -> usize) -> Self {
        let new_position = Self {
            x: closure(self.x),
            y: closure(self.y),
            z: closure(self.z),
        };
        assert!(new_position.in_bounds());
        new_position
    }

    pub(super) fn in_bounds(&self) -> bool {
        self.x < N && self.y < N && self.z < N
    }

    pub(super) fn as_uvec3(&self) -> UVec3 {
        UVec3::new(self.x as u32, self.y as u32, self.z as u32)
    }

    pub(super) fn as_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }

    pub(super) fn get_octant(&self) -> Octant {
        match self.fieldwise(|value| value >= N / 2) {
            (false, false, false) => Octant::NxNyNz,
            (false, false, true) => Octant::NxNyPz,
            (false, true, false) => Octant::NxPyNz,
            (false, true, true) => Octant::NxPyPz,
            (true, false, false) => Octant::PxNyNz,
            (true, false, true) => Octant::PxNyPz,
            (true, true, false) => Octant::PxPyNz,
            (true, true, true) => Octant::PxPyPz,
        }
    }

    pub(super) fn iter() -> impl Iterator<Item = Self> {
        (0..N).flat_map(|z| (0..N).flat_map(move |y| (0..N).map(move |x| Self { x, y, z })))
    }

    pub(super) fn iter_plate(direction: CubicDirection) -> impl Iterator<Item = Self> {
        (0..N).flat_map(move |b| {
            (0..N).map(move |a| match direction {
                CubicDirection::Nx => Self { x: 0, y: a, z: b },
                CubicDirection::Px => Self {
                    x: N - 1,
                    y: a,
                    z: b,
                },
                CubicDirection::Ny => Self { x: a, y: 0, z: b },
                CubicDirection::Py => Self {
                    x: a,
                    y: N - 1,
                    z: b,
                },
                CubicDirection::Nz => Self { x: a, y: b, z: 0 },
                CubicDirection::Pz => Self {
                    x: a,
                    y: b,
                    z: N - 1,
                },
            })
        })
    }

    pub(super) fn downsample(&self, subchunk_octant: Octant) -> Self {
        let offset = subchunk_octant.unit_offset() * N as u32 / 2;
        let halved = self.transform(|field| field / 2);
        let subchunk_position = Self {
            x: halved.x + offset.x as usize,
            y: halved.y + offset.y as usize,
            z: halved.z + offset.z as usize,
        };
        assert!(subchunk_position.in_bounds());
        subchunk_position
    }

    pub(super) fn upsample(&self, subvoxel_octant: Octant) -> Self {
        let wrapped = self.transform(|field| field * 2 % N);
        let offset = subvoxel_octant.unit_offset();
        Self {
            x: wrapped.x + offset.x as usize,
            y: wrapped.y + offset.y as usize,
            z: wrapped.z + offset.z as usize,
        }
    }

    pub(super) fn adjacent(&self, direction: CubicDirection) -> Option<Self> {
        let position = IVec3::new(self.x as i32, self.y as i32, self.z as i32);
        let new_position = position + direction.as_ivec3();
        if new_position.min_element() >= 0 && new_position.max_element() < N as i32 {
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
    #[test]
    fn the_tests() {
        todo!()
    }
}
