use super::{Provider, Provides};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use bevy_trait_query::{One, RegisterExt};

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

    /// Registers functions this node provides to its descendants.
    ///
    /// The lifetime `'a` allows closures to borrow from `self`, enabling methods
    /// with `&self` arguments to be registered using `add_method_*` methods.
    fn register_provides<'a>(&'a self, provides: &mut Provides<'a>);

    /// Returns `true` if this node should not be subdivided to save resources.
    fn should_subdivide(&self) -> bool;

    /// Subdivides this node into higher-detail children using `child_commands.add_child()`.
    /// Called when the node is close enough to a viewer to warrant more detail.
    fn subdivide(
        &self,
        transform: &GlobalTransform,
        provider: &Provider<'_>,
        child_commands: ChildCommands,
    );
}

#[derive(Component)]
#[require(GlobalTransform)]
pub struct Viewer;

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

#[derive(Default)]
pub struct ProckitFrameworkPlugin {
    registrations: Vec<Box<dyn Fn(&mut App) + Send + Sync>>,
}

impl ProckitFrameworkPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<Node: ProceduralNode + Component>(mut self) -> Self {
        self.registrations.push(Box::new(|app: &mut App| {
            app.register_component_as::<dyn ProceduralNode, Node>();
        }));
        self
    }
}

impl Plugin for ProckitFrameworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProckitFrameworkConfig>()
            .add_systems(Update, resample);
        for registration in &self.registrations {
            registration(app);
        }
    }
}

/// Collects ancestor entity IDs by walking up from a parent to the root.
fn collect_ancestor_entities(
    start_parent: Option<Entity>,
    hierarchy: &Query<&ChildOf>,
) -> Vec<Entity> {
    let mut ancestors = Vec::new();
    let mut current = start_parent;

    while let Some(entity) = current {
        ancestors.push(entity);
        current = hierarchy.get(entity).ok().map(|c| c.parent());
    }

    ancestors
}

/// Builds a Provider from ancestor nodes, with all ancestor references held simultaneously.
/// The `ancestor_refs` slice should be ordered from closest ancestor (index 0) to root (last).
fn build_provider_from_refs<'a>(ancestor_refs: &'a [Ref<'a, dyn ProceduralNode>]) -> Provider<'a> {
    let mut provider = Provider::empty();
    for node_ref in ancestor_refs.iter() {
        let node: &'a dyn ProceduralNode = &**node_ref;
        let mut provides = Provides::new();
        node.register_provides(&mut provides);
        provider.push(provides);
    }
    provider
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
    hierarchy: Query<&ChildOf>,
    all_nodes: Query<One<&dyn ProceduralNode>>,
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
            let ancestor_entities =
                collect_ancestor_entities(child_of.map(|c| c.parent()), &hierarchy);
            let ancestor_nodes: Vec<_> = all_nodes.iter_many(&ancestor_entities).collect();
            let provider = build_provider_from_refs(&ancestor_nodes);

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
