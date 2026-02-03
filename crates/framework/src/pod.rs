use super::{Provider, Provides, Space, Subdivide};
use bevy::prelude::*;
use get_size2::GetSize;
use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

/// `ProceduralNode` defines the interface for nodes in the procedurally-generated,
/// level-of-detail hierarchy that can be managed by the prockit framework. See the
/// documentation for each member method for more details.
pub trait ProceduralNode: GetSize + Default + Send + Sync + 'static {
    /// Registers functions this node provides to its descendants.
    /// Functions must own their captured data (use `move` closures with cloned data).
    fn provides() -> Provides<Self>;

    fn slots(&self) -> Option<Slots>;

    fn place(provider: &Provider) -> bool;

    /// Generates this node's data using the given transform and ancestral provider.
    fn generate(&mut self, provider: &Provider);

    fn render_sizes(&self) -> ();

    fn render(&self, provider: &Provider) -> ();
}

#[derive(Component, Default)]
pub struct Pod<T: ProceduralNode> {
    data: Arc<RwLock<T>>,
}

impl<T: ProceduralNode> Pod<T> {
    pub fn curry<S: Space, R>(
        &self,
        function: fn(&T, &S::Position) -> R,
        position: &S::Position,
    ) -> R {
        function(self.data.read().unwrap().deref(), position)
    }

    pub(crate) fn generate(provider: &Provider) -> Self {
        let mut generated = T::default();
        generated.generate(provider);
        Self {
            data: Arc::new(RwLock::new(generated)),
        }
    }

    pub(crate) fn subdivide(&self) -> Option<Subdivide> {
        self.data.read().unwrap().subdivide()
    }

    pub(crate) fn node_size(&self) -> usize {
        self.data.read().unwrap().get_size()
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

    #[test]
    fn pod_size() {
        #[derive(Default, GetSize)]
        pub struct VecNode {
            stuff: Vec<&'static str>,
        }

        impl ProceduralNode for VecNode {
            fn provides() -> Provides<Self> {
                Provides::<Self>::new()
            }
            fn subdivide(&self) -> Option<Subdivide> {
                None
            }
            fn generate(&mut self, _provider: &Provider) {}
        }

        let one = Pod {
            data: Arc::new(RwLock::new(VecNode {
                stuff: vec!["this", "is", "the", "first"],
            })),
        };

        let two = Pod {
            data: Arc::new(RwLock::new(VecNode {
                stuff: vec!["i'm", "second"],
            })),
        };

        assert_eq!(one.node_size(), 100);
        assert_eq!(two.node_size(), 100);
    }
}
