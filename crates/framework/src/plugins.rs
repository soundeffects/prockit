use super::{NodeList, Provider, Provides, RegisterSpace, Space};
use bevy::{platform::collections::HashSet, prelude::*};
use bevy_trait_query::RegisterExt;
use std::any::TypeId;

/// `ProceduralNode` defines the interface for nodes in the procedurally-generated,
/// level-of-detail hierarchy that can be managed by the prockit framework. See the
/// documentation for each member method for more details.
#[bevy_trait_query::queryable]
pub trait ProceduralNode<S: Space> {
    /// Registers functions this node provides to its descendants.
    fn provides<'a>(&'a self, instance: &mut Provides<'a, S>);

    /// Subdivides this node into higher-detail children using `child_commands.add_child()`.
    /// Called when the node is close enough to a viewer to warrant more detail.
    fn subdivide(&self) -> Option<NodeList<S>>;

    fn in_bounds(&self, position: S::Position) -> bool;
    fn bound_points(&self, transform: S::GlobalTransform) -> Vec<S::Position>;

    fn display_size(&self);

    fn init() -> Self
    where
        Self: Sized;

    fn generate(&mut self, transform: &S::GlobalTransform, provider: &Provider<'_, S>);
}

#[derive(Default)]
pub struct ProckitFrameworkPlugin {
    spaces: HashSet<TypeId>,
    registrations: Vec<Box<dyn Fn(&mut App) + Send + Sync>>,
}

impl ProckitFrameworkPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<S: Space, Node: ProceduralNode<S> + Component>(mut self) -> Self {
        if !self.spaces.contains(&TypeId::of::<S>()) {
            self.registrations.push(Box::new(|app: &mut App| {
                app.add_plugins(RegisterSpace::<S>::new());
            }));
        }
        self.registrations.push(Box::new(|app: &mut App| {
            app.register_component_as::<dyn ProceduralNode<S>, Node>()
                .register_required_components::<Node, S::GlobalTransform>();
        }));
        self
    }
}

impl Plugin for ProckitFrameworkPlugin {
    fn build(&self, app: &mut App) {
        for registration in &self.registrations {
            registration(app);
        }
    }
}

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;
