//! Bevy plugin and trait definitions for the prockit framework.
//!
//! This module provides:
//!
//! - [`FrameworkPlugin`]: The main Bevy plugin that sets up the procedural generation systems
//! - Memory size constants ([`KB`], [`MB`], [`GB`]) for convenient configuration

use crate::{generate::GenerateTask, provides::PodProvides, registry::NodeRegistry, Pod};

use super::{ProceduralNode, Space, Viewer};
use bevy::prelude::*;

/// One kilobyte (1024 bytes) for memory configuration.
pub const KB: usize = 1024;

/// One megabyte (1024 KB) for memory configuration.
pub const MB: usize = 1024 * KB;

/// One gigabyte (1024 MB) for memory configuration.
pub const GB: usize = 1024 * MB;

/// The main Bevy plugin for the prockit procedural generation framework.
///
/// `FrameworkPlugin` sets up all the systems and resources needed for procedural
/// generation, including:
///
/// - Memory-aware LOD threshold management
/// - Async generation task scheduling and polling
/// - Component hooks for memory tracking
/// - Node registry for placement-based generation
///
/// # Configuration
///
/// The plugin is configured using a builder pattern with two main methods:
///
/// - [`with_space`](Self::with_space): Register a coordinate space with memory limits
/// - [`with_node`](Self::with_node): Register a procedural node type
///
/// # Example
///
/// ```
/// use prockit_framework::{FrameworkPlugin, RealSpace, ProceduralNode, Subdivide, Provider, Provides, MB};
/// use bevy::prelude::*;
/// use get_size2::GetSize;
///
/// #[derive(Component, Clone, Default, GetSize)]
/// struct TerrainChunk;
///
/// impl ProceduralNode for TerrainChunk {
///     fn provides(&self, _provides: &mut Provides<Self>) {}
///     fn subdivide(&self) -> Option<Subdivide> { None }
///     fn place(_provider: &Provider) -> Option<Self> { Some(Self) }
/// }
///
/// fn main() {
///     App::new()
///         .add_plugins((
///             MinimalPlugins,
///             FrameworkPlugin::new()
///                 .with_space::<RealSpace>(64 * MB, 0.5)
///                 .with_node::<RealSpace, TerrainChunk>(),
///         ));
///         // .run(); // Commented out for doctest
/// }
/// ```
#[derive(Default)]
pub struct FrameworkPlugin {
    registrations: Vec<Box<dyn Fn(&mut App) + Send + Sync>>,
}

impl FrameworkPlugin {
    /// Creates a new, empty framework plugin.
    ///
    /// Use [`with_space`](Self::with_space) and [`with_node`](Self::with_node) to
    /// configure the plugin before adding it to your Bevy app.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a coordinate space with the framework.
    ///
    /// This method sets up:
    /// - The [`Thresholds`](super::Thresholds) resource for memory-aware LOD
    /// - The [`NodeRegistry`](crate::registry::NodeRegistry) resource for node registration
    /// - Required components for [`Viewer`] entities
    /// - Core systems for generation, polling, resampling, and threshold calibration
    ///
    /// # Arguments
    ///
    /// * `memory_limit` - Target memory budget in bytes (use [`KB`], [`MB`], [`GB`] constants)
    /// * `free_percentage` - Target percentage of budget to keep free (0.0-1.0).
    ///   Higher values are more conservative with memory.
    ///
    /// # Example
    ///
    /// ```
    /// use prockit_framework::{FrameworkPlugin, RealSpace, MB};
    ///
    /// let plugin = FrameworkPlugin::new()
    ///     .with_space::<RealSpace>(128 * MB, 0.3);
    /// ```
    pub fn with_space<S: Space>(mut self, memory_limit: usize, free_percentage: f32) -> Self {
        self.registrations.push(Box::new(move |app| {
            app.insert_resource(Thresholds::<S>::new(memory_limit, free_percentage))
                .insert_resource(NodeRegistry::new())
                .register_required_components::<Viewer<S>, S::GlobalTransform>()
                .add_systems(
                    Update,
                    (
                        Thresholds::<S>::resample,
                        Thresholds::<S>::recalibrate,
                        GenerateTask::<S>::create_tasks,
                        GenerateTask::<S>::poll_tasks,
                    ),
                );
        }));
        self
    }

