use bevy::math::UVec3;
use ndshape::{AbstractShape, ConstPow2Shape3u32};

use crate::voxel::Voxel;

#[derive(Clone)]
pub(crate) struct Chunk {
    mask: u64,
    children: u64,
    values: [Voxel; 64],
}

impl Chunk {
    pub(crate) const SHAPE: ConstPow2Shape3u32<2, 2, 2> = ConstPow2Shape3u32::<2, 2, 2>;

    pub(crate) fn new() -> Self {
        Self {
            mask: 0,
            children: 0,
            values: [Voxel::default(); 64],
        }
    }
}
