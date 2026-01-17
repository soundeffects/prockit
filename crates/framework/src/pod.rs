use super::{Provider, Provides, Space, Subdivide};
use bevy::prelude::*;
use deepsize::DeepSizeOf;
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock},
};

/// `ProceduralNode` defines the interface for nodes in the procedurally-generated,
/// level-of-detail hierarchy that can be managed by the prockit framework. See the
/// documentation for each member method for more details.
pub trait ProceduralNode: DeepSizeOf + Default + Send + Sync + 'static {
    /// Registers functions this node provides to its descendants.
    /// Functions must own their captured data (use `move` closures with cloned data).
    fn provides(interface: Provides<'_, Self>);

    /// Subdivides this node into higher-detail children.
    /// Called when the node is close enough to a viewer to warrant more detail.
    /// Returns `None` if the node should not subdivide.
    fn subdivide(&self) -> Option<Subdivide>;

    /// Generates this node's data using the given transform and ancestral provider.
    fn generate(&mut self, provider: &Provider);
}

#[derive(Component, DeepSizeOf, Default)]
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
}

impl<T: ProceduralNode> Clone for Pod<T> {
    fn clone(&self) -> Self {
        Pod {
            data: self.data.clone(),
        }
    }
}
