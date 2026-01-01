use super::CubicDirection;
use bevy::prelude::Vec3;

pub(super) struct Face {
    origin: Vec3,
    normal: CubicDirection,
}

impl Face {
    pub(super) fn new(origin: Vec3, normal: CubicDirection) -> Self {
        Self { origin, normal }
    }

    pub(super) fn origin(&self) -> Vec3 {
        self.origin
    }

    pub(super) fn normal(&self) -> CubicDirection {
        self.normal
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_tests() {
        todo!()
    }
}
