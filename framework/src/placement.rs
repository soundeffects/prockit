use crate::Space;
use bevy::utils::TypeIdMap;
use std::any::{Any, TypeId};

/// A placement opportunity offered by a [`ProceduralNode`] type for potential children, which may
/// be populated by another spawned [`ProceduralNode`] if it accepts the opportunity. A
/// `Placement` may place an object in multiple [`Space`]s simultaneously.
///
/// # Example
/// ```
/// # use prockit_framework::{Placement, PlacementIn, RealSpace, RealSpacePlacement, RealSpaceRegion};
/// # use bevy::prelude::*;
///
/// let placement = Placement::new()
///     .with(PlacementIn::<RealSpace> {
///         placement_type: RealSpacePlacement::NodeSubdivide,
///         region: RealSpaceRegion::default(),
///         transform: Transform::from_translation(Vec3::X),
///         detail_scale: 1.0,
///     });
/// ```
#[derive(Default)]
pub struct Placement {
    space_data: TypeIdMap<Box<dyn Any + Send + Sync>>,
}

impl Placement {
    /// Create a new, empty `Placement`. Use [`Placement::with`] to place in a [`Space`].
    pub fn new() -> Self {
        Self {
            space_data: TypeIdMap::default(),
        }
    }

    /// Add placement data for a [`Space`].
    ///
    /// # Example
    /// ```
    /// # use prockit_framework::{Placement, PlacementIn, RealSpace, RealSpacePlacement, RealSpaceRegion};
    /// # use bevy::prelude::*;
    ///
    /// let placement = Placement::new()
    ///     .with(PlacementIn::<RealSpace> {
    ///         placement_type: RealSpacePlacement::VolumeSubdivide,
    ///         region: RealSpaceRegion { min: Vec3::ZERO, max: Vec3::ONE },
    ///         transform: Transform::IDENTITY,
    ///         detail_scale: 0.5,
    ///     });
    /// ```
    pub fn with<S: Space>(mut self, data: PlacementIn<S>) -> Self {
        self.space_data.insert(TypeId::of::<S>(), Box::new(data));
        self
    }

    /// Get the [`Placement`] for a space if it exists, or `None`.
    ///
    /// # Example
    /// ```
    /// # use prockit_framework::{Placement, PlacementIn, RealSpace};
    ///
    /// let placement = Placement::new();
    /// if let Some(real_placement) = placement.get::<RealSpace>() {
    ///     println!("Detail scale: {}", placement.detail_scale);
    /// }
    /// ```
    pub fn get<S: Space>(&self) -> Option<&PlacementIn<S>> {
        self.space_data
            .get(&TypeId::of::<S>())
            .and_then(|boxed| boxed.downcast_ref())
    }
}

/// Placement info for a [`ProceduralNode`] in a given [`Space`].
pub struct PlacementIn<S: Space> {
    /// The type of placement, from the options provided by the [`Space`]
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
    fn test_empty_placement_set() {
        let placements = Placement::new();
        assert!(placements.get::<RealSpace>().is_none());
    }

    #[test]
    fn test_placement_set_real_space() {
        let placements = Placement::new().with(PlacementIn::<RealSpace> {
            placement_type: RealSpacePlacement::NodeSubdivide,
            region: RealSpaceRegion::default(),
            transform: Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
            detail_scale: 0.5,
        });

        let placement = placements.get::<RealSpace>().unwrap();
        assert_eq!(placement.placement_type, RealSpacePlacement::NodeSubdivide);
        assert_eq!(placement.detail_scale, 0.5);
    }
}
