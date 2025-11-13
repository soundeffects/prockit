//! # Mesh Generation System
//!
//! Converts voxel chunks into renderable 3D meshes.
//!
//! This module provides:
//! - Asynchronous mesh generation from voxel data
//! - Face-based mesh creation with proper normals
//! - Material assignment and management
//! - Integration with Bevy's rendering system

use crate::volumes::{MeshOnHold, Voch};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};
use rand::{Rng, rng};

/// Resource that manages materials for voxel meshes.
///
/// This stores a collection of materials that can be randomly assigned
/// to different chunks for visual variety.
#[derive(Resource)]
struct VoxelMaterial {
    /// Array of material handles for random assignment
    materials: [Handle<StandardMaterial>; 10],
}

/// Component that tracks an asynchronous mesh generation task.
#[derive(Component)]
struct MeshTask {
    /// The async task that will generate the mesh
    task: Task<Mesh>,
}

/// Creates a mesh from a Voch by generating faces for visible surfaces.
///
/// This function iterates through all visible faces in the chunk and creates
/// quad geometry with proper normals and triangle indices.
///
/// # Arguments
///
/// * `voch` - The chunk to convert to a mesh
///
/// # Returns
///
/// A Bevy mesh ready for rendering
fn create_voch_mesh(voch: &Voch) -> Mesh {
}

/// Creates random materials for voxel meshes during startup.
///
/// This system generates 10 random colored materials that will be
/// assigned to different chunks for visual variety.
fn create_voxel_material(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mut rng = rng();
    let materials: Vec<Handle<StandardMaterial>> = (0..10)
        .map(|_| {
            materials.add(StandardMaterial::from_color(Color::srgb(
                rng.random(),
                rng.random(),
                rng.random(),
            )))
        })
        .collect();
    commands.insert_resource(VoxelMaterial {
        materials: materials.try_into().unwrap(),
    });
}

/// Spawns mesh generation tasks for chunks that need meshing.
///
/// This system finds chunks that have been generated but don't yet have meshes
/// and spawns async tasks to generate the mesh data.
fn compute_meshes(
    mut commands: Commands,
    mesh_vochs: Query<(Entity, &Voch), Without<MeshOnHold>>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    for (entity, voch) in mesh_vochs.iter() {
        let voch = voch.clone();
        let task = task_pool.spawn(async move { create_voch_mesh(&voch) });
        commands.entity(entity).insert(MeshTask { task });
    }
}

/// Polls mesh generation tasks and applies completed meshes to entities.
///
/// This system checks for completed mesh generation tasks and:
/// - Adds the generated mesh to the entity
/// - Assigns a random material
/// - Replaces the full Voch with a reduced version (without padding)
fn poll_mesh_tasks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    voxel_material: Res<VoxelMaterial>,
    mut mesh_tasks: Query<(Entity, &mut MeshTask, &Voch)>,
) {
    for (entity, mut mesh_task, voch) in mesh_tasks.iter_mut() {
        if let Some(mesh) = block_on(future::poll_once(&mut mesh_task.task)) {
            let mesh_handle = meshes.add(mesh);
            let mut rng = rng();
            let voxel_material_handle = voxel_material.materials[rng.random_range(0..10)].clone();
            commands
                .entity(entity)
                .insert((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(voxel_material_handle),
                    voch.shed(1),
                ))
                .remove::<MeshTask>();
        }
    }
}

/// Sets up the mesh generation system in the Bevy app.
///
/// This function adds the material creation system to startup and
/// the mesh generation systems to the update schedule.
pub(crate) fn setup(app: &mut App) {
    app.add_systems(Startup, create_voxel_material)
        .add_systems(Update, (compute_meshes, poll_mesh_tasks));
}
