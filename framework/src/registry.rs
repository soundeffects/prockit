use crate::{Pod, ProceduralNode, Provider};
use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use std::{
    any::TypeId,
    sync::Arc,
};

/// A registry of procedural node types that can be tried for placement.
///
/// When a parent node offers placements via `subdivide()`, the framework uses
/// the `NodeRegistry` to try registered node types in random order until one
/// accepts the placement via `ProceduralNode::place()`.
///
/// # Example
/// ```
/// # use prockit_framework::{NodeRegistry, ProceduralNode, Provider, Provides, Subdivide};
/// # use bevy::prelude::*;
/// # use get_size2::GetSize;
/// # #[derive(Component, Clone, Default, GetSize)]
/// # struct MyNode;
/// # impl ProceduralNode for MyNode {
/// #     fn provides(&self, _: &mut Provides<Self>) {}
/// #     fn subdivide(&self) -> Option<Subdivide> { None }
/// #     fn place(_: &Provider) -> Option<Self> { Some(Self) }
/// # }
/// let mut registry = NodeRegistry::new();
/// registry.register::<MyNode>();
/// ```
#[derive(Resource, Clone)]
pub struct NodeRegistry {
    nodes: Arc<Vec<RegisteredNode>>,
}

/// Internal storage for a registered node type.
struct RegisteredNode {
    type_id: TypeId,
    type_name: &'static str,
    /// Attempts placement, returns Some(spawner) if accepted.
    try_place: fn(&Provider) -> Option<Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>>,
}

impl NodeRegistry {
    /// Create an empty node registry.
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(Vec::new()),
        }
    }

    /// Register a procedural node type with this registry.
    ///
    /// Registered node types will be tried in random order when the framework
    /// attempts to fill a placement.
    ///
    /// # Example
    /// ```
    /// # use prockit_framework::{NodeRegistry, ProceduralNode, Provider, Provides, Subdivide};
    /// # use bevy::prelude::*;
    /// # use get_size2::GetSize;
    /// # #[derive(Component, Clone, Default, GetSize)]
    /// # struct MyNode;
    /// # impl ProceduralNode for MyNode {
    /// #     fn provides(&self, _: &mut Provides<Self>) {}
    /// #     fn subdivide(&self) -> Option<Subdivide> { None }
    /// #     fn place(_: &Provider) -> Option<Self> { Some(Self) }
    /// # }
    /// let mut registry = NodeRegistry::new();
    /// registry.register::<MyNode>();
    /// ```
    pub fn register<T: ProceduralNode>(&mut self) {
        Arc::make_mut(&mut self.nodes).push(RegisteredNode {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            try_place: |provider| {
                Pod::<T>::place(provider).map(|pod| {
                    let spawner: Box<dyn FnOnce(&mut EntityCommands) + Send + Sync> =
                        Box::new(move |commands| {
                            commands.insert(pod);
                        });
                    spawner
                })
            },
        });
    }

    /// Check if a node type is registered.
    pub fn is_registered<T: ProceduralNode>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.nodes.iter().any(|n| n.type_id == type_id)
    }

    /// Get the number of registered node types.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Try nodes in random order, return first that accepts (with its spawner).
    ///
    /// This is used by the generation system to attempt placement with registered
    /// node types until one accepts.
    pub fn try_place(
        &self,
        provider: &Provider,
        rng: &mut impl Rng,
    ) -> Option<Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>> {
        let mut indices: Vec<_> = (0..self.nodes.len()).collect();
        indices.shuffle(rng);

        for idx in indices {
            if let Some(spawner) = (self.nodes[idx].try_place)(provider) {
                return Some(spawner);
            }
        }
        None
    }

    /// Get the type names of all registered nodes (for debugging).
    pub fn type_names(&self) -> Vec<&'static str> {
        self.nodes.iter().map(|n| n.type_name).collect()
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Provides, Subdivide};
    use get_size2::GetSize;

    #[derive(Component, Clone, Default, GetSize)]
    struct AcceptingNode;

    impl ProceduralNode for AcceptingNode {
        fn provides(&self, _provides: &mut Provides<Self>) {}

        fn subdivide(&self) -> Option<Subdivide> {
            None
        }

        fn place(_provider: &Provider) -> Option<Self> {
            Some(Self)
        }
    }

    #[derive(Component, Clone, Default, GetSize)]
    struct RejectingNode;

    impl ProceduralNode for RejectingNode {
        fn provides(&self, _provides: &mut Provides<Self>) {}

        fn subdivide(&self) -> Option<Subdivide> {
            None
        }

        fn place(_provider: &Provider) -> Option<Self> {
            None
        }
    }

    #[test]
    fn test_registry_new_is_empty() {
        let registry = NodeRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut registry = NodeRegistry::new();
        registry.register::<AcceptingNode>();

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.is_registered::<AcceptingNode>());
        assert!(!registry.is_registered::<RejectingNode>());
    }

    #[test]
    fn test_registry_try_place_accepts() {
        let mut registry = NodeRegistry::new();
        registry.register::<AcceptingNode>();

        let provider = Provider::root();
        let mut rng = rand::thread_rng();

        let result = registry.try_place(&provider, &mut rng);
        assert!(result.is_some());
    }

    #[test]
    fn test_registry_try_place_rejects() {
        let mut registry = NodeRegistry::new();
        registry.register::<RejectingNode>();

        let provider = Provider::root();
        let mut rng = rand::thread_rng();

        let result = registry.try_place(&provider, &mut rng);
        assert!(result.is_none());
    }

    #[test]
    fn test_registry_try_place_mixed() {
        let mut registry = NodeRegistry::new();
        registry.register::<RejectingNode>();
        registry.register::<AcceptingNode>();

        let provider = Provider::root();
        let mut rng = rand::thread_rng();

        // Should find AcceptingNode eventually (random order)
        let result = registry.try_place(&provider, &mut rng);
        assert!(result.is_some());
    }

    #[test]
    fn test_registry_type_names() {
        let mut registry = NodeRegistry::new();
        registry.register::<AcceptingNode>();
        registry.register::<RejectingNode>();

        let names = registry.type_names();
        assert_eq!(names.len(), 2);
    }
}
