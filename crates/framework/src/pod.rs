use super::{Provider, Subdivide};
use crate::placement::Placement;
use bevy::prelude::*;
use get_size2::GetSize;
use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

/// `ProceduralNode` defines the interface for nodes in the procedurally-generated,
/// level-of-detail hierarchy that can be managed by the prockit framework.
///
/// Procedural nodes are **space-agnostic**: a single node can exist in and provide
/// samplers for multiple spaces simultaneously. The framework handles collecting
/// samplers and placements across all spaces.
///
/// Each procedural node can:
/// - Export sampling functions to its descendants via [`ProceduralNode::provides`]
/// - Offer child placements via [`ProceduralNode::subdivide`]
/// - Accept or reject a placement and generate itself via [`ProceduralNode::place`]
///
/// # Example
/// ```
/// use prockit_framework::{ProceduralNode, Provider, Provides, Subdivide, Placement, RealSpace};
/// use bevy::prelude::*;
/// use get_size2::GetSize;
///
/// #[derive(Component, Clone, Default, GetSize)]
/// struct MyNode {
///     value: f32,
/// }
///
/// impl ProceduralNode for MyNode {
///     fn provides(&self, provides: &mut Provides<Self>) {
///         // Add samplers for RealSpace
///         provides.add::<RealSpace, _>("get_value", MyNode::get_value);
///     }
///
///     fn subdivide(&self) -> Option<Subdivide> {
///         None // Leaf node
///     }
///
///     fn place(provider: &Provider) -> Option<Self> {
///         // Accept all placements, generate with default value
///         Some(Self { value: 42.0 })
///     }
/// }
///
/// impl MyNode {
///     fn get_value(&self, _position: &Vec3) -> f32 {
///         self.value
///     }
/// }
/// ```
pub trait ProceduralNode: Component + Clone + GetSize + Send + Sync + 'static {
    /// Export sampling functions to descendants.
    ///
    /// A node can provide samplers for multiple spaces. The functions are collected
    /// into a [`Provider`] and passed to child nodes during generation.
    ///
    /// # Example
    /// ```
    /// # use prockit_framework::{ProceduralNode, Provides, RealSpace};
    /// # use bevy::prelude::*;
    /// # use get_size2::GetSize;
    /// # #[derive(Component, Clone, Default, GetSize)]
    /// # struct MyNode { value: f32 }
    /// impl ProceduralNode for MyNode {
    ///     fn provides(&self, provides: &mut Provides<Self>) {
    ///         provides.add::<RealSpace, _>("value", MyNode::get_value);
    ///     }
    ///     # fn subdivide(&self) -> Option<Subdivide> { None }
    ///     # fn place(_: &Provider) -> Option<Self> { None }
    /// }
    /// # impl MyNode { fn get_value(&self, _: &Vec3) -> f32 { self.value } }
    /// ```
    fn provides(&self, provides: &mut super::Provides<Self>);

    /// Return child placements for this node.
    ///
    /// Returns `None` if this node has no children (leaf node).
    /// Returns `Some(Subdivide)` containing placements for child nodes.
    ///
    /// The [`Subdivide`] struct holds space-agnostic [`Placement`]s, each of which
    /// can contain data for multiple spaces.
    fn subdivide(&self) -> Option<Subdivide>;

    /// Decide whether to accept the placement and generate if accepted.
    ///
    /// This method combines placement acceptance and node generation into a single step.
    /// The `provider` contains:
    /// - The [`Placement`] being offered (access via `provider.placement()`)
    /// - Sampling functions from ancestor nodes (access via `provider.query()`)
    ///
    /// Returns `Some(node)` if the node accepts the placement (with the node fully generated),
    /// or `None` if the node rejects this placement.
    fn place(provider: &Provider) -> Option<Self>;
}

/// A wrapper around a [`ProceduralNode`] that enables thread-safe access for sampler currying.
///
/// `Pod<T>` stores the node data in an `Arc<RwLock<T>>`, allowing sampling functions
/// to be called from any thread while the node is being used.
#[derive(Component, Default)]
pub struct Pod<T: ProceduralNode> {
    data: Arc<RwLock<T>>,
}

impl<T: ProceduralNode> Pod<T> {
    /// Curry a sampling function with this pod's data for a specific space.
    ///
    /// This is used internally by the [`Provider`] system to bind node data
    /// to sampling functions.
    pub fn curry<S: super::Space, R>(
        &self,
        function: fn(&T, &S::Position) -> R,
        position: &S::Position,
    ) -> R {
        function(self.data.read().unwrap().deref(), position)
    }

    /// Create a new pod by calling `ProceduralNode::place` with the given provider.
    ///
    /// Returns `Some(pod)` if the node accepts the placement, `None` otherwise.
    pub(crate) fn place(provider: &Provider) -> Option<Self> {
        T::place(provider).map(|node| Self {
            data: Arc::new(RwLock::new(node)),
        })
    }

    /// Get the child placements for this node.
    pub(crate) fn subdivide(&self) -> Option<Subdivide> {
        self.data.read().unwrap().subdivide()
    }

    /// Get the memory size of the contained node.
    pub(crate) fn node_size(&self) -> usize {
        self.data.read().unwrap().get_size()
    }

    /// Get a reference to the underlying node.
    pub fn read(&self) -> impl Deref<Target = T> + '_ {
        self.data.read().unwrap()
    }
}

impl<T: ProceduralNode> Clone for Pod<T> {
    fn clone(&self) -> Self {
        Pod {
            data: self.data.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RealSpace;

    #[derive(Component, Clone, Default, GetSize)]
    struct TestNode {
        value: i32,
    }

    impl ProceduralNode for TestNode {
        fn provides(&self, _provides: &mut super::Provides<Self>) {}

        fn subdivide(&self) -> Option<Subdivide> {
            None
        }

        fn place(_provider: &Provider) -> Option<Self> {
            Some(Self { value: 42 })
        }
    }

    #[test]
    fn test_pod_place_accepts() {
        let provider = Provider::root();
        let pod = Pod::<TestNode>::place(&provider);
        assert!(pod.is_some());
        assert_eq!(pod.unwrap().read().value, 42);
    }

    #[test]
    fn test_pod_clone_shares_data() {
        let provider = Provider::root();
        let pod1 = Pod::<TestNode>::place(&provider).unwrap();
        let pod2 = pod1.clone();

        // Both pods should point to the same data
        assert_eq!(pod1.read().value, pod2.read().value);
    }

    #[test]
    fn test_pod_subdivide_returns_none_for_leaf() {
        let provider = Provider::root();
        let pod = Pod::<TestNode>::place(&provider).unwrap();
        assert!(pod.subdivide().is_none());
    }
}