    /// Registers a procedural node type with the framework.
    ///
    /// This method sets up:
    /// - Node registration with the [`NodeRegistry`](crate::registry::NodeRegistry)
    /// - Required `GlobalTransform` component for the space
    /// - Required [`PodProvides`] component for sampler collection
    /// - Component hooks for memory tracking (add/remove callbacks)
    ///
    /// # Type Parameters
    ///
    /// * `S` - The [`Space`] this node's transforms operate in
    /// * `T` - The procedural node type, must implement [`ProceduralNode`] and [`Component`]
    ///
    /// # Example
    ///
    /// ```
    /// use prockit_framework::{FrameworkPlugin, RealSpace, ProceduralNode, Subdivide, Provider, Provides, MB};
    /// use bevy::prelude::*;
    /// use get_size2::GetSize;
    ///
    /// #[derive(Component, Clone, Default, GetSize)]
    /// struct MyNode;
    ///
    /// impl ProceduralNode for MyNode {
    ///     fn provides(&self, _provides: &mut Provides<Self>) {}
    ///     fn subdivide(&self) -> Option<Subdivide> { None }
    ///     fn place(_provider: &Provider) -> Option<Self> { Some(Self) }
    /// }
    ///
    /// let plugin = FrameworkPlugin::new()
    ///     .with_space::<RealSpace>(64 * MB, 0.5)
    ///     .with_node::<RealSpace, MyNode>();
    /// ```
    pub fn with_node<S: Space, T: ProceduralNode>(mut self) -> Self {
        self.registrations.push(Box::new(|app| {
            app.world_mut()
                .resource_mut::<NodeRegistry>()
                .register::<T>();
            app.register_required_components::<Pod<T>, S::GlobalTransform>()
                .add_systems(Startup, Self::register_hooks::<S, T>);
        }));
        self
    }

    /// Startup system that registers component hooks for memory tracking and PodProvides creation.
    ///
    /// This system is run during [`Startup`] for each registered node type. It sets up
    /// `on_add` and `on_remove` hooks that:
    /// - Create a [`PodProvides`] component when a [`Pod<T>`] is added
    /// - Update the [`Thresholds`](super::Thresholds) resource when nodes are spawned or despawned
    fn register_hooks<S: Space, T: ProceduralNode>(world: &mut World) {
        world
            .register_component_hooks::<Pod<T>>()
            .on_add(|mut world, entity, _| {
                let pod = world.entity(entity).get::<Pod<T>>().unwrap().clone();
                let size = pod.node_size();
                let pod_provides = PodProvides::new(pod);
                world.commands().entity(entity).insert(pod_provides);
                world.resource_mut::<Thresholds<S>>().add(size);
            })
            .on_remove(|mut world, entity, _| {
                let size = world.entity(entity).get::<Pod<T>>().unwrap().node_size();
                world.resource_mut::<Thresholds<S>>().sub(size);
            });
    }
}

