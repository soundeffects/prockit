use std::collections::VecDeque;

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_console::{reply, ConsoleCommand};
use clap::Parser;
use ndshape::{ConstPow2Shape3u32, Shape};

const CHUNK_WIDTH: usize = 32;
const CHUNK_SIZE: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_WIDTH;
const CHUNK_SHAPE: ConstPow2Shape3u32<5, 5, 5> = ConstPow2Shape3u32::<5, 5, 5>;

pub enum StoreChunkLayout {
    EMPTY,
    SPARSE,
    COMPRESSED,
    DENSE,
}

pub struct StoreChunk {
    voxels: Vec<Voxel>,
    layout: StoreChunkLayout,
}

pub struct Voxel {
    id: u8,
}

impl StoreChunk {
    pub fn new() -> Self {
        Self {
            voxels: Vec::<Voxel>::new(),
            layout: StoreChunkLayout::EMPTY,
        }
    }

    pub fn generate(&mut self) {}

    pub fn clear(&mut self) {}
}

#[derive(Component)]
pub struct VoxelStore {
    store: HashMap<IVec3, StoreChunk>,
}

impl VoxelStore {
    pub fn new() -> Self {
        Self {
            store: HashMap::<IVec3, StoreChunk>::new(),
        }
    }

    pub fn generate_disc(&mut self, horizontal_limit: u32, vertical_limit: u32) {
        let mut visited = HashSet::<IVec3>::new();
        let mut queue = VecDeque::<IVec3>::new();
        queue.push_back(IVec3::ZERO);

        let directions = [IVec3::X, IVec3::NEG_X, IVec3::Z, IVec3::NEG_Z];
        let horizontal_limit_squared = horizontal_limit.pow(2);
        let mut count = 0u32;

        while !queue.is_empty() {
            // Unwrap the first element because we know that the queue must not be empty
            let position = queue.pop_front().unwrap();

            // Continues if the visited set already contained position
            if !visited.insert(position) {
                continue;
            }

            for height in -(vertical_limit as i32)..(vertical_limit as i32) {
                let mut new_chunk = StoreChunk::new();
                new_chunk.generate();
                self.store.insert(position + IVec3::Y * height, new_chunk);
            }

            for direction in directions {
                let next_position = position + direction;
                let horizontal_distance = next_position.xz().distance_squared(IVec2::ZERO) as u32;

                if horizontal_distance < horizontal_limit_squared {
                    queue.push_back(next_position);
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "voxelstore")]
pub struct StoreInfoCommand;

pub fn store_info_command(
    mut command: ConsoleCommand<StoreInfoCommand>,
    voxel_stores: Query<(&Name, &VoxelStore)>,
) {
    if command.take().is_some_and(|result| result.is_ok()) {
        for (name, voxel_store) in &voxel_stores {
            reply!(
                command,
                "The voxel store named {} has {} chunks.",
                name,
                voxel_store.len()
            );
        }

        command.ok();
    }
}
