use bevy::prelude::*;

// TODO
// make chunks all allocate their memory in the same shared pool

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

impl From<&Octant> for UVec3 {
    fn from(octant: &Octant) -> UVec3 {
        match octant {
            Octant::NxNyNz => UVec3::new(0, 0, 0),
            Octant::NxNyPz => UVec3::new(0, 0, 1),
            Octant::NxPyNz => UVec3::new(0, 1, 0),
            Octant::NxPyPz => UVec3::new(0, 1, 1),
            Octant::PxNyNz => UVec3::new(1, 0, 0),
            Octant::PxNyPz => UVec3::new(1, 0, 1),
            Octant::PxPyNz => UVec3::new(1, 1, 0),
            Octant::PxPyPz => UVec3::new(1, 1, 1),
        }
    }
}

impl From<UVec3> for Octant {
    fn from(position: UVec3) -> Octant {
        match (
            (position.x % 2) != 0,
            (position.y % 2) != 0,
            (position.z % 2) != 0,
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
}

#[derive(Component)]
struct Chunk {
    voxels: Vec<u16>,
    side_length: u8,
}

impl Chunk {
    pub(crate) fn generate(
        side_length: usize,
        chunk_center: Vec3,
        chunk_scale: f32,
        sampler: impl Fn(Vec3) -> u16,
    ) -> Self {
        let scale_unit = chunk_scale / (side_length as f32);
        let chunk_start = chunk_center - Vec3::splat(0.5 * chunk_scale);
        Self {
            voxels: Self::positions(side_length)
                .map(|position| {
                    let voxel_center = position.as_vec3() + Vec3::splat(0.5);
                    sampler(chunk_start + (voxel_center * scale_unit))
                })
                .collect(),
            side_length: side_length as u8,
        }
    }

    pub(crate) fn enhance(
        &self,
        chunk_transform: GlobalTransform,
        sampler: impl Fn(Vec3, u16, Octant) -> u16,
    ) -> Vec<Self> {
        let side_length = self.side_length as usize;
        let half_length = side_length / 2;
        let scale = chunk_transform.scale().max_element();
        assert_eq!(chunk_transform.scale().x, scale);
        assert_eq!(chunk_transform.scale().y, scale);
        assert_eq!(chunk_transform.scale().z, scale);
        let scale_unit = scale / (side_length as f32);
        let chunk_start = chunk_transform.translation() - Vec3::splat(0.5 * scale);
        Octant::ALL
            .iter()
            .map(|octant| {
                let chunk_start = chunk_start + UVec3::from(octant).as_vec3() * 0.5;
                let scale_unit = scale_unit / 2;
                Self {
                    voxels: Self::positions(side_length)
                        .map(|position| {
                            let parent_position = (position / 2) + offset;
                            let parent_index =
                                Self::position_to_index(parent_position, side_length);
                            let parent_voxel = self.voxels[parent_index];
                            let voxel_position = chunk_start + (position.as_vec3() * scale_unit);
                            sampler(voxel_position, parent_voxel, position.into())
                        })
                        .collect(),
                    side_length: self.side_length,
                }
            })
            .collect()
    }

    fn positions(side_length: usize) -> impl Iterator<Item = UVec3> {
        let total_size = side_length * side_length * side_length;
        (0..total_size).map(move |index| Self::index_to_position(index, side_length))
    }

    fn index_to_position(index: usize, side_length: usize) -> UVec3 {
        let side_length = side_length;
        let z = index / side_length / side_length;
        let y = index / side_length % side_length;
        let x = index % side_length;
        UVec3::new(x as u32, y as u32, z as u32)
    }

    fn position_to_index(position: UVec3, side_length: usize) -> usize {
        let mut index = (position.z as usize) * side_length * side_length;
        index += (position.y as usize) * side_length;
        index += position.x as usize;
        index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, rng};

    fn indices_positions(size: usize) {
        for index in 0..(size * size * size) {
            let position = Chunk::index_to_position(index, size);
            assert!(position.x >= 0);
            assert!(position.x < size as u32);
            assert!(position.y >= 0);
            assert!(position.y < size as u32);
            assert!(position.z >= 0);
            assert!(position.z < size as u32);
            let matched_index = Chunk::position_to_index(position, size);
            assert!(matched_index >= 0);
            assert!(matched_index < size * size * size);
            assert_eq!(index, matched_index);
        }
    }

    #[test]
    fn indices_positions_16() {
        indices_positions(16);
    }

    #[test]
    fn indices_positions_to_128() {
        for size in 1..=128 {
            indices_positions(size);
        }
    }

    fn iter_positions(size: usize) {
        let mut checklist = vec![false; size * size * size];
        for position in Chunk::positions(size) {
            checklist[Chunk::position_to_index(position, size)] = true;
        }
        for value in checklist {
            assert!(value);
        }
    }

    #[test]
    fn iter_positions_16() {
        iter_positions(16);
    }

    #[test]
    fn iter_positions_to_128() {
        for size in 1..=128 {
            iter_positions(size);
        }
    }

    fn generate_positions(chunk_center: Vec3, scale: f32, size: usize) {
        let scale_unit = scale / (size as f32);
        let last_position = Vec3::INFINITY;
        Chunk::generate(size, chunk_center, scale, |position| {
            assert!(position.x > chunk_center.x - (scale * 0.5));
            assert!(position.x < chunk_center.x + (scale * 0.5));
            assert_eq!((position.x - chunk_center.x) % scale_unit, 0.5 * scale_unit);
            assert!(position.y > chunk_center.y - (scale * 0.5));
            assert!(position.y < chunk_center.y + (scale * 0.5));
            assert_eq!((position.y - chunk_center.y) % scale_unit, 0.5 * scale_unit);
            assert!(position.z > chunk_center.z - (scale * 0.5));
            assert!(position.z < chunk_center.z + (scale * 0.5));
            assert_eq!((position.z - chunk_center.z) % scale_unit, 0.5 * scale_unit);
            let end_offset = (scale - scale_unit) * 0.5;
            let start_position = chunk_center - Vec3::splat(-end_offset);
            if last_position == Vec3::INFINITY {
                assert_eq!(position, start_position);
            }
            if position.y == last_position.y && position.z == last_position.z {
                assert_eq!(position.x - last_position.x, scale_unit);
            }
            if last_position.y == end_offset {
                assert_eq!(position.x, -end_offset);
            }
            if last_position.z == end_offset {
                assert_eq!(position.y, -end_offset);
            }
            0
        });
    }

    #[test]
    fn random_center_generate_positions() {
        let mut generator = rng();
        for _ in 0..10 {
            let center = Vec3::new(
                generator.random_range(-100.0..100.0),
                generator.random_range(-100.0..100.0),
                generator.random_range(-100.0..100.0),
            );
            generate_positions(center, 16.0, 16);
        }
    }

    #[test]
    fn random_scale_generate_positions() {
        let mut generator = rng();
        for _ in 0..10 {
            let scale = rng().random_range(-100.0..100.0);
            generate_positions(Vec3::ZERO, scale, 16);
        }
    }

    #[test]
    fn generate_positions_16() {
        generate_positions(Vec3::ZERO, 16.0, 16);
    }

    #[test]
    fn generate_positions_to_128() {
        for size in 1..=128 {
            generate_positions(Vec3::ZERO, size as f32, size);
        }
    }

    #[test]
    fn all_random_generate_positions() {
        let mut generator = rng();
        let center = Vec3::new(
            generator.random_range(-100.0..100.0),
            generator.random_range(-100.0..100.0),
            generator.random_range(-100.0..100.0),
        );
        let scale = generator.random_range(-100.0..100.0);
        let size = generator.random_range(1..=128);
        generate_positions(center, scale, size);
    }
}
