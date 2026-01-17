//! Bevy plugin and trait definitions for the prockit framework.
//!
//! This module provides:
//!
//! - [`ProceduralNode`] trait: The interface all procedural node types must implement
//! - [`FrameworkPlugin`]: The main Bevy plugin that sets up the procedural generation systems
//! - Memory size constants ([`KB`], [`MB`], [`GB`]) for convenient configuration

use crate::ProceduralNode;

use super::{GenerateTask, Space, Viewer};
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
/// use prockit_framework::{FrameworkPlugin, RealSpace, ProceduralNode, Subdivisions, Provider, Provides, MB};
/// use bevy::prelude::*;
///
/// #[derive(Component, Clone)]
/// struct TerrainChunk;
///
/// impl ProceduralNode<RealSpace> for TerrainChunk {
///     fn provides(&self, _instance: &mut Provides<RealSpace>) {}
///     fn subdivide(&self) -> Option<Subdivisions<RealSpace>> { None }
///     fn init() -> Self { TerrainChunk }
///     fn generate(&mut self, _transform: &GlobalTransform, _provider: &Provider<RealSpace>) {}
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

    /// Registers a procedural node type with the framework.
    ///
    /// This method sets up:
    /// - Trait query registration for the node type
    /// - Required `GlobalTransform` component
    /// - Component hooks for memory tracking (add/remove callbacks)
    ///
    /// # Type Parameters
    ///
    /// * `S` - The [`Space`] this node type operates in
    /// * `T` - The procedural node type, must implement [`ProceduralNode<S>`] and [`Component`]
    ///
    /// # Example
    ///
    /// ```
    /// use prockit_framework::{FrameworkPlugin, RealSpace, ProceduralNode, Subdivisions, Provider, Provides, MB};
    /// use bevy::prelude::*;
    ///
    /// #[derive(Component, Clone)]
    /// struct MyNode;
    ///
    /// # impl ProceduralNode<RealSpace> for MyNode {
    /// #     fn provides(&self, _: &mut Provides<RealSpace>) {}
    /// #     fn subdivide(&self) -> Option<Subdivisions<RealSpace>> { None }
    /// #     fn init() -> Self { MyNode }
    /// #     fn generate(&mut self, _: &GlobalTransform, _: &Provider<RealSpace>) {}
    /// # }
    ///
    /// let plugin = FrameworkPlugin::new()
    ///     .with_space::<RealSpace>(64 * MB, 0.5)
    ///     .with_node::<RealSpace, MyNode>();
    /// ```
    pub fn with_node<S: Space, T: ProceduralNode<S> + Component>(mut self) -> Self {
        self.registrations.push(Box::new(|app| {
            app.register_component_as::<dyn ProceduralNode<S>, T>()
                .register_required_components::<T, S::GlobalTransform>()
                .add_systems(Startup, Self::register_hooks::<S, T>);
        }));
        self
    }

    /// Startup system that registers component hooks for memory tracking.
    ///
    /// This system is run during [`Startup`] for each registered node type. It sets up
    /// `on_add` and `on_remove` hooks that update the [`Thresholds`](super::Thresholds)
    /// resource when nodes are spawned or despawned.
    fn register_hooks<S: Space, T: ProceduralNode<S> + Component>(world: &mut World) {
        world
            .register_component_hooks::<T>()
            .on_add(|mut world, _| world.resource_mut::<Thresholds<S>>().add(size_of::<T>()))
            .on_remove(|mut world, _| world.resource_mut::<Thresholds<S>>().sub(size_of::<T>()));
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
    use crate::{Provider, Provides, RealSpace, Subdivisions};

    #[derive(Component, Clone)]
    struct TestNode {
        value: i32,
    }

    impl ProceduralNode<RealSpace> for TestNode {
        fn provides(&self, _instance: &mut Provides<RealSpace>) {}
        fn subdivide(&self) -> Option<Subdivisions<RealSpace>> {
            None
        }
        fn init() -> Self {
            TestNode { value: 0 }
        }
        fn generate(&mut self, _transform: &GlobalTransform, _provider: &Provider<RealSpace>) {
            self.value = 42;
        }
    }

    #[derive(Component, Clone)]
    struct ProviderNode {
        multiplier: f32,
    }

    impl ProceduralNode<RealSpace> for ProviderNode {
        fn provides(&self, instance: &mut Provides<RealSpace>) {
            let mult = self.multiplier;
            instance.add("scale", move |pos: &Vec3| *pos * mult);
        }
        fn subdivide(&self) -> Option<Subdivisions<RealSpace>> {
            None
        }
        fn init() -> Self {
            ProviderNode { multiplier: 1.0 }
        }
        fn generate(&mut self, _transform: &GlobalTransform, _provider: &Provider<RealSpace>) {}
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
        let plugin = FrameworkPlugin::new().with_node::<RealSpace, TestNode>();
        assert_eq!(plugin.registrations.len(), 1);
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
    fn test_procedural_node_init() {
        let node = TestNode::init();
        assert_eq!(node.value, 0);
    }

    #[test]
    fn test_procedural_node_generate() {
        let mut node = TestNode::init();
        let transform = GlobalTransform::IDENTITY;
        let provider = Provider::<RealSpace>::empty();
        node.generate(&transform, &provider);
        assert_eq!(node.value, 42);
    }

    #[test]
    fn test_procedural_node_subdivide_returns_none() {
        let node = TestNode { value: 0 };
        assert!(node.subdivide().is_none());
    }

    #[test]
    fn test_procedural_node_provides() {
        let node = ProviderNode { multiplier: 2.0 };
        let mut provides = Provides::<RealSpace>::new();
        node.provides(&mut provides);

        use crate::NameQuery;
        let scale_fn = provides.query::<Vec3>(&NameQuery::exact("scale"));
        assert!(scale_fn.is_some());

        let scale = scale_fn.unwrap();
        assert_eq!(scale(&Vec3::splat(1.0)), Vec3::splat(2.0));
    }

    #[test]
    fn test_procedural_node_component_in_world() {
        let mut world = World::new();
        let entity = world
            .spawn((TestNode { value: 100 }, GlobalTransform::IDENTITY))
            .id();

        let node = world.entity(entity).get::<TestNode>().unwrap();
        assert_eq!(node.value, 100);
    }

    #[test]
    fn test_framework_plugin_build_inserts_thresholds_resource() {
        let mut app = App::new();
        app.add_plugins(FrameworkPlugin::new().with_space::<RealSpace>(64 * MB, 0.5));

        // The resource should be inserted after plugin build
        assert!(app.world().contains_resource::<Thresholds<RealSpace>>());
    }

    #[test]
    fn test_procedural_node_clone() {
        let node = TestNode { value: 42 };
        let cloned = node.clone();
        assert_eq!(cloned.value, 42);
    }
}
