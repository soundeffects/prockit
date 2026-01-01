use bevy::prelude::UVec3;

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
    pub(super) const ALL: [Octant; 8] = [
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
    pub(super) fn unit_offset(&self) -> UVec3 {
        match self {
            Octant::NxNyNz => UVec3::ZERO,
            Octant::NxNyPz => UVec3::new(0, 0, 1),
            Octant::NxPyNz => UVec3::new(0, 1, 0),
            Octant::NxPyPz => UVec3::new(0, 1, 1),
            Octant::PxNyNz => UVec3::new(1, 0, 0),
            Octant::PxNyPz => UVec3::new(1, 0, 1),
            Octant::PxPyNz => UVec3::new(1, 1, 0),
            Octant::PxPyPz => UVec3::ONE,
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
