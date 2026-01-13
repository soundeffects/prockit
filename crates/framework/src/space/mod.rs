mod allocations;
mod trade;

use crate::{ProceduralNode, Provider, Provides};
pub use allocations::NodeList;
pub(crate) use allocations::{Allocations, SpawnNode};
use bevy::prelude::*;
use std::marker::PhantomData;
pub(crate) use trade::Trade;

pub trait Space: Send + Sync + 'static {
    type Position;
    type GlobalTransform: Component + Default;
    type LocalTransform: Component + Default;
    type Display;

    fn noticeability(node: &Self::Transform, viewer: &Self::Transform) -> f32;
}

#[derive(Component)]
pub struct Viewer<S: Space> {
    priority: f32,
    space_phantom_data: PhantomData<S>,
}

impl<S: Space> Viewer<S> {
    pub fn new(priority: f32) -> Self {
        Self {
            priority,
            space_phantom_data: PhantomData,
        }
    }

    pub fn priority(&self) -> f32 {
        self.priority
    }
}

pub(crate) struct RegisterSpace<S: Space> {
    space_phantom_data: PhantomData<S>,
}

impl<S: Space> RegisterSpace<S> {
    pub(crate) fn new() -> Self {
        Self {
            space_phantom_data: PhantomData,
        }
    }
}

impl<S: Space> Plugin for RegisterSpace<S> {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, resample::<S>)
            .register_required_components::<Viewer<S>, S::GlobalTransform>();
    }
}

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
