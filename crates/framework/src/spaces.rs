use bevy::prelude::*;

/// The `Space` trait defines the conceptual space in which procedural generation operates.
/// One might imagine physical space as a 3D space, images as a 2D space, audio as 1D space,
/// and so on. As long as elements have a position and can be transformed in the space, it can
/// support procedural generation.
///
/// # Associated Types
/// * `Position` - The coordinate type used for spatial sampling (e.g., `Vec3`)
/// * `GlobalTransform` - Component type for world-space transforms
/// * `LocalTransform` - Component type for parent-relative transforms
///
/// # Example
/// ```
/// # use prockit_framework::{Space};
/// struct OneDimension;
///
/// struct OneTransform {
///     scale: f32,
///     translation: f32
/// }
///
/// impl Space for OneDimension {
///     type Position = f32;
///     type GlobalTransform = OneTransform;
///     type LocalTransform = OneTransform;
///
///     fn noticeability(node: &OneTransform, viewer: &OneTransform) -> f32 {
///         node.scale / (node.translation - viewer.translation).abs()
///     }
///
///     fn push_transform(parent: &OneTransform, child: &OneTransform) -> OneTransform {
///         OneTransform {
///             scale: parent.scale * child.scale,
///             translation parent.translation + child.translation
///         }
///     }
/// }
/// ```
pub trait Space: Clone + Send + Sync + 'static {
    /// A type definition for sampling functions in `Provider`s to use as input.
    type Position;

    /// The component type for world-space (global) transforms.
    /// Must be a Bevy component that can be cloned and has a sensible default.
    type GlobalTransform: Component + Clone + Default;

    /// The component type for parent-relative (local) transforms.
    /// Must be a Bevy component that can be cloned and has a sensible default.
    type LocalTransform: Component + Clone + Default;

    /// Computes the "noticeability" of a node from a viewer's perspective. This usually uses
    /// a combination of distance from viewers, scale of procedural node, and priority of
    /// viewers to create a noticeability score.
    ///
    /// The scale of noticeability depends on the implementation, but higher values mean greater
    /// noticeability.
    ///
    /// # Example
    fn noticeability(node: &Self::GlobalTransform, viewer: &Self::GlobalTransform) -> f32;

    /// Composes a parent's global transform with a child's local transform to produce
    /// the child's global transform.
    fn push_transform(
        parent: &Self::GlobalTransform,
        child: &Self::LocalTransform,
    ) -> Self::GlobalTransform;
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
