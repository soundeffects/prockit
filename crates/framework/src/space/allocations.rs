use super::Space;
use crate::{ProceduralNode, Provider, Provides};
use bevy::{
    ecs::{component::Mutable, entity_disabling::Disabled},
    platform::collections::HashMap,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_trait_query::One;
use std::{any::TypeId, marker::PhantomData, sync::Arc};

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

#[derive(Component)]
pub(crate) struct Generate;

// #[derive(Component)]
// pub(crate) struct GenerateTask<S: Space, Node: ProceduralNode<S>> {
//     task: Task<Node>,
//     space_phantom_data: PhantomData<S>,
// }
//
// impl<S: Space, Node: ProceduralNode<S>> GenerateTask<S, Node> {
//     pub(crate) fn new(task: Task<Node>) -> Self {
//         Self {
//             task,
//             space_phantom_data: PhantomData,
//         }
//     }
// }

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
fn build_provider_from_refs<'a, S: Space + 'static>(
    ancestor_refs: &'a [Ref<'a, dyn ProceduralNode<S>>],
) -> Provider<'a, S> {
    let mut provider = Provider::empty();
    for node_ref in ancestor_refs.iter() {
        let node: &'a dyn ProceduralNode<S> = &**node_ref;
        let mut provides = Provides::new();
        node.provides(&mut provides);
        provider.push(provides);
    }
    provider
}

fn spawn_nodes<S: Space, Node: ProceduralNode<S> + Component<Mutability = Mutable>>(
    mut commands: Commands,
    free_nodes: Query<Entity, (With<Node>, With<Disabled>)>,
    mut spawns: MessageReader<SpawnNode<S, Node>>,
    transforms: Query<&S::GlobalTransform>,
    hierarchy: Query<&ChildOf>,
    all_nodes: Query<One<&dyn ProceduralNode<S>>>,
) {
    let mut reader = spawns.read().peekable();
    let mut free_nodes = free_nodes.iter().peekable();
    while reader.peek().is_some() && free_nodes.peek().is_some() {
        let message = reader.next().unwrap();
        let free_entity = free_nodes.next().unwrap();
        let global_transform = if let Some(parent) = message.parent() {
            commands.entity(parent).add_child(free_entity);
            transforms.get(parent).unwrap()
        } else {
            &S::GlobalTransform::default()
        };
            .retain::<Node>()
            .insert(Generate);

        commands.entity_mut(free_entity);

        // let ancestor_entities = collect_ancestor_entities(message.parent(), &hierarchy);
        // let ancestor_nodes: Vec<_> = all_nodes.iter_many(&ancestor_entities).collect();
        // let provider = build_provider_from_refs(&ancestor_nodes);
        // task_pool.spawn(async {
        //     free_node.generate(global_transform, &provider);
        // });
        // let task = task_pool.spawn(async { free_node.generate(message.local_transform, Provider::new()) });
        // commands
        //     .entity(free_entity)
        //     .insert(GenerateTask::<S, Node>::new(task));
    }
}

// fn poll_generation<S: Space, Node: ProceduralNode<S> + Component>(
//     mut commands: Commands,
//     nodes: Query<(Entity, &GenerateTask<S, Node>)>,
// ) {
//     for (entity, mut generate_task) in nodes {
//         if let Some(node_data) = generate_task
//     }
// }

pub struct NodeList<S: Space> {
    nodes: Vec<(TypeId, S::LocalTransform)>,
}

impl<S: Space> NodeList<S> {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add<Node: ProceduralNode<S>>(&mut self, transform: S::LocalTransform) {
        self.nodes.push((TypeId::of::<Node>(), transform));
    }

    pub fn iter(&self) -> impl Iterator<Item = &(TypeId, S::LocalTransform)> {
        self.nodes.iter()
    }
}

#[derive(Resource)]
pub(crate) struct Allocations<S: Space> {
    potential_allocations: HashMap<TypeId, u64>,
    space_phantom_data: PhantomData<S>,
}

impl<S: Space> Allocations<S> {
    pub(crate) fn new() -> Self {
        Self {
            potential_allocations: HashMap::new(),
            space_phantom_data: PhantomData,
        }
    }

    pub(crate) fn queue(&mut self, list: NodeList<S>) {}
}
