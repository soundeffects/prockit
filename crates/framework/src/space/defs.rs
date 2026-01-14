use bevy::prelude::*;

pub trait Space: Clone + Send + Sync + 'static {
    type Position;
    type GlobalTransform: Component + Clone + Default;
    type LocalTransform: Component + Clone + Default;
    type Display;

    fn noticeability(node: &Self::GlobalTransform, viewer: &Self::GlobalTransform) -> f32;
    fn push_transform(
        parent: &Self::GlobalTransform,
        child: &Self::LocalTransform,
    ) -> Self::GlobalTransform;
}

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

    fn push_transform(parent: &GlobalTransform, child: &Transform) -> GlobalTransform {
        parent.mul_transform(*child)
    }
}