impl Plugin for FrameworkPlugin {
    /// Builds the framework plugin by executing all registered configuration closures.
    fn build(&self, app: &mut App) {
        for registration in &self.registrations {
            registration(app);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Provider, Provides, RealSpace, Subdivide};
    use get_size2::GetSize;

    #[derive(Component, Clone, Default, GetSize)]
    struct TestNode {
        value: i32,
    }

    impl ProceduralNode for TestNode {
        fn provides(&self, _provides: &mut Provides<Self>) {}

        fn subdivide(&self) -> Option<Subdivide> {
            None
        }

        fn place(_provider: &Provider) -> Option<Self> {
            Some(Self { value: 42 })
        }
    }

    #[derive(Component, Clone, Default, GetSize)]
    struct ProviderNode {
        multiplier: f32,
    }

    impl ProviderNode {
        fn scale(&self, pos: &Vec3) -> Vec3 {
            *pos * self.multiplier
        }
    }

    impl ProceduralNode for ProviderNode {
        fn provides(&self, provides: &mut Provides<Self>) {
            provides.add::<RealSpace, _>("scale", ProviderNode::scale);
        }

        fn subdivide(&self) -> Option<Subdivide> {
            None
        }

        fn place(_provider: &Provider) -> Option<Self> {
            Some(Self { multiplier: 1.0 })
        }
    }

    #[test]
    fn test_kb_constant() {
        assert_eq!(KB, 1024);
    }

    #[test]
    fn test_mb_constant() {
        assert_eq!(MB, 1024 * 1024);
    }

    #[test]
    fn test_gb_constant() {
        assert_eq!(GB, 1024 * 1024 * 1024);
    }

    #[test]
    fn test_framework_plugin_new() {
        let plugin = FrameworkPlugin::new();
        assert!(plugin.registrations.is_empty());
    }

    #[test]
    fn test_framework_plugin_default() {
        let plugin = FrameworkPlugin::default();
        assert!(plugin.registrations.is_empty());
    }

    #[test]
    fn test_framework_plugin_with_space_adds_registration() {
        let plugin = FrameworkPlugin::new().with_space::<RealSpace>(64 * MB, 0.5);
        assert_eq!(plugin.registrations.len(), 1);
    }

    #[test]
    fn test_framework_plugin_with_node_adds_registration() {
        let plugin = FrameworkPlugin::new()
            .with_space::<RealSpace>(64 * MB, 0.5)
            .with_node::<RealSpace, TestNode>();
        assert_eq!(plugin.registrations.len(), 2);
    }

    #[test]
    fn test_framework_plugin_chaining() {
        let plugin = FrameworkPlugin::new()
            .with_space::<RealSpace>(64 * MB, 0.5)
            .with_node::<RealSpace, TestNode>()
            .with_node::<RealSpace, ProviderNode>();
        assert_eq!(plugin.registrations.len(), 3);
    }

    #[test]
    fn test_procedural_node_place() {
        let provider = Provider::root();
        let node = TestNode::place(&provider);
        assert!(node.is_some());
        assert_eq!(node.unwrap().value, 42);
    }

    #[test]
    fn test_procedural_node_subdivide_returns_none() {
        let node = TestNode { value: 0 };
        assert!(node.subdivide().is_none());
    }

    #[test]
    fn test_pod_in_world() {
        let mut world = World::new();
        let provider = Provider::root();
        let pod = Pod::<TestNode>::place(&provider).unwrap();
        let entity = world.spawn((pod, GlobalTransform::IDENTITY)).id();

        let pod = world.entity(entity).get::<Pod<TestNode>>().unwrap();
        assert_eq!(pod.read().value, 42);
    }

    #[test]
    fn test_framework_plugin_build_inserts_thresholds_resource() {
        let mut app = App::new();
        app.add_plugins(FrameworkPlugin::new().with_space::<RealSpace>(64 * MB, 0.5));

        // The resource should be inserted after plugin build
        assert!(app.world().contains_resource::<Thresholds<RealSpace>>());
    }

    #[test]
    fn test_framework_plugin_build_inserts_node_registry() {
        let mut app = App::new();
        app.add_plugins(FrameworkPlugin::new().with_space::<RealSpace>(64 * MB, 0.5));

        // The NodeRegistry should be inserted after plugin build
        assert!(app.world().contains_resource::<NodeRegistry>());
    }

    #[test]
    fn test_procedural_node_clone() {
        let node = TestNode { value: 42 };
        let cloned = node.clone();
        assert_eq!(cloned.value, 42);
    }
}
