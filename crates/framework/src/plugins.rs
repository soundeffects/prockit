use super::{Provider, Provides};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use bevy_trait_query::{One, RegisterExt};
use std::marker::PhantomData;

/// A wrapper around Bevy's `Commands` that only exposes the ability to add children, passed to
/// a `ProceduralNode` when `ProceduralNode::subdivide` is called.
pub struct ChildCommands<'w, 's> {
    commands: Commands<'w, 's>,
    parent: Entity,
}

impl<'w, 's> ChildCommands<'w, 's> {
    /// Creates a new `ChildCommands`, to be used by a `ProceduralNode` to add children to
    /// itself.
    pub(crate) fn new(commands: Commands<'w, 's>, parent: Entity) -> Self {
        Self { commands, parent }
    }

    /// Spawns a bundle as a child of the `ProceduralNode` associated with the `ChildCommands`.
    pub fn add_child(&mut self, bundle: impl Bundle) {
        self.commands.entity(self.parent).with_child(bundle);
    }

    /// Uses a spawner function to add multiple children to the `ProceduralNode` associated with
    /// the `ChildCommands`.
    pub fn add_children(&mut self, func: impl FnOnce(&mut RelatedSpawnerCommands<ChildOf>)) {
        self.commands.entity(self.parent).with_children(func);
    }
}

/// `ProceduralNode` defines the interface for nodes in the procedurally-generated,
/// level-of-detail hierarchy that can be managed by the prockit framework. See the
/// documentation for each member method for more details.
#[bevy_trait_query::queryable]
pub trait ProceduralNode {
    /// Returns the minimum squared distance from this node to a viewer position, used to
    /// determine when to increase or decrease the level-of-detail.
    fn distance(&self, transform: GlobalTransform, viewer: Vec3) -> f32 {
        transform.translation().distance_squared(viewer)
    }

    /// Returns the functions this node provides to its descendants in the form of a `Provides`
    /// struct.
    fn provides(&self) -> Provides;

    /// Returns `true` if this node should not be subdivided to save resources.
    fn should_subdivide(&self) -> bool;

    /// Subdivides this node into higher-detail children using `child_commands.add_child()`.
    /// Called when the node is close enough to a viewer to warrant more detail.
    fn subdivide(
        &self,
        transform: &GlobalTransform,
        provider: &Provider,
        child_commands: ChildCommands,
    );
}

#[derive(Component)]
pub struct Viewer;

pub struct ProceduralNodePlugin<T> {
    procedural_node_type: PhantomData<T>,
}

impl<T> ProceduralNodePlugin<T> {
    pub fn new() -> Self {
        Self {
            procedural_node_type: PhantomData,
        }
    }
}

impl<T> Plugin for ProceduralNodePlugin<T>
where
    T: Component + ProceduralNode + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.register_component_as::<dyn ProceduralNode, T>();
    }
}

/// Configuration for the prockit framework's level-of-detail management.
#[derive(Resource)]
pub struct ProckitFrameworkConfig {
    /// Distance threshold below which nodes should subdivide (viewer is close, need more detail).
    /// This is compared against squared distance values.
    pub subdivide_threshold: f32,
    /// Distance threshold above which nodes should collapse (viewer is far, need less detail).
    /// This is compared against squared distance values.
    pub collapse_threshold: f32,
}

impl Default for ProckitFrameworkConfig {
    fn default() -> Self {
        Self {
            subdivide_threshold: 100.0, // 10 units squared
            collapse_threshold: 400.0,  // 20 units squared
        }
    }
}

pub struct ProckitFrameworkPlugin;

impl Plugin for ProckitFrameworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProckitFrameworkConfig>()
            .add_systems(Update, resample);
    }
}

/// Collects a `Provider` for a node by walking up from its parent to the root.
///
/// The resulting `Provider` contains `Provides` from each ancestor, ordered from
/// the direct parent (first) to the root (last).
fn collect_provider(
    start_parent: Option<Entity>,
    nodes: &Query<(One<&dyn ProceduralNode>, Option<&ChildOf>)>,
) -> Provider {
    let mut provides_chain = Vec::new();
    let mut current = start_parent;

    while let Some(entity) = current {
        if let Ok((node, child_of)) = nodes.get(entity) {
            provides_chain.push(node.provides());
            current = child_of.map(|c| c.parent());
        } else {
            break;
        }
    }

    Provider::new(provides_chain)
}

/// Manages level-of-detail by subdividing and collapsing nodes based on viewer distance.
fn resample(
    mut commands: Commands,
    config: Res<ProckitFrameworkConfig>,
    viewers: Query<&GlobalTransform, With<Viewer>>,
    leaves: Query<
        (
            Entity,
            One<&dyn ProceduralNode>,
            &GlobalTransform,
            Option<&ChildOf>,
        ),
        Without<Children>,
    >,
    branches: Query<(
        Entity,
        One<&dyn ProceduralNode>,
        &GlobalTransform,
        &Children,
    )>,
    all_nodes: Query<(One<&dyn ProceduralNode>, Option<&ChildOf>)>,
) {
    // Collect viewer positions
    let viewer_positions: Vec<Vec3> = viewers.iter().map(|t| t.translation()).collect();

    if viewer_positions.is_empty() {
        return;
    }

    let mut leaves_once_removed = vec![];

    // Subdivide
    for (entity, node, transform, child_of) in leaves.iter() {
        if let Some(parent) = child_of {
            leaves_once_removed.push(parent.parent());
        }

        if !node.should_subdivide() {
            continue;
        }

        let min_dist = viewer_positions
            .iter()
            .map(|&viewer_pos| node.distance(*transform, viewer_pos))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        if min_dist < config.subdivide_threshold {
            let provider = collect_provider(child_of.map(|c| c.parent()), &all_nodes);
            node.subdivide(
                &transform,
                &provider,
                ChildCommands::new(commands.reborrow(), entity),
            );
        }
    }

    // Collapse
    for (entity, node, transform, _children) in leaves_once_removed
        .iter()
        .filter_map(|entity| branches.get(*entity).ok())
        .filter(|(_, _, _, children)| children.iter().all(|child| leaves.contains(child)))
    {
        let min_dist = viewer_positions
            .iter()
            .map(|&viewer_pos| node.distance(*transform, viewer_pos))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        if min_dist > config.collapse_threshold {
            commands.entity(entity).despawn_children();
        }
    }
}
