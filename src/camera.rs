use encase::{ShaderType, UniformBuffer};
use glam::Vec3;

#[derive(Clone, Copy, ShaderType)]
pub(crate) struct Camera {
    position: glam::Vec3,
    direction: Vec3,
    up: Vec3,
    fov: f32,
    far: f32,
    screen_width: u32,
    screen_height: u32,
}

impl Camera {
    pub(crate) fn new(
        position: Vec3,
        direction: Vec3,
        up: Vec3,
        fov: f32,
        far: f32,
        screen_width: u32,
        screen_height: u32,
    ) -> Self {
        Self {
            position,
            direction,
            up,
            fov,
            far,
            screen_width,
            screen_height,
        }
    }

    pub(crate) fn to_uniform_data(&self) -> Vec<u8> {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(self).unwrap();
        buffer.into_inner()
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;
    }
}
