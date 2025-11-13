mod chunk_index;
mod chunk_position;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use chunk_index::ChunkIndex;
use chunk_position::ChunkPosition;
use std::ops::Neg;

/// Editable static parameter for the size of one dimension of a chunk. Should be a power of two.
const CHUNK_LENGTH: usize = 16;

// The following are derived constants from the `CHUNK_LENGTH` constant above.
/// The number of boundaries between voxels along one dimension of a chunk.
const INNER_LENGTH: usize = CHUNK_LENGTH - 1;
/// The midpoint of a chunk along one dimension.
const HALF_LENGTH: usize = CHUNK_LENGTH / 2;
/// The size, in number of voxels, for a two-dimensional slice of the chunk along any grid-aligned
/// axis.
const PLATE_SIZE: usize = CHUNK_LENGTH * CHUNK_LENGTH;
/// The size, in number of voxels, for a two-dimensional slice of an octant sub-chunk (half the
/// size of a normal chunk) along any grid-aligned axis.
const HALF_PLATE_SIZE: usize = HALF_LENGTH * HALF_LENGTH;
/// The size, in number of voxels, of a chunk.
const CHUNK_SIZE: usize = PLATE_SIZE * CHUNK_LENGTH;

/// Eight octants of a cubic region, representing the 2×2×2 sub-regions. This struct is used to
/// address subdivisions in both chunk bounds and voxel bounds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Octant {
    NxNyNz,
    NxNyPz,
    NxPyNz,
    NxPyPz,
    PxNyNz,
    PxNyPz,
    PxPyNz,
    PxPyPz,
}

impl Octant {
    const ALL: [Octant; 8] = [
        Octant::NxNyNz,
        Octant::NxNyPz,
        Octant::NxPyNz,
        Octant::NxPyPz,
        Octant::PxNyNz,
        Octant::PxNyPz,
        Octant::PxPyNz,
        Octant::PxPyPz,
    ];

    /// Three-dimensional offset of the given octant when considered as a sub-chunk.
    fn unit_offset(&self) -> UVec3 {
        match self {
            Octant::NxNyNz => UVec3::ZERO,
            Octant::NxNyPz => UVec3::new(0, 0, 1),
            Octant::NxPyNz => UVec3::new(0, 1, 0),
            Octant::NxPyPz => UVec3::new(0, 1, 1),
            Octant::PxNyNz => UVec3::new(1, 0, 0),
            Octant::PxNyPz => UVec3::new(1, 0, 1),
            Octant::PxPyNz => UVec3::new(1, 1, 0),
            Octant::PxPyPz => UVec3::ONE
        }
    }

    /// Determine which sub-chunk octant contains the given parent chunk position.
    fn containing(position: ChunkPosition) -> Octant {
        match position.fieldwise(|value| value >= HALF_LENGTH) {
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
}

/// Six directions, for the positive and negative directions along each three-dimensional axis.
#[derive(Clone, Copy)]
enum CubicDirection {
    Nx,
    Px,
    Ny,
    Py,
    Nz,
    Pz,
}

impl CubicDirection {
    /// Returns the basis vector for the axis the `CubicDirection` falls on, as a `ChunkPosition`.
    fn axis_basis(&self) -> IVec3 {
        match self {
            CubicDirection::Nx => IVec3::NEG_X,
            CubicDirection::Px => IVec3::X,
            CubicDirection::Ny => IVec3::NEG_Y,
            CubicDirection::Py => IVec3::Y,
            CubicDirection::Nz => IVec3::NEG_Z,
            CubicDirection::Pz => IVec3::Z
        }
    }
}

impl Neg for CubicDirection {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            CubicDirection::Nx => CubicDirection::Px,
            CubicDirection::Px => CubicDirection::Nx,
            CubicDirection::Ny => CubicDirection::Py,
            CubicDirection::Py => CubicDirection::Ny,
            CubicDirection::Nz => CubicDirection::Pz,
            CubicDirection::Pz => CubicDirection::Nz,
        }
    }
}

/// A square face between two voxels when meshing with the blocky algorithm.
struct Face {
    origin: ChunkIndex,
    direction: CubicDirection,
}

/// A one-dimensional and unquantified normal of a surface. Such a normal may be positive,
/// negative, or zero only.
enum SimpleNormal {
    Positive,
    Negative,
    None,
}

impl SimpleNormal {
    #[inline]
    fn as_face(
        &self,
        origin: ChunkIndex,
        direction: CubicDirection,
    ) -> Option<Face> {
        match self {
            SimpleNormal::Positive => Some(Face {
                origin,
                direction
            }),
            SimpleNormal::Negative => Some(Face {
                origin,
                direction: -direction,
            }),
            SimpleNormal::None => None,
        }
    }
}

