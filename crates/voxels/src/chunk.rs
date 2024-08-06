use core::f32;
use std::{cmp::Reverse};
use bevy::{math::FloatOrd, prelude::*};
use priority_queue::PriorityQueue;

/// A `MeshingChunk` holds a view of some storage chunks such that it can create a mesh of those
/// chunks at a given level of detail.
#[derive(Component, Clone, Copy)]
pub struct MeshingChunk;

/// A `ChunkLoader` marks an entity so that the chunk loading system can load chunks around it
/// dynamically as the entity moves through the world.
#[derive(Component)]
pub struct ChunkViewer {
    just_spawned: bool,
    distance: f32
}

impl Default for ChunkViewer {
    fn default() -> Self {
        Self { just_spawned: true, distance: 64.0 }
    }
}

/// The `ChunkAllocation` resource
#[derive(Resource)]
pub struct ChunkAllocation {
    count: u32
}

impl ChunkAllocation {
    fn new(count: u32) -> Self {
        Self { count }
    }
}

impl Default for ChunkAllocation {
    fn default() -> Self {
        Self { count: 1024 }
    }
}

/// The `ChunkQueue` resource
#[derive(Resource)]
pub struct ChunkQueue {
    insertion: PriorityQueue<IVec3, FloatOrd>,
    removal: PriorityQueue<Entity, Reverse<FloatOrd>>,
}

impl ChunkQueue {
    fn new() -> Self {
        Self { insertion: PriorityQueue::new(), removal: PriorityQueue::new() }
    }
}

impl Default for ChunkQueue {
    fn default() -> Self {
        Self { insertion: PriorityQueue::new(), removal: PriorityQueue::new() }
    }
}

/// `allocate_chunks`
fn allocate_chunks(mut commands: Commands, chunk_allocation: Res<ChunkAllocation>, mut chunk_queue: ResMut<ChunkQueue>, chunks: Query<Entity, With<MeshingChunk>>) {
    if chunk_allocation.is_changed() {
        let difference = chunk_allocation.count as i32 - chunks.iter().count() as i32;
        if difference > 0 {
            for _ in 0..difference {
                chunk_queue.removal.push(commands.spawn(MeshingChunk).id(), Reverse(FloatOrd(f32::INFINITY)));
            }
        } else {
            for _ in 0..-difference {
                commands.entity(chunk_queue.removal.pop().unwrap().0).despawn();
            }
        }
    }
}

fn move_chunk(mut commands: Commands, mut chunk_queue: ResMut<ChunkQueue>) {
    if let Some(insertion) = chunk_queue.insertion.peek() {
        if let Some(removal) = chunk_queue.removal.peek() {
            // Clone the priorities, so that we can use them later and allow other methods to borrow
            // the chunk queue
            let insertion_priority = insertion.1.clone();
            let Reverse(removal_priority) = removal.1.clone();
            if removal_priority > insertion_priority {
                // Clone the data we need to make the insertion, so that chunk queue is no longer
                // borrowed
                let insertion_point = insertion.0.clone();
                let removal_entity = removal.0.clone();

                // Now that chunk queue is free, we can mutate it by popping the insertion queue and
                // updating the removal queue
                chunk_queue.insertion.pop();
                chunk_queue.removal.push(removal_entity, Reverse(insertion_priority));
                
                commands.entity(removal_entity).insert(Transform::from_translation(insertion_point.as_vec3() * 32.0).with_scale(Vec3::splat(32.0)));
            }
        }
    }
}

/// `load_chunks`
fn load_chunks(
    mut commands: Commands,
    mut chunk_loaders: Query<(&Transform, &mut ChunkViewer)>,
) {
    for (transform, mut chunk_loader) in chunk_loaders.iter_mut() {
        if chunk_loader.just_spawned {
            let chunk_position = (transform.translation / 32.0).as_ivec3();
            let chunk_queue = vec![chunk_position];

            commands.spawn((Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(32.0))))
            chunk_loader.just_spawned = false;
        }
    }
    todo!()
}

/// The `VoxelPlugin` is the main plugin for this crate. It adds systems to load chunks around chunk
/// loader entities.
pub struct VoxelPlugin {
    loaded_chunks: u32,
}

impl Default for VoxelPlugin {
    fn default() -> Self {
        Self { loaded_chunks: 128 }
    }
}

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkAllocation>().add_systems(Update, allocate_chunks);
    }
}
