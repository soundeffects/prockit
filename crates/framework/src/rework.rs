use bevy::prelude::*;

pub trait Space {
    type Position;
}

pub struct Real {}

impl Space for Real {
    type Position = Vec3;
}

pub struct Provides<S: Space> {
    attributes: Vec<Box<dyn Fn(S::Position) -> i32 + Send + Sync>>,
}

#[derive(Component)]
pub struct Viewer<S: Space> {
    position: S::Position,
}

pub trait ProceduralNode<S: Space> {
    fn in_bounds(&self, position: S::Position) -> bool;
    fn bound_points(&self) -> Vec<S::Position>;
    fn subdivide(&self, provider: (), child_commands: ());
}
