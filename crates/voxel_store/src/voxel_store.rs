use crate::{chunk::Chunk, sampler::Sampler, voxel::Voxel};
use bevy::{prelude::*, utils::HashMap};
use std::{
    array::from_fn,
    ops::{Range, RangeBounds},
};

#[derive(Component)]
pub struct VoxelStore {
    levels: [HashMap<IVec3, Chunk>; 16],
}

impl VoxelStore {
    pub fn new() -> Self {
        Self {
            levels: from_fn(|_| HashMap::new()),
        }
    }

    pub fn len(&self) -> usize {
        let mut total_len = 0;
        for level in &self.levels {
            total_len += level.len();
        }
        total_len
    }

    pub fn write(&self, x: Range<i64>, y: Range<i64>, z: Range<i64>, sampler: Sampler) {}
}
