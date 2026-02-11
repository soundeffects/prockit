use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures::check_ready},
};
use bevy_flycam::prelude::*;
use block_mesh::{
    GreedyQuadsBuffer, MergeVoxel, RIGHT_HANDED_Y_UP_CONFIG, VoxelVisibility, greedy_quads,
    ndshape::{ConstShape, ConstShape3u32},
};
use prockit_framework::{
    FrameworkPlugin, GB, PendingGenerate, ProceduralNode, Provider, Provides, RealSpace,
    Subdivision, Subdivisions, Viewer,
};
use rand::prelude::*;

pub const CHUNK_LENGTH: u32 = 16;
pub const PADDED_LENGTH: u32 = CHUNK_LENGTH + 2;
pub const CENTER: Vec3 = Vec3::new(0., 0., 0.);
pub const RADIUS: f32 = 6.0;

type ChunkShape = ConstShape3u32<CHUNK_LENGTH, CHUNK_LENGTH, CHUNK_LENGTH>;
type PaddedShape = ConstShape3u32<PADDED_LENGTH, PADDED_LENGTH, PADDED_LENGTH>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Voxel {
    Empty,
    Full,
}

impl block_mesh::Voxel for Voxel {
    fn get_visibility(&self) -> VoxelVisibility {
        match self {
            Voxel::Empty => VoxelVisibility::Empty,
            Voxel::Full => VoxelVisibility::Opaque,
        }
    }
}

impl MergeVoxel for Voxel {
    type MergeValue = Self;

    fn merge_value(&self) -> Self {
        *self
    }
}

#[derive(Component, Clone)]
pub struct Chunk {
    voxels: [Voxel; ChunkShape::SIZE as usize],
}

impl Chunk {
    fn new() -> Self {
        Self {
            voxels: [Voxel::Empty; ChunkShape::SIZE as usize],
        }
    }

    fn opaque(&self, position: &Vec3) -> bool {
        self.voxels[ChunkShape::linearize(position.round().as_uvec3().to_array()) as usize]
            == Voxel::Full
    }

    fn mesh(&self) -> Mesh {
        // Most of this is functionality unrelated to Prockit.
        // We use the `block-mesh` crate here to build a mesh for a voxel chunk.
        let mut padded = [Voxel::Empty; PaddedShape::SIZE as usize];
        for i in 0..ChunkShape::SIZE {
            let [x, y, z] = ChunkShape::delinearize(i);
            let padded_index = ChunkShape::linearize([x + 1, y + 1, z + 1]);
            padded[padded_index as usize] = self.voxels[i as usize];
        }

        let mut buffer = GreedyQuadsBuffer::new(padded.len());
        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;
        greedy_quads(
            &padded,
            &PaddedShape {},
            [0; 3],
            [PADDED_LENGTH - 1; 3],
            &faces,
            &mut buffer,
        );

        let mut positions = Vec::with_capacity(buffer.quads.num_quads() * 4);
        let mut normals = Vec::with_capacity(buffer.quads.num_quads() * 4);
        let mut indices = Vec::with_capacity(buffer.quads.num_quads() * 6);

        for (group, face) in buffer.quads.groups.into_iter().zip(faces.into_iter()) {
            for quad in group.into_iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                positions.extend_from_slice(&face.quad_mesh_positions(&quad.into(), 1.0));
                normals.extend_from_slice(&face.quad_mesh_normals());
            }
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
    }
}

impl ProceduralNode<RealSpace> for Chunk {
    fn provides(&self, instance: &mut Provides<RealSpace>) {
        // Clone self so the closure owns its data
        let self_clone = self.clone();
        instance.add("opaque", move |position: &Vec3| self_clone.opaque(position));
    }

    fn subdivide(&self) -> Option<Subdivisions<RealSpace>> {
        // Check if all voxels are the same. If so, this chunk does not contain the isosurface,
        // and thus should not be subdivided.
        if self.voxels.iter().all(|voxel| *voxel == self.voxels[0]) {
            return None;
        }

        Some(Subdivisions::from((
            Subdivision::<_, Chunk>::new(Transform::from_xyz(-0.25, -0.25, -0.25)),
            Subdivision::<_, Chunk>::new(Transform::from_xyz(-0.25, -0.25, 0.25)),
            Subdivision::<_, Chunk>::new(Transform::from_xyz(-0.25, 0.25, -0.25)),
            Subdivision::<_, Chunk>::new(Transform::from_xyz(-0.25, 0.25, 0.25)),
            Subdivision::<_, Chunk>::new(Transform::from_xyz(0.25, -0.25, -0.25)),
            Subdivision::<_, Chunk>::new(Transform::from_xyz(0.25, -0.25, 0.25)),
            Subdivision::<_, Chunk>::new(Transform::from_xyz(0.25, 0.25, -0.25)),
            Subdivision::<_, Chunk>::new(Transform::from_xyz(0.25, 0.25, 0.25)),
        )))
    }

    fn init() -> Self {
        Self::new()
    }

    fn generate(&mut self, transform: &GlobalTransform, _provider: &Provider<RealSpace>) {
        println!("new generate");
        let scale = transform.scale().max_element();
        let half_chunk = scale / 2.0;
        let half_voxel = scale / CHUNK_LENGTH as f32 / 2.0;

        let chunk_center = transform.translation();

        for i in 0..ChunkShape::SIZE {
            let [x, y, z] = ChunkShape::delinearize(i);
            let position = Vec3::new(x as f32, y as f32, z as f32)
                + Vec3::splat(half_voxel - half_chunk)
                + chunk_center;
            if position.distance_squared(CENTER) < RADIUS * RADIUS {
                self.voxels[i as usize] = Voxel::Full;
            }
        }
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn((
        Chunk::new(),
        PendingGenerate,
        Transform::from_scale(Vec3::splat(1024.0)),
    ));
    commands.spawn((Camera3d::default(), Viewer::<RealSpace>::new(1.0), FlyCam));
    commands.spawn(Mesh3d(meshes.add(Sphere::new(1.0))));
}

#[derive(Component)]
struct MeshTask {
    task: Task<Mesh>,
}

fn mesh(
    mut commands: Commands,
    old_meshes: Query<Entity, (With<Mesh3d>, With<Children>)>,
    need_mesh: Query<(Entity, &Chunk), (Without<Mesh3d>, Without<Children>)>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    for entity in old_meshes {
        commands.entity(entity).remove::<Mesh3d>();
    }

    for (entity, chunk) in need_mesh {
        println!("new mesh");
        let chunk = chunk.clone();
        commands.entity(entity).insert(MeshTask {
            task: task_pool.spawn(async move { chunk.mesh() }),
        });
    }
}

fn poll(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tasks: Query<(Entity, &mut MeshTask)>,
) {
    let mut rng = rand::rng();
    for (entity, mut mesh_task) in &mut tasks {
        if let Some(mesh) = check_ready(&mut mesh_task.task) {
            commands.entity(entity).remove::<MeshTask>().insert((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::srgb(
                    rng.random::<f32>(),
                    rng.random::<f32>(),
                    rng.random::<f32>(),
                )))),
            ));
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            NoCameraPlayerPlugin,
            FrameworkPlugin::new()
                .with_space::<RealSpace>(1 * GB, 0.2)
                .with_node::<_, Chunk>(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (mesh, poll))
        .run();
}
