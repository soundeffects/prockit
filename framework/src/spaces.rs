use bevy::prelude::*;

/// The `Space` trait defines the conceptual space in which procedural generation operates.
/// This includes any number of Euclidean spaces of arbitrary dimensions, and any space which
/// defines transforms.
///
/// # Example
/// ```
/// # use prockit_framework::{Space};
/// # use bevy::prelude::*;
/// struct RealSpace;
///
/// impl Space for RealSpace {
///     type Position = Vec3;
///     type GlobalTransform = GlobalTransform;
///     type LocalTransform = Transform;
///     type Region = Sphere;
///     type PlacementType = // enum list...
/// # ();
///
///     fn noticeability(node: &GlobalTransform, viewer: &GlobalTransform) -> f32 {
///         unimplemented!()
///     }
///
///     fn push_transform(parent: &GlobalTransform, child: &Transform) -> GlobalTransform {
///         unimplemented!()
///     }
/// }
/// ```
pub trait Space: Clone + Send + Sync + 'static {
    /// The coordinate type used for spatial sampling (e.g. [`Vec3`] for 3D space)
    type Position;

    /// A [`Component`] for world-space transforms (e.g. [`GlobalTransform`] for 3D)
    type GlobalTransform: Component + Clone + Default;

    /// A [`Component`] for parent-relative transforms (e.g. [`Transform`] for 3D)
    type LocalTransform: Component + Clone + Default;

    /// Describes a region of space a [`ProceduralNode`] governs (e.g., [`Sphere`] for 3D)
    type LocalRegion: Clone + Default + Send + Sync + 'static;

    /// An `enum` type of placement strategies (e.g. scattering vs subdivision)
    type PlacementType: Clone + Copy + PartialEq + Eq + Send + Sync + 'static;

    /// Computes the "noticeability" of a [`ProceduralNode`] from a viewer's perspective. Depending
    /// on the implementation, thi usually uses a combination of distance from viewers and the scale
    /// of the procedural node
    fn noticeability(node: &Self::GlobalTransform, viewer: &Self::GlobalTransform) -> f32;

    /// Composes a child's global transform from its local transform and the parent's global
    /// transform
    fn push_transform(
        parent: &Self::GlobalTransform,
        child: &Self::LocalTransform,
    ) -> Self::GlobalTransform;
}

/// Placement types for [`RealSpace`], representing different strategies for placing
/// child nodes in 3D space.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RealSpacePlacement {
    /// `VolumeSubdivide` strategy semantically allows children of any [`ProceduralNode`] type which
    /// represents volumes to be spawned to cover the region of the parent node
    VolumeSubdivide,
    /// `NodeSubdivide` strategy semantically mandates that only [`ProceduralNode`]s of the same
    /// type as the parent node are allowed as children
    NodeSubdivide,
    /// `VolumeScatter` allows nodes to be placed in random subregions, semantically indicating to
    /// the children that they may be at any point inside the opaque volume (usually solid material)
    /// of the parent node
    VolumeScatter,
    /// `SurfaceScatter` allows nodes to be placed in random subregions, semantically indicating to
    /// the children that they are specifically near the surface (bordering on some transparent
    /// medium, like air) of a parent node.
    SurfaceScatter,
}

/// Axis-aligned bounding box defining a local region in [`RealSpace`].
#[derive(Clone, Debug, Default)]
pub struct RealSpaceRegion {
    /// Minimum corner of the bounding box
    pub min: Vec3,
    /// Maximum corner of the bounding box
    pub max: Vec3,
}

/// A [`Space`] implementation for standard 3D game world space using Bevy's transform types.
/// The noticeability calculation uses the node's scale and distance-squared from the viewer,
/// as an approximation for the area of the field-of-view an object may cover.
///
/// # Example
/// ```
/// use prockit_framework::{FrameworkPlugin, RealSpace, MB};
///
/// // Configure the framework plugin for 3D space
/// // Tell the space to use roughly 50% of a 64MB memory reservation
/// let plugin = FrameworkPlugin::new()
///     .with_space::<RealSpace>(64 * MB, 0.5);
/// ```
#[derive(Clone)]
pub struct RealSpace;

impl Space for RealSpace {
    type Position = Vec3;
    type GlobalTransform = GlobalTransform;
    type LocalTransform = Transform;
    type LocalRegion = RealSpaceRegion;
    type PlacementType = RealSpacePlacement;

    fn noticeability(node: &GlobalTransform, viewer: &GlobalTransform) -> f32 {
        node.scale().max_element() / viewer.translation().distance_squared(node.translation())
    }

    fn push_transform(parent: &GlobalTransform, child: &Transform) -> GlobalTransform {
        parent.mul_transform(*child)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realspace_noticeability_increases_with_scale() {
        let viewer = GlobalTransform::from_translation(Vec3::ZERO);
        let small_node = GlobalTransform::from(
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)).with_scale(Vec3::splat(1.0)),
        );
        let large_node = GlobalTransform::from(
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)).with_scale(Vec3::splat(10.0)),
        );

        let small_noticeability = RealSpace::noticeability(&small_node, &viewer);
        let large_noticeability = RealSpace::noticeability(&large_node, &viewer);

        assert!(large_noticeability > small_noticeability);
    }

    #[test]
    fn test_realspace_noticeability_decreases_with_distance() {
        let viewer = GlobalTransform::from_translation(Vec3::ZERO);
        let near_node = GlobalTransform::from(
            Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)).with_scale(Vec3::splat(1.0)),
        );
        let far_node = GlobalTransform::from(
            Transform::from_translation(Vec3::new(20.0, 0.0, 0.0)).with_scale(Vec3::splat(1.0)),
        );

        let near_noticeability = RealSpace::noticeability(&near_node, &viewer);
        let far_noticeability = RealSpace::noticeability(&far_node, &viewer);

        assert!(near_noticeability > far_noticeability);
    }

    #[test]
    fn test_realspace_push_transform_translation() {
        let parent = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let child = Transform::from_translation(Vec3::new(5.0, 0.0, 0.0));
        let result = RealSpace::push_transform(&parent, &child);
        assert_eq!(result.translation(), Vec3::new(15.0, 0.0, 0.0));
    }

    #[test]
    fn test_realspace_push_transform_with_scale() {
        let parent = GlobalTransform::from_scale(Vec3::splat(2.0));
        let child = Transform::from_translation(Vec3::new(5.0, 0.0, 0.0));
        let result = RealSpace::push_transform(&parent, &child);
        assert_eq!(result.translation(), Vec3::new(10.0, 0.0, 0.0));
    }

    #[test]
    fn test_realspace_push_transform_identity() {
        let parent = GlobalTransform::IDENTITY;
        let child = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let result = RealSpace::push_transform(&parent, &child);
        assert_eq!(result.translation(), Vec3::new(1.0, 2.0, 3.0));
    }
}
