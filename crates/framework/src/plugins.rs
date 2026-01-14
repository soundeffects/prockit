use super::{GenerateTask, Provider, Provides, Space, Subdivisions, Thresholds, Viewer};
use bevy::prelude::*;
use bevy_trait_query::RegisterExt;

/// `ProceduralNode` defines the interface for nodes in the procedurally-generated,
/// level-of-detail hierarchy that can be managed by the prockit framework. See the
/// documentation for each member method for more details.
///
/// Implementations must be `Send + Sync + 'static` to support async generation.
#[bevy_trait_query::queryable]
pub trait ProceduralNode<S: Space>: Send + Sync + 'static {
    /// Registers functions this node provides to its descendants.
    /// Functions must own their captured data (use `move` closures with cloned data).
    fn provides(&self, instance: &mut Provides<S>);

    /// Subdivides this node into higher-detail children.
    /// Called when the node is close enough to a viewer to warrant more detail.
    /// Returns `None` if the node should not subdivide.
    fn subdivide(&self) -> Option<Subdivisions<S>>;

    /// Creates a new uninitialized instance of this node type.
    fn init() -> Self
    where
        Self: Sized;

    /// Generates this node's data using the given transform and ancestral provider.
    fn generate(&mut self, transform: &S::GlobalTransform, provider: &Provider<S>);
}

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;

#[derive(Default)]
pub struct FrameworkPlugin {
    registrations: Vec<Box<dyn Fn(&mut App) + Send + Sync>>,
}

impl FrameworkPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_space<S: Space>(mut self, memory_limit: usize, free_percentage: f32) -> Self {
        self.registrations.push(Box::new(move |app| {
            app.insert_resource(Thresholds::<S>::new(memory_limit, free_percentage))
                .register_required_components::<Viewer<S>, S::GlobalTransform>()
                .add_systems(
                    Update,
                    (
                        Thresholds::<S>::resample,
                        Thresholds::<S>::recalibrate,
                        GenerateTask::<S>::poll_tasks,
                        GenerateTask::<S>::create_tasks,
                    ),
                );
        }));
        self
    }

    pub fn with_node<S: Space, T: ProceduralNode<S> + Component>(mut self) -> Self {
        self.registrations.push(Box::new(|app| {
            app.register_component_as::<dyn ProceduralNode<S>, T>()
                .register_required_components::<T, S::GlobalTransform>()
                .add_systems(Startup, Self::register_hooks::<S, T>);
        }));
        self
    }

    fn register_hooks<S: Space, T: ProceduralNode<S> + Component>(world: &mut World) {
        world
            .register_component_hooks::<T>()
            .on_add(|mut world, _| world.resource_mut::<Thresholds<S>>().add(size_of::<T>()))
            .on_remove(|mut world, _| world.resource_mut::<Thresholds<S>>().sub(size_of::<T>()));
    }
}

impl Plugin for FrameworkPlugin {
    fn build(&self, app: &mut App) {
        for registration in &self.registrations {
            registration(app);
        }
    }
}
