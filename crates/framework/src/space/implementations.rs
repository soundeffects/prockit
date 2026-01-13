use super::Space;
use bevy::prelude::*;

#[derive(Clone)]
pub struct RealSpace;

impl Space for RealSpace {
    type Position = Vec3;
    type GlobalTransform = GlobalTransform;
    type LocalTransform = Transform;
    type Display = Mesh;

    fn noticeability(node: &GlobalTransform, viewer: &GlobalTransform) -> f32 {
        node.scale().max_element() / viewer.translation().distance_squared(node.translation())
    }
}
