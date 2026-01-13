use super::Space;
use crate::{ProceduralNode, Provider, Provides};
use bevy::{
    ecs::{component::Mutable, entity_disabling::Disabled},
    platform::collections::HashMap,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};
use bevy_trait_query::One;
use std::{any::TypeId, marker::PhantomData};

#[derive(Message)]
pub(crate) struct SpawnNode<S: Space, Node: ProceduralNode<S>> {
    parent: Option<Entity>,
    local_transform: S::LocalTransform,
    node_phantom_data: PhantomData<Node>,
}

impl<S: Space, Node: ProceduralNode<S>> SpawnNode<S, Node> {
    pub(crate) fn new(parent: Option<Entity>, local_transform: S::LocalTransform) -> Self {
        Self {
            parent,
            local_transform,
            node_phantom_data: PhantomData,
        }
    }

    pub(crate) fn parent(&self) -> Option<Entity> {
        self.parent
    }
}

/// Marker component indicating a node needs to have its data generated.
#[derive(Component)]
pub(crate) struct Generate;

/// Marker component indicating a node needs subdivision.
#[derive(Component)]
pub(crate) struct NeedsSubdivision;

/// Holds the result of generating a single child node.
pub struct GeneratedChild<S: Space, Node> {
    pub node: Node,
    pub local_transform: S::LocalTransform,
}

/// Component holding an async task that generates all children for a subdividing node.
/// All children are generated in a single task to minimize overhead.
#[derive(Component)]
pub struct GenerationTask<S: Space, Node: ProceduralNode<S>> {
    task: Task<Vec<GeneratedChild<S, Node>>>,
    _marker: PhantomData<S>,
}

impl<S: Space, Node: ProceduralNode<S>> GenerationTask<S, Node> {
    pub fn new(task: Task<Vec<GeneratedChild<S, Node>>>) -> Self {
        Self {
            task,
            _marker: PhantomData,
        }
    }
}

/// Collects ancestor entity IDs by walking up from a parent to the root.
/// Returns ancestors ordered from closest (index 0) to root (last).
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

/// Builds an owned Provider from ancestor nodes.
/// The ancestors should be ordered from closest (index 0) to root (last).
fn build_owned_provider<S: Space + 'static>(
    ancestor_entities: &[Entity],
    all_nodes: &Query<One<&dyn ProceduralNode<S>>>,
) -> Provider<S> {
    let mut provider = Provider::empty();
    // Iterate in reverse so root is pushed first, then closer ancestors
    for &entity in ancestor_entities.iter().rev() {
        if let Ok(node) = all_nodes.get(entity) {
            let mut provides = Provides::new();
            node.provides(&mut provides);
            provider.push(provides);
        }
    }
    provider
}

/// System that spawns async generation tasks for nodes that need subdivision.
/// All children of a single parent are generated in one async task.
pub fn spawn_generation_tasks<
    S: Space + 'static,
    Node: ProceduralNode<S> + Component<Mutability = Mutable> + Clone,
>(
    mut commands: Commands,
    nodes_to_subdivide: Query<
        (Entity, &Node, &S::GlobalTransform),
        (With<NeedsSubdivision>, Without<GenerationTask<S, Node>>),
    >,
    hierarchy: Query<&ChildOf>,
    all_nodes: Query<One<&dyn ProceduralNode<S>>>,
) where
    S::GlobalTransform: Clone,
    S::LocalTransform: Clone + Send + 'static,
{
    let task_pool = AsyncComputeTaskPool::get();

    for (entity, node, global_transform) in nodes_to_subdivide.iter() {
        // Get the list of children to generate
        let Some(child_list) = node.subdivide() else {
            // Node decided not to subdivide, remove the marker
            commands.entity(entity).remove::<NeedsSubdivision>();
            continue;
        };

        // Build provider from ancestors (this is cloneable now)
        let ancestor_entities = collect_ancestor_entities(Some(entity), &hierarchy);
        let provider = build_owned_provider::<S>(&ancestor_entities, &all_nodes);

        // Clone data needed for the async task
        let transform = global_transform.clone();

        // Collect child transforms for the task
        let child_transforms: Vec<S::LocalTransform> = child_list
            .iter()
            .map(|(_, local_transform)| local_transform.clone())
            .collect();

        // Spawn a single task to generate all children
        let task = task_pool.spawn(async move {
            let mut results = Vec::with_capacity(child_transforms.len());

            for local_transform in child_transforms {
                // Create and generate each child node
                let mut child = Node::init();
                // TODO: Compute proper child global transform from parent + local
                child.generate(&transform, &provider);

                results.push(GeneratedChild {
                    node: child,
                    local_transform,
                });
            }

            results
        });

        // Attach the task to the parent entity
        commands
            .entity(entity)
            .remove::<NeedsSubdivision>()
            .insert(GenerationTask::<S, Node>::new(task));
    }
}

/// System that polls generation tasks and spawns child entities when complete.
pub fn poll_generation_tasks<
    S: Space + 'static,
    Node: ProceduralNode<S> + Component<Mutability = Mutable>,
>(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut GenerationTask<S, Node>)>,
) where
    S::LocalTransform: Clone,
{
    for (parent_entity, mut generation_task) in tasks.iter_mut() {
        // Poll the task without blocking
        if let Some(children) = block_on(poll_once(&mut generation_task.task)) {
            // Task completed, spawn all children
            for child in children {
                commands.spawn((child.node, child.local_transform, ChildOf(parent_entity)));
            }

            // Remove the task component
            commands
                .entity(parent_entity)
                .remove::<GenerationTask<S, Node>>();
        }
    }
}

// Legacy spawn_nodes function - kept for reference but may be removed
fn _spawn_nodes<S: Space, Node: ProceduralNode<S> + Component<Mutability = Mutable>>(
    mut commands: Commands,
    free_nodes: Query<Entity, (With<Node>, With<Disabled>)>,
    mut spawns: MessageReader<SpawnNode<S, Node>>,
    transforms: Query<&S::GlobalTransform>,
    _hierarchy: Query<&ChildOf>,
    _all_nodes: Query<One<&dyn ProceduralNode<S>>>,
) where
    S::GlobalTransform: Clone,
{
    let mut reader = spawns.read().peekable();
    let mut free_nodes = free_nodes.iter().peekable();
    while reader.peek().is_some() && free_nodes.peek().is_some() {
        let message = reader.next().unwrap();
        let free_entity = free_nodes.next().unwrap();
        let _global_transform = if let Some(parent) = message.parent() {
            commands.entity(parent).add_child(free_entity);
            transforms.get(parent).unwrap().clone()
        } else {
            S::GlobalTransform::default()
        };

        commands
            .entity(free_entity)
            .remove::<Disabled>()
            .insert(Generate);
    }
}
