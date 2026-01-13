use bevy::prelude::*;
use prockit_framework::{
    NodeList, ProceduralNode, ProckitFrameworkPlugin, Provider, Provides, RealSpace,
};

pub const CHUNK_LENGTH: usize = 16;
pub const CENTER: Vec3 = Vec3::new(0., 0., 0.);
pub const RADIUS: f32 = 6.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Voxel {
    Empty,
    Full,
}

#[derive(Component, Clone)]
pub struct Chunk {
    voxels: [Voxel; CHUNK_LENGTH * CHUNK_LENGTH * CHUNK_LENGTH],
    scale: f32,
}

impl Chunk {
    fn new(scale: f32) -> Self {
        Self {
            voxels: [Voxel::Empty; CHUNK_LENGTH * CHUNK_LENGTH * CHUNK_LENGTH],
            scale,
        }
    }

    fn generate_voxels(&mut self, chunk_center: Vec3) {
        let half_chunk = self.scale / 2.0;
        let half_voxel = self.scale / CHUNK_LENGTH as f32 / 2.0;

        for z in 0..16 {
            for y in 0..16 {
                for x in 0..16 {
                    let position = Vec3::new(
                        x as f32 - half_chunk + half_voxel,
                        y as f32 - half_chunk + half_voxel,
                        z as f32 - half_chunk + half_voxel,
                    ) + chunk_center;
                    if position.distance_squared(CENTER) < RADIUS * RADIUS {
                        self.voxels[Self::linearize(x, y, z)] = Voxel::Full;
                    }
                }
            }
        }
    }

    fn linearize(x: usize, y: usize, z: usize) -> usize {
        z * CHUNK_LENGTH * CHUNK_LENGTH + y * CHUNK_LENGTH + x
    }

    fn opaque(&self, position: &Vec3) -> bool {
        let x = position.x.round() as usize;
        let y = position.y.round() as usize;
        let z = position.z.round() as usize;
        self.voxels[Self::linearize(x, y, z)] == Voxel::Full
    }
}

impl ProceduralNode<RealSpace> for Chunk {
    fn provides(&self, instance: &mut Provides<RealSpace>) {
        // Clone self so the closure owns its data
        let self_clone = self.clone();
        instance.add("opaque", move |position: &Vec3| self_clone.opaque(position));
    }

    fn subdivide(&self) -> Option<NodeList<RealSpace>> {
        // Check if all voxels are the same. If so, this chunk does not contain the isosurface,
        // and thus should not be subdivided.
        if self.voxels.iter().all(|voxel| *voxel == self.voxels[0]) {
            return None;
        }

        let mut list = NodeList::new();
        // Add 8 children at octant offsets
        for offset in [
            Vec3::new(-0.25, -0.25, -0.25),
            Vec3::new(-0.25, -0.25, 0.25),
            Vec3::new(-0.25, 0.25, -0.25),
            Vec3::new(-0.25, 0.25, 0.25),
            Vec3::new(0.25, -0.25, -0.25),
            Vec3::new(0.25, -0.25, 0.25),
            Vec3::new(0.25, 0.25, -0.25),
            Vec3::new(0.25, 0.25, 0.25),
        ] {
            list.add::<Chunk>(Transform::from_translation(offset).with_scale(Vec3::splat(0.5)));
        }
        Some(list)
    }

    fn in_bounds(&self, _position: Vec3) -> bool {
        false
    }

    fn bound_points(&self, _transform: GlobalTransform) -> Vec<Vec3> {
        Vec::new()
    }

    fn display_size(&self) -> f32 {
        self.scale
    }

    fn init() -> Self {
        Self::new(1.0)
    }

    fn generate(&mut self, transform: &GlobalTransform, _provider: &Provider<RealSpace>) {
        self.scale = transform.scale().max_element();
        self.generate_voxels(transform.translation());
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Chunk::new(1024.0),
        Transform::from_scale(Vec3::splat(1024.0)),
    ));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ProckitFrameworkPlugin::new().with::<RealSpace, Chunk>(),
        ))
        .add_systems(Startup, setup)
        .run();
}