/// A chunk holds a 16×16×16 voxel grid as a linear array of fixed size. Each element of data is
/// stored as a `u16`.
#[derive(Clone, Component)]
pub(super) struct Chunk {
    voxels: [u16; CHUNK_SIZE],
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            voxels: [0; CHUNK_SIZE],
        }
    }
}

impl Chunk {
    /// Creates a new chunk with voxel values determined by the sampler. The sampler sees each
    /// three-dimensional position in the chunk and returns a voxel data value, which is stored.
    pub(super) fn generate(sampler: impl Fn(UVec3) -> u16) -> Self {
        let mut chunk = Chunk::default();
        for (index, position) in ChunkPosition::enumerate() {
            chunk.voxels[index.as_usize()] = sampler(position.as_uvec3())
        }
        chunk
    }

    /// The chunk is upsampled by a factor of two, leading to eight new sub-chunks in each octant
    /// of this chunk (acting as parent). The sampler receives the three-dimensional position and
    /// previous data of a voxel in the parent chunk, and an octant of that voxel's bounds which it
    /// must generate a new voxel value for.
    pub(super) fn enhance(&self, sampler: impl Fn(UVec3, u16, Octant) -> u16) -> [Self; 8] {
        let mut chunks = [
            Chunk::default(),
            Chunk::default(),
            Chunk::default(),
            Chunk::default(),
            Chunk::default(),
            Chunk::default(),
            Chunk::default(),
            Chunk::default(),
        ];

        for (index, position) in ChunkPosition::enumerate() {
            let subchunk = Octant::containing(position);
            for subvoxel in Octant::ALL {
                let subvoxel_position = position.subvoxel_in_subchunk(subvoxel);
                chunks[subchunk as usize].voxels[subvoxel_position.as_index().as_usize()] =
                    sampler(position.as_uvec3(), self.voxels[index.as_usize()], subvoxel);
            }
        }
        chunks
    }

    /// With a surface function which determines when two voxels should have a face between them,
    /// we iterate along all axes and collect all existing faces between voxels. This will omit any
    /// faces that fall on the boundary of this chunk and neighboring chunks.
    pub(super) fn interior_faces(&self, surface: impl Fn(u16, u16) -> SimpleNormal) -> Vec<Face> {
        ChunkPosition::enumerate().flat_map(
            |(index, position)| {
                [CubicDirection::Px, CubicDirection::Py, CubicDirection::Pz]
                    .iter()
                    .filter_map(|direction| {
                        if let Some(adjacent_position) = position.increment(*direction) {
                            surface(self.voxels[index.as_usize()], self.voxels[adjacent_position.as_index().as_usize()])
                                .as_face(index, *direction)
                        } else {
                            None
                        }
                    })
            }
        ).collect()
    }

