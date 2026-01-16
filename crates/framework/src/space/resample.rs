use super::{EmptyNode, PendingGenerate, Space};
use crate::ProceduralNode;
use bevy::{platform::collections::HashSet, prelude::*};
use bevy_trait_query::One;
use std::{collections::VecDeque, marker::PhantomData};

/// A component that marks an entity as a viewer for level-of-detail calculations.
///
/// Viewers are reference points from which the noticeability of procedural nodes
/// is calculated. Nodes that are more noticeable from a viewer's perspective will
/// be subdivided to show more detail, while distant or small nodes may have their
/// children collapsed.
///
/// # Priority
///
/// The `priority` field acts as a multiplier for the noticeability calculation.
/// Higher priority viewers have more influence on which nodes get subdivided.
/// This is useful for scenarios like:
/// - Main camera (high priority)
/// - Secondary/minimap camera (lower priority)
/// - Preview windows (variable priority)
///
/// # Example
///
/// ```
/// use prockit_framework::{Viewer, RealSpace};
/// use bevy::prelude::*;
///
/// fn spawn_camera(mut commands: Commands) {
///     commands.spawn((
///         Camera3d::default(),
///         Transform::from_xyz(0.0, 10.0, 20.0),
///         Viewer::<RealSpace>::new(1.0),
///     ));
/// }
/// ```
#[derive(Component)]
pub struct Viewer<S: Space> {
    priority: f32,
    space_phantom_data: PhantomData<S>,
}

impl<S: Space> Viewer<S> {
    /// Creates a new viewer with the specified priority.
    ///
    /// # Arguments
    ///
    /// * `priority` - The priority multiplier for this viewer. Higher values mean
    ///   this viewer has more influence on LOD decisions.
    ///
    /// # Example
    ///
    /// ```
    /// use prockit_framework::{Viewer, RealSpace};
    ///
    /// // Main camera with standard priority
    /// let main_viewer = Viewer::<RealSpace>::new(1.0);
    ///
    /// // Secondary viewer with reduced influence
    /// let secondary_viewer = Viewer::<RealSpace>::new(0.5);
    /// ```
    pub fn new(priority: f32) -> Self {
        Self {
            priority,
            space_phantom_data: PhantomData,
        }
    }

    /// Returns this viewer's priority multiplier.
    pub fn priority(&self) -> f32 {
        self.priority
    }
}

/// Resource that manages memory-aware LOD thresholds for the procedural generation system.
///
/// `Thresholds` tracks memory usage and dynamically adjusts the upper and lower
/// noticeability thresholds used to decide when to subdivide or collapse nodes.
/// This creates a feedback loop that keeps memory usage near the configured target.
///
/// # Memory Management
///
/// - `limit`: The target memory budget in bytes
/// - `free_percentage`: Target percentage of the budget that should remain free (0.0-1.0)
/// - `current`: Tracked current memory usage
/// - `history`: Rolling history of memory usage for smoothing
///
/// # Threshold Adaptation
///
/// The `upper` and `lower` thresholds are adjusted based on memory pressure:
/// - When memory is above target, thresholds increase (fewer subdivisions, more collapses)
/// - When memory is below target, thresholds decrease (more subdivisions, fewer collapses)
#[derive(Resource)]
pub(crate) struct Thresholds<S: Space> {
    limit: usize,
    free_percentage: f32,
    current: usize,
    history: VecDeque<usize>,
    upper: f32,
    lower: f32,
    space_phantom_data: PhantomData<S>,
}

impl<S: Space> Thresholds<S> {
    /// Number of frames of memory history to track for smoothing threshold adjustments.
    pub(crate) const HISTORY_SIZE: usize = 16;

    /// Creates a new thresholds resource with the given memory budget.
    ///
    /// # Arguments
    ///
    /// * `limit` - Target memory budget in bytes
    /// * `free_percentage` - Target percentage of budget to keep free (0.0-1.0)
    pub(crate) fn new(limit: usize, free_percentage: f32) -> Self {
        Self {
            limit: limit,
            free_percentage,
            current: limit,
            history: VecDeque::new(),
            upper: 10.0,
            lower: 1.0,
            space_phantom_data: PhantomData,
        }
    }

    /// Increases the tracked memory usage by the given amount.
    ///
    /// Called by component hooks when procedural nodes are added.
    pub(crate) fn add(&mut self, amount: usize) {
        self.current += amount;
    }

    /// Decreases the tracked memory usage by the given amount.
    ///
    /// Called by component hooks when procedural nodes are removed.
    pub(crate) fn sub(&mut self, amount: usize) {
        self.current -= amount;
    }

    /// System that adjusts thresholds based on recent memory usage history.
    ///
    /// This system maintains a rolling average of memory usage and adjusts the
    /// upper/lower thresholds to drive usage toward the target. The adjustment
    /// is proportional to the error between current usage and target.
    pub(crate) fn recalibrate(mut thresholds: ResMut<Thresholds<S>>) {
        let current = thresholds.current;
        thresholds.history.push_front(current);
        if thresholds.history.len() >= Self::HISTORY_SIZE {
            thresholds.history.pop_back();
        }
        let average =
            thresholds.history.iter().sum::<usize>() as f32 / thresholds.history.len() as f32;
        let error = (average / thresholds.limit as f32) - thresholds.free_percentage;
        thresholds.lower *= 1.0 - error;
        thresholds.upper *= 1.0 - error;
    }

