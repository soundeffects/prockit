use bytemuck::cast_slice;
use encase::{ShaderType, UniformBuffer};
use glam::{Vec2, Vec3};
use winit::dpi::PhysicalSize;

#[derive(Clone, Copy, ShaderType)]
pub(crate) struct Camera {
    position: Vec3,
    direction: Vec3,
    up: Vec3,
    fov: f32,
    far: f32,
    screen: Vec2,
}

impl Camera {
    pub(crate) fn new(
        position: Vec3,
        direction: Vec3,
        up: Vec3,
        fov: f32,
        far: f32,
        screen: PhysicalSize<u32>,
    ) -> Self {
        Self {
            position,
            direction,
            up,
            fov,
            far,
            screen: Vec2::new(screen.width as f32, screen.height as f32),
        }
    }

    pub(crate) fn to_uniform_data(&self) -> [f32; 13] {
        [
            self.position.x,
            self.position.y,
            self.position.z,
            self.direction.x,
            self.direction.y,
            self.direction.z,
            self.up.x,
            self.up.y,
            self.up.z,
            self.fov,
            self.far,
            self.screen.x,
            self.screen.y,
        ]
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.screen = Vec2::new(width as f32, height as f32);
    }
}
