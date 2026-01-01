mod face;
mod mesh;
mod position;
mod rules;
mod simple_normal;
mod storage;
mod voxel_tag;

use super::{cubic_direction::CubicDirection, octant::Octant, voxel::Voxel};
use bevy::prelude::*;
use face::Face;
use position::Position;
pub(super) use rules::Rules;
use storage::Storage;
use voxel_tag::VoxelTag;

#[derive(Clone, Default, Component)]
pub(super) struct Chunk<V: Voxel, const N: usize> {
    storage: Storage<V, N>,
}

impl<V: Voxel, const N: usize> Chunk<V, N> {
    /// Creates a new chunk with voxel values determined by the sampler. The sampler sees each
    /// three-dimensional position in the chunk and returns a voxel data value, which is stored.
    pub(super) fn generate(rules: impl Rules<V, N>) -> Self {
        let mut storage = Storage::default();
        for position in Position::<N>::iter() {
            storage.set(position, rules.generate(position));
        }
        Self { storage }
    }

    /// The chunk is upsampled by a factor of two, leading to eight new sub-chunks in each octant
    /// of this chunk (acting as parent). The sampler receives the three-dimensional position and
    /// previous data of a voxel in the parent chunk, and an octant of that voxel's bounds which it
    /// must generate a new voxel value for.
    pub(super) fn enhance(&self, rules: impl Rules<V, N>) -> [Self; 8] {
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

        for position in Position::iter() {
            for subvoxel in Octant::ALL {
                let subvoxel_position = position.upsample(subvoxel);
                chunks[position.get_octant() as usize].storage.set(
                    subvoxel_position,
                    rules.enhance(self.voxel_tag(position), subvoxel),
                );
            }
        }
        chunks
    }

    /// With a surface function which determines when two voxels should have a face between them,
    /// we iterate along all axes and collect all existing faces between voxels. This will omit any
    /// faces that fall on the boundary of this chunk and neighboring chunks.
    pub(super) fn interior_faces(&self, rules: impl Rules<V, N>) -> Vec<Face> {
        CubicDirection::POSITIVE
            .iter()
            .flat_map(|direction| {
                Position::iter().filter_map(|position| {
                    if let Some(adjacent_position) = position.adjacent(*direction) {
                        rules
                            .surface(self.voxel_tag(position), self.voxel_tag(adjacent_position))
                            .as_face(position.as_vec3(), *direction)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    pub(super) fn boundary_faces(
        &self,
        direction: CubicDirection,
        other: &Chunk<V, N>,
        bordering_subchunk: Option<Octant>,
        rules: impl Rules<V, N>,
    ) -> Vec<Face> {
        Position::iter_plate(direction)
            .zip(Position::iter_plate(-direction))
            .filter_map(|(position, other_position)| {
                let other_position = if let Some(octant) = bordering_subchunk {
                    other_position.downsample(octant)
                } else {
                    other_position
                };
                rules
                    .surface(self.voxel_tag(position), other.voxel_tag(other_position))
                    .as_face(position.as_vec3(), direction)
            })
            .collect()
    }

    pub(super) fn voxel_tag(&self, position: Position<N>) -> VoxelTag<V, N> {
        VoxelTag::new(self.storage.get(position), position)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_tests() {
        todo!()
    }
}
