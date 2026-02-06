use crate::Space;
use bevy::utils::TypeIdMap;
use std::any::{Any, TypeId};

/// A placement opportunity offered by a parent node.
///
/// Placement is **space-agnostic**: it can store placement data for multiple spaces simultaneously.
/// This allows nodes that exist in multiple spaces to receive all relevant placement information
/// in a single structure.
///
/// # Example
/// ```
/// use prockit_framework::{Placement, SpacePlacement, RealSpace, RealSpacePlacement, RealSpaceRegion};
/// use bevy::prelude::*;
///
/// let placement = Placement::new()
///     .with_space::<RealSpace>(SpacePlacement {
///         placement_type: RealSpacePlacement::NodeSubdivide,
///         region: RealSpaceRegion::default(),
///         transform: Transform::from_translation(Vec3::X),
///         detail_scale: 1.0,
///     });
/// ```
pub struct Placement {
    /// Per-space placement data (region, transform, detail_scale, placement_type)
    space_data: TypeIdMap<Box<dyn Any + Send + Sync>>,
}

impl Default for Placement {
    fn default() -> Self {
        Self::new()
    }
}

impl Placement {
    /// Create a new empty placement with no space data.
    pub fn new() -> Self {
        Self {
            space_data: TypeIdMap::default(),
        }
    }

    /// Add placement data for a specific space. Returns self for chaining.
    ///
    /// # Example
    /// ```
    /// use prockit_framework::{Placement, SpacePlacement, RealSpace, RealSpacePlacement, RealSpaceRegion};
    /// use bevy::prelude::*;
    ///
    /// let placement = Placement::new()
    ///     .with_space::<RealSpace>(SpacePlacement {
    ///         placement_type: RealSpacePlacement::VolumeSubdivide,
    ///         region: RealSpaceRegion { min: Vec3::ZERO, max: Vec3::ONE },
    ///         transform: Transform::IDENTITY,
    ///         detail_scale: 0.5,
    ///     });
    /// ```
    pub fn with_space<S: Space>(mut self, data: SpacePlacement<S>) -> Self {
        self.space_data
            .insert(TypeId::of::<S>(), Box::new(data));
        self
    }

    /// Get placement data for a specific space, if present.
    ///
    /// # Example
    /// ```
    /// use prockit_framework::{Placement, SpacePlacement, RealSpace};
    ///
    /// let placement = Placement::new();
    /// if let Some(space_placement) = placement.get::<RealSpace>() {
    ///     println!("Detail scale: {}", space_placement.detail_scale);
    /// }
    /// ```
    pub fn get<S: Space>(&self) -> Option<&SpacePlacement<S>> {
        self.space_data
            .get(&TypeId::of::<S>())
            .and_then(|boxed| boxed.downcast_ref())
    }

    /// Check if this placement has data for a specific space.
    pub fn has_space<S: Space>(&self) -> bool {
        self.space_data.contains_key(&TypeId::of::<S>())
    }
}

/// Typed placement data for a specific space.
///
/// Contains all the information a node needs to decide whether to accept a placement
/// and to generate itself at that placement.
pub struct SpacePlacement<S: Space> {
    /// The type of placement (e.g., VolumeSubdivide, SurfaceScatter)
    pub placement_type: S::PlacementType,
    /// The local region this placement governs
    pub region: S::LocalRegion,
    /// The local transform relative to the parent
    pub transform: S::LocalTransform,
    /// The detail scale for this placement (smaller = more detailed)
    pub detail_scale: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RealSpace, RealSpacePlacement, RealSpaceRegion};
    use bevy::prelude::*;

    #[test]
    fn test_placement_new_is_empty() {
        let placement = Placement::new();
        assert!(!placement.has_space::<RealSpace>());
    }

    #[test]
    fn test_placement_with_space() {
        let placement = Placement::new().with_space::<RealSpace>(SpacePlacement {
            placement_type: RealSpacePlacement::NodeSubdivide,
            region: RealSpaceRegion::default(),
            transform: Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
            detail_scale: 0.5,
        });

        assert!(placement.has_space::<RealSpace>());

        let space_data = placement.get::<RealSpace>().unwrap();
        assert_eq!(space_data.placement_type, RealSpacePlacement::NodeSubdivide);
        assert_eq!(space_data.detail_scale, 0.5);
    }

    #[test]
    fn test_placement_get_missing_space_returns_none() {
        let placement = Placement::new();
        assert!(placement.get::<RealSpace>().is_none());
    }
}