    pub(super) fn equal_subdivision_boundary_faces(
        &self,
        direction: CubicDirection,
        other: &Chunk,
        surface: impl Fn(u16, u16) -> SimpleNormal,
    ) -> Vec<Face> {
        let mut faces = vec![];

        for index in 0..PLATE_SIZE {
            let offset = ChunkPosition::from(ChunkIndex(index));
            let (a, b) = (offset.0, offset.1);
            assert!(offset.2 == 0);

            let (position, other_position) = match direction {
                CubicDirection::Nx => (ChunkPosition(0, a, b), ChunkPosition(INNER_LENGTH, a, b)),
                CubicDirection::Px => (ChunkPosition(INNER_LENGTH, a, b), ChunkPosition(0, a, b)),
                CubicDirection::Ny => (ChunkPosition(a, 0, b), ChunkPosition(a, INNER_LENGTH, b)),
                CubicDirection::Py => (ChunkPosition(a, INNER_LENGTH, b), ChunkPosition(a, 0, b)),
                CubicDirection::Nz => (ChunkPosition(a, b, 0), ChunkPosition(a, b, INNER_LENGTH)),
                CubicDirection::Pz => (ChunkPosition(a, b, INNER_LENGTH), ChunkPosition(a, b, 0))
            }

            let origin = ChunkIndex::from(position);
            surface(self.voxels[origin.0], other.voxels[ChunkIndex::from(other_position).0])
                .try_push_face(&mut faces, origin, positive_direction, negative_direction);
        }

        match direction {
            CubicDirection::Px => {
                for y in 0..CHUNK_LENGTH {
                    for x in 0..CHUNK_LENGTH {
                        let index = ChunkIndex::from(ChunkPosition(INNER_LENGTH, x, y));
                        let adjacent = ChunkIndex::from(ChunkPosition(x, y, 0));
                        assert!(adjacent.0 < PLATE_SIZE);
                        surface(self.voxels[index.0], voxels[adjacent.0]).try_push_face(
                            &mut faces,
                            index,
                            CubicDirection::Px,
                            CubicDirection::Nx,
                        )
                    }
                }
            }

            CubicDirection::Nx => {
                for y in 0..CHUNK_LENGTH {
                    for x in 0..CHUNK_LENGTH {
                        let index = ChunkIndex::from(ChunkPosition(0, x, y));
                        let adjacent = ChunkIndex::from(ChunkPosition(x, y, 0));
                        assert!(adjacent.0 < PLATE_SIZE);
                        surface(self.voxels[index.0], voxels[adjacent.0]).try_push_face(
                            &mut faces,
                            index,
                            CubicDirection::Nx,
                            CubicDirection::Px,
                        )
                    }
                }
            }

            CubicDirection::Py => {
                for y in 0..CHUNK_LENGTH {
                    for x in 0..CHUNK_LENGTH {
                        let index = ChunkIndex::from(ChunkPosition(x, INNER_LENGTH, y));
                        let adjacent = ChunkIndex::from(ChunkPosition(x, y, 0));
                        assert!(adjacent.0 < PLATE_SIZE);
                        surface(self.voxels[index.0], voxels[adjacent.0]).try_push_face(
                            &mut faces,
                            index,
                            CubicDirection::Py,
                            CubicDirection::Ny,
                        )
                    }
                }
            }

            CubicDirection::Ny => {
                for y in 0..CHUNK_LENGTH {
                    for x in 0..CHUNK_LENGTH {
                        let index = ChunkIndex::from(ChunkPosition(x, 0, y));
                        let adjacent = ChunkIndex::from(ChunkPosition(x, y, 0));
                        assert!(adjacent.0 < PLATE_SIZE);
                        surface(self.voxels[index.0], voxels[adjacent.0]).try_push_face(
                            &mut faces,
                            index,
                            CubicDirection::Ny,
                            CubicDirection::Py,
                        )
                    }
                }
            }

            CubicDirection::Pz => {
                for y in 0..CHUNK_LENGTH {
                    for x in 0..CHUNK_LENGTH {
                        let index = ChunkIndex::from(ChunkPosition(x, y, INNER_LENGTH));
                        let adjacent = ChunkIndex::from(ChunkPosition(x, y, 0));
                        assert!(adjacent.0 < PLATE_SIZE);
                        surface(self.voxels[index.0], voxels[adjacent.0]).try_push_face(
                            &mut faces,
                            index,
                            CubicDirection::Pz,
                            CubicDirection::Nz,
                        )
                    }
                }
            }

            CubicDirection::Pz => {
                for y in 0..CHUNK_LENGTH {
                    for x in 0..CHUNK_LENGTH {
                        let index = ChunkIndex::from(ChunkPosition(x, y, 0));
                        let adjacent = ChunkIndex::from(ChunkPosition(x, y, 0));
                        assert!(adjacent.0 < PLATE_SIZE);
                        surface(self.voxels[index.0], voxels[adjacent.0]).try_push_face(
                            &mut faces,
                            index,
                            CubicDirection::Nz,
                            CubicDirection::Pz,
                        )
                    }
                }
            }
        }
        faces
    }

    pub(super) fn higher_subdivision_boundary_faces(
        direction: CubicDirection,
        voxels: [u16; HALF_PLATE_SIZE],
    ) -> Vec<Face> {
        todo!()
    }

    /// Create a bevy `Mesh` from the chunk’s interior voxel faces.
    pub(super) fn interior_mesh(&self, surface: impl Fn(u16, u16) -> SimpleNormal) -> Mesh {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        let mut current_index = 0;
        for face in self.interior_faces(surface) {
            let offsets = [
                Vec3::new(-0.5, -0.5, 0.),
                Vec3::new(-0.5, 0.5, 0.),
                Vec3::new(0.5, -0.5, 0.),
                Vec3::new(0.5, 0.5, 0.),
            ];

            let offsets = match face.direction {
                CubicDirection::Px => offsets.map(|offset| offset.zxy()),
                CubicDirection::Nx => offsets.map(|offset| offset.zxy()),
                CubicDirection::Py => offsets.map(|offset| offset.xzy()),
                CubicDirection::Ny => offsets.map(|offset| offset.xzy()),
                _ => offsets,
            };

            let voxel_center = UVec3::from(ChunkPosition::from(face.origin)).as_vec3();
            for offset in offsets {
                positions.push(voxel_center + face.direction.as_vec3() * 0.5 + offset);
                normals.push(face.direction.as_vec3());
            }

            for offset in [0, 1, 2, 2, 1, 3] {
                indices.push(current_index + offset);
            }
            current_index += 6;
        }
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
    }

    /// Iterator over all voxel indices within a chunk.
    fn iter() -> impl Iterator<Item = ChunkIndex> {
        (0..CHUNK_SIZE).map(|index| ChunkIndex(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
