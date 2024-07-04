#[derive(Clone, Copy, Default)]
pub struct Voxel {
    value: u32,
}

impl Voxel {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

impl From<u32> for Voxel {
    fn from(value: u32) -> Self {
        Self { value }
    }
}

impl Into<u32> for Voxel {
    fn into(self) -> u32 {
        self.value
    }
}