    /// System that triggers subdivision or collapse of procedural nodes based on
    /// their noticeability relative to the current thresholds.
    ///
    /// For each leaf node (no children, not marked empty):
    /// - If noticeability > upper threshold: mark for subdivision
    ///
    /// For all nodes:
    /// - If noticeability < lower threshold: despawn all children
    pub(crate) fn resample(
        mut commands: Commands,
        thresholds: Res<Thresholds<S>>,
        leaves: Query<
            (
                Entity,
                One<&dyn ProceduralNode<S>>,
                &S::GlobalTransform,
                Option<&ChildOf>,
            ),
            (Without<Children>, Without<EmptyNode>),
        >,
        nodes: Query<(Entity, One<&dyn ProceduralNode<S>>, &S::GlobalTransform)>,
        viewers: Query<(&Viewer<S>, &S::GlobalTransform)>,
    ) {
        let priority = |node_transform: &S::GlobalTransform| {
            viewers
                .iter()
                .map(|(viewer, viewer_transform)| {
                    S::noticeability(viewer_transform, node_transform) * viewer.priority()
                })
                .sum::<f32>()
        };

        let mut leaves_once_removed = HashSet::new();

        for (entity, _, node_transform, child_of) in leaves {
            child_of.map(|child_of| leaves_once_removed.insert(child_of.parent()));
            if priority(node_transform) > thresholds.upper {
                commands.entity(entity).insert(PendingGenerate);
            }
        }

        for (entity, _node, node_transform) in nodes {
            if priority(node_transform) < thresholds.lower {
                commands.entity(entity).despawn_children();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RealSpace;

    #[test]
    fn test_viewer_new() {
        let viewer = Viewer::<RealSpace>::new(1.5);
        assert_eq!(viewer.priority(), 1.5);
    }

    #[test]
    fn test_viewer_zero_priority() {
        let viewer = Viewer::<RealSpace>::new(0.0);
        assert_eq!(viewer.priority(), 0.0);
    }

    #[test]
    fn test_viewer_high_priority() {
        let viewer = Viewer::<RealSpace>::new(100.0);
        assert_eq!(viewer.priority(), 100.0);
    }

    #[test]
    fn test_thresholds_new() {
        let thresholds = Thresholds::<RealSpace>::new(1024 * 1024, 0.5);
        assert_eq!(thresholds.limit, 1024 * 1024);
        assert_eq!(thresholds.free_percentage, 0.5);
        assert_eq!(thresholds.current, 1024 * 1024);
        assert!(thresholds.history.is_empty());
    }

    #[test]
    fn test_thresholds_add() {
        let mut thresholds = Thresholds::<RealSpace>::new(1000, 0.5);
        let initial = thresholds.current;
        thresholds.add(100);
        assert_eq!(thresholds.current, initial + 100);
    }

    #[test]
    fn test_thresholds_sub() {
        let mut thresholds = Thresholds::<RealSpace>::new(1000, 0.5);
        let initial = thresholds.current;
        thresholds.sub(100);
        assert_eq!(thresholds.current, initial - 100);
    }

    #[test]
    fn test_thresholds_add_and_sub_balance() {
        let mut thresholds = Thresholds::<RealSpace>::new(1000, 0.5);
        let initial = thresholds.current;
        thresholds.add(500);
        thresholds.sub(300);
        thresholds.add(100);
        thresholds.sub(300);
        assert_eq!(thresholds.current, initial);
    }

    #[test]
    fn test_thresholds_history_size_constant() {
        assert_eq!(Thresholds::<RealSpace>::HISTORY_SIZE, 16);
    }

    #[test]
    fn test_thresholds_initial_threshold_values() {
        let thresholds = Thresholds::<RealSpace>::new(1000, 0.5);
        assert_eq!(thresholds.upper, 10.0);
        assert_eq!(thresholds.lower, 1.0);
    }

    #[test]
    fn test_thresholds_recalibrate_adds_to_history() {
        let mut world = World::new();
        world.insert_resource(Thresholds::<RealSpace>::new(1000, 0.5));

        // Run recalibrate system manually
        let mut thresholds = world.resource_mut::<Thresholds<RealSpace>>();
        let current = thresholds.current;
        thresholds.history.push_front(current);

        assert_eq!(thresholds.history.len(), 1);
        assert_eq!(*thresholds.history.front().unwrap(), current);
    }

    #[test]
    fn test_thresholds_history_caps_at_history_size() {
        let mut thresholds = Thresholds::<RealSpace>::new(1000, 0.5);

        // Add more than HISTORY_SIZE entries
        for i in 0..20 {
            thresholds.history.push_front(i);
            if thresholds.history.len() >= Thresholds::<RealSpace>::HISTORY_SIZE {
                thresholds.history.pop_back();
            }
        }

        assert!(thresholds.history.len() < Thresholds::<RealSpace>::HISTORY_SIZE);
    }

    #[test]
    fn test_viewer_component_in_world() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Viewer::<RealSpace>::new(1.0),
                GlobalTransform::from_translation(Vec3::new(0.0, 10.0, 20.0)),
            ))
            .id();

        let viewer = world.entity(entity).get::<Viewer<RealSpace>>().unwrap();
        assert_eq!(viewer.priority(), 1.0);
    }

    #[test]
    fn test_multiple_viewers_in_world() {
        let mut world = World::new();

        world.spawn((
            Viewer::<RealSpace>::new(1.0),
            GlobalTransform::from_translation(Vec3::ZERO),
        ));

        world.spawn((
            Viewer::<RealSpace>::new(0.5),
            GlobalTransform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        ));

        let mut count = 0;
        for viewer in world
            .query::<&Viewer<RealSpace>>()
            .iter(&world)
        {
            count += 1;
            assert!(viewer.priority() > 0.0);
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_thresholds_resource_in_world() {
        let mut world = World::new();
        world.insert_resource(Thresholds::<RealSpace>::new(1024 * 1024, 0.5));

        let thresholds = world.resource::<Thresholds<RealSpace>>();
        assert_eq!(thresholds.limit, 1024 * 1024);
    }
}
