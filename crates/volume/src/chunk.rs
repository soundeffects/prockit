use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use std::ops::{Add, Mul, Sub};

const CHUNK_LENGTH: usize = 16;
const INNER_LENGTH: usize = CHUNK_LENGTH - 1;
const HALF_LENGTH: usize = CHUNK_LENGTH / 2;
const CHUNK_SIZE: usize = CHUNK_LENGTH * CHUNK_LENGTH * CHUNK_LENGTH;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Octant {
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

    fn unit_offset(&self) -> ChunkPosition {
        match self {
            Octant::NxNyNz => ChunkPosition(0, 0, 0),
            Octant::NxNyPz => ChunkPosition(0, 0, 1),
            Octant::NxPyNz => ChunkPosition(0, 1, 0),
            Octant::NxPyPz => ChunkPosition(0, 1, 1),
            Octant::PxNyNz => ChunkPosition(1, 0, 0),
            Octant::PxNyPz => ChunkPosition(1, 0, 1),
            Octant::PxPyNz => ChunkPosition(1, 1, 0),
            Octant::PxPyPz => ChunkPosition(1, 1, 1),
        }
    }

    fn chunk_octant(position: ChunkPosition) -> Octant {
        match (
            position.0 >= HALF_LENGTH,
            position.1 >= HALF_LENGTH,
            position.2 >= HALF_LENGTH,
        ) {
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ChunkIndex(usize);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct ChunkPosition(usize, usize, usize);

impl From<ChunkPosition> for ChunkIndex {
    #[inline]
    fn from(position: ChunkPosition) -> Self {
        assert!(position.0 < CHUNK_LENGTH);
        assert!(position.1 < CHUNK_LENGTH);
        assert!(position.2 < CHUNK_LENGTH);
        ChunkIndex(
            position.0 + position.1 * CHUNK_LENGTH + position.2 * CHUNK_LENGTH * CHUNK_LENGTH,
        )
    }
}

impl From<ChunkIndex> for ChunkPosition {
    #[inline]
    fn from(index: ChunkIndex) -> Self {
        assert!(index.0 < CHUNK_SIZE);
        Self(
            index.0 % CHUNK_LENGTH,
            index.0 / CHUNK_LENGTH % CHUNK_LENGTH,
            index.0 / CHUNK_LENGTH / CHUNK_LENGTH,
        )
    }
}

impl From<ChunkPosition> for UVec3 {
    fn from(position: ChunkPosition) -> Self {
        Self::new(position.0 as u32, position.1 as u32, position.2 as u32)
    }
}

impl Add<Self> for ChunkPosition {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        ChunkPosition(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl Sub<Self> for ChunkPosition {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        ChunkPosition(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl Mul<usize> for ChunkPosition {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self::Output {
        ChunkPosition(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

struct Face {
    origin: ChunkIndex,
    direction: Dir3,
}

enum SimpleNormal {
    Positive,
    Negative,
    None,
}

#[derive(Clone, Component)]
struct Chunk {
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
    pub(crate) fn generate(sampler: impl Fn(UVec3) -> u16) -> Self {
        let mut chunk = Chunk::default();
        for index in Self::iter() {
            chunk.voxels[index.0] = sampler(UVec3::from(ChunkPosition::from(index)));
        }
        chunk
    }

    pub(crate) fn enhance(&self, sampler: impl Fn(UVec3, u16, Octant) -> u16) -> [Self; 8] {
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

        for index in Self::iter() {
            let position = ChunkPosition::from(index);
            let current_octant = Octant::chunk_octant(position);
            let offset = position * 2 - current_octant.unit_offset() * CHUNK_LENGTH;
            for sub_octant in Octant::ALL {
                let sub_voxel_position = offset + sub_octant.unit_offset();
                let sub_voxel_index = ChunkIndex::from(sub_voxel_position);
                chunks[current_octant as usize].voxels[sub_voxel_index.0] =
                    sampler(UVec3::from(position), self.voxels[index.0], sub_octant);
            }
        }
        chunks
    }

    fn interior_faces(&self, surface: impl Fn(u16, u16) -> SimpleNormal) -> Vec<Face> {
        let mut faces = Vec::new();
        for z in 0..CHUNK_LENGTH {
            for y in 0..CHUNK_LENGTH {
                for x in 0..INNER_LENGTH {
                    let current = ChunkIndex::from(ChunkPosition(x, y, z));
                    let adjacent = ChunkIndex::from(ChunkPosition(x + 1, y, z));
                    match surface(self.voxels[current.0], self.voxels[adjacent.0]) {
                        SimpleNormal::Positive => faces.push(Face {
                            origin: current,
                            direction: Dir3::X,
                        }),
                        SimpleNormal::Negative => faces.push(Face {
                            origin: current,
                            direction: Dir3::NEG_X,
                        }),
                        SimpleNormal::None => (),
                    }
                }
            }
        }

        for z in 0..CHUNK_LENGTH {
            for y in 0..INNER_LENGTH {
                for x in 0..CHUNK_LENGTH {
                    let current = ChunkIndex::from(ChunkPosition(x, y, z));
                    let adjacent = ChunkIndex::from(ChunkPosition(x, y + 1, z));
                    match surface(self.voxels[current.0], self.voxels[adjacent.0]) {
                        SimpleNormal::Positive => faces.push(Face {
                            origin: current,
                            direction: Dir3::Y,
                        }),
                        SimpleNormal::Negative => faces.push(Face {
                            origin: current,
                            direction: Dir3::NEG_Y,
                        }),
                        SimpleNormal::None => (),
                    }
                }
            }
        }

        for z in 0..INNER_LENGTH {
            for y in 0..CHUNK_LENGTH {
                for x in 0..CHUNK_LENGTH {
                    let current = ChunkIndex::from(ChunkPosition(x, y, z));
                    let adjacent = ChunkIndex::from(ChunkPosition(x, y, z + 1));
                    match surface(self.voxels[current.0], self.voxels[adjacent.0]) {
                        SimpleNormal::Positive => faces.push(Face {
                            origin: current,
                            direction: Dir3::Z,
                        }),
                        SimpleNormal::Negative => faces.push(Face {
                            origin: current,
                            direction: Dir3::NEG_Z,
                        }),
                        SimpleNormal::None => (),
                    }
                }
            }
        }
        faces
    }

    pub(crate) fn interior_mesh(&self, surface: impl Fn(u16, u16) -> SimpleNormal) -> Mesh {
        let positions = Vec::new();
        let normals = Vec::new();
        let indices = Vec::new();
        for face in self.interior_faces(surface) {
            positions.add()
        }
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
    }

    fn iter() -> impl Iterator<Item = ChunkIndex> {
        (0..CHUNK_SIZE).map(|index| ChunkIndex(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_indices() -> impl Iterator<Item = usize> {
        0..CHUNK_SIZE
    }

    fn valid_positions() -> impl Iterator<Item = UVec3> {
        (0..CHUNK_LENGTH).flat_map(|z| {
            (0..CHUNK_LENGTH).flat_map(move |y| {
                (0..CHUNK_LENGTH).map(move |x| UVec3::new(x as u32, y as u32, z as u32))
            })
        })
    }

    #[test]
    fn check_chunk_octants() {
        for octant in Octant::ALL {
            let position = octant.unit_offset() * CHUNK_LENGTH as u32 * 2;
            assert_eq!(Octant::chunk_octant(position), octant);
        }
    }

    #[test]
    fn check_all_valid_chunk_indices() {
        for (index, position) in valid_indices().zip(valid_positions()) {
            let chunk_index = ChunkIndex(index);
            let computed_position = UVec3::from(chunk_index);
            assert_eq!(computed_position, position);
            let computed_chunk_index = ChunkIndex::from_position(computed_position);
            assert_eq!(computed_chunk_index, chunk_index);
        }
    }

    #[test]
    #[should_panic(expected = "Invalid chunk positions should trigger debug asserts")]
    fn check_invalid_chunk_index_x() {
        let position = UVec3::new(CHUNK_LENGTH as u32, 0, 0);
        ChunkIndex::from_position(position);
    }

    #[test]
    #[should_panic(expected = "Invalid chunk positions should trigger debug asserts")]
    fn check_invalid_chunk_index_y() {
        let position = UVec3::new(0, CHUNK_LENGTH as u32, 0);
        ChunkIndex::from_position(position);
    }

    #[test]
    #[should_panic(expected = "Invalid chunk positions should trigger debug asserts")]
    fn check_invalid_chunk_index_z() {
        let position = UVec3::new(0, 0, CHUNK_LENGTH as u32);
        ChunkIndex::from_position(position);
    }

    #[test]
    fn check_generate() {
        let chunk = Chunk::generate(|chunk_index| {
            let position = UVec3::from(chunk_index);
            let recomputed_chunk_index = ChunkIndex::from_position(position);
            recomputed_chunk_index.0 as u16
        });
        for index in valid_indices() {
            assert_eq!(chunk.voxels[index], index as u16)
        }
    }

    #[test]
    fn check_enhance_octant_order() {
        let chunk = Chunk::generate(|chunk_index| chunk_index.0 as u16);
        let chunks = chunk.enhance(|chunk_index, voxel_value, sub_octant| {
            assert_eq!(chunk_index.0, voxel_value as usize);
            sub_octant as u16
        });
        for child in chunks {
            for index in 0..child.voxels.len() {
                let position = UVec3::from(ChunkIndex(index));
                let expected_octant = match (
                    position.x % 2 == 0,
                    position.y % 2 == 0,
                    position.z % 2 == 0,
                ) {
                    (false, false, false) => Octant::NxNyNz,
                    (false, false, true) => Octant::NxNyPz,
                    (false, true, false) => Octant::NxPyNz,
                    (false, true, true) => Octant::NxPyPz,
                    (true, false, false) => Octant::PxNyNz,
                    (true, false, true) => Octant::PxNyPz,
                    (true, true, false) => Octant::PxPyNz,
                    (true, true, true) => Octant::PxPyPz,
                };
                assert_eq!(expected_octant as u16, child.voxels[index])
            }
        }
    }

    #[test]
    fn check_enhance_index_order() {
        let chunk = Chunk::generate(|chunk_index| chunk_index.0 as u16);
        let chunks = chunk.enhance(|chunk_index, voxel_value, _sub_octant| {
            assert_eq!(chunk_index.0, voxel_value as usize);
            chunk_index.0 as u16
        });
        for octant in Octant::ALL {
            let child = &chunks[octant as usize];
            for index in 0..child.voxels.len() {
                let position = (UVec3::from(ChunkIndex(index))
                    + octant.unit_offset() * CHUNK_LENGTH as u32)
                    / 2;
                let chunk_index = ChunkIndex::from_position(position);
                assert_eq!(chunk_index.0 as u16, child.voxels[index]);
            }
        }
    }
}
