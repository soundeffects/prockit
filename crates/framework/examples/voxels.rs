use bevy::prelude::*;
use prockit_framework::{
    ChildCommands, ProceduralNode, ProckitFrameworkPlugin, Provider, Provides, RealSpace,
};

pub const CHUNK_LENGTH: usize = 16;
pub const CENTER: Vec3 = Vec3::new(0., 0., 0.);
pub const RADIUS: f32 = 6.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Voxel {
    Empty,
    Full,
}

#[derive(Component)]
pub struct Chunk {
    voxels: [Voxel; CHUNK_LENGTH * CHUNK_LENGTH * CHUNK_LENGTH],
}

impl Chunk {
    fn generate(chunk_center: Vec3, scale: f32) -> Self {
        let mut chunk = Self {
            voxels: [Voxel::Empty; CHUNK_LENGTH * CHUNK_LENGTH * CHUNK_LENGTH],
        };
        let half_chunk = scale / 2.0;
        let half_voxel = scale / CHUNK_LENGTH as f32 / 2.0;

        for z in 0..16 {
            for y in 0..16 {
                for x in 0..16 {
                    let position = Vec3::new(
                        x as f32 - half_chunk + half_voxel,
                        y as f32 - half_chunk + half_voxel,
                        z as f32 - half_chunk + half_voxel,
                    ) + chunk_center;
                    if position.distance_squared(CENTER) < RADIUS * RADIUS {
                        chunk.voxels[Self::linearize(x, y, z)] = Voxel::Full;
                    }
                }
            }
        }

        chunk
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
    fn provides<'a>(&'a self, instance: &mut Provides<'a, RealSpace>) {
        instance.add("opaque", |position: &Vec3| self.opaque(position));
    }

    fn should_subdivide(&self) -> bool {
        // Check if all voxels are the same. If so, this chunk does not contain the isosurface,
        // and thus should not be subdivided.
        !self.voxels.iter().all(|voxel| *voxel == self.voxels[0])
    }

    fn subdivide(
        &self,
        transform: &GlobalTransform,
        _provider: &Provider<'_, RealSpace>,
        mut child_commands: ChildCommands,
    ) {
        let scale = transform.scale().max_element() / 2.0;
        for offset in [
            Vec3::new(-0.25, -0.25, -0.25),
            Vec3::new(-0.25, -0.25, 0.25),
            Vec3::new(-0.25, 0.25, -0.25),
            Vec3::new(-0.25, 0.25, 0.25),
            Vec3::new(0.25, -0.25, -0.25),
            Vec3::new(0.25, -0.25, 0.25),
            Vec3::new(0.25, 0.25, 0.25),
        ] {
            child_commands.add_child((
                Chunk::generate(transform.translation() + offset * scale, scale),
                Transform::from_translation(offset).with_scale(Vec3::splat(0.5)),
            ));
        }
    }

    fn in_bounds(&self, _position: Vec3) -> bool {
        false
    }

    fn bound_points(&self) -> Vec<Vec3> {
        Vec::new()
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Chunk::generate(Vec3::ZERO, 1024.0),
        Transform::from_scale(Vec3::splat(1024.0)),
    ));
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ProckitFrameworkPlugin::new().with::<_, Chunk>(),
        ))
        .add_systems(Startup, setup)
        .run();
}
