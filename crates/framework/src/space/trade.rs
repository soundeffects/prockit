use super::{Space, Viewer};
use crate::ProceduralNode;
use bevy::{
    ecs::system::QueryLens,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use std::{any::TypeId, cmp::Ordering, marker::PhantomData};

/// List of child nodes to be spawned during subdivision.
pub struct NodeList<S: Space> {
    nodes: Vec<(TypeId, S::LocalTransform)>,
}

impl<S: Space> NodeList<S> {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add<Node: ProceduralNode<S> + 'static>(&mut self, transform: S::LocalTransform) {
        self.nodes.push((TypeId::of::<Node>(), transform));
    }

    pub fn iter(&self) -> impl Iterator<Item = &(TypeId, S::LocalTransform)> {
        self.nodes.iter()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl<S: Space> Default for NodeList<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Resource)]
pub(crate) struct Trade<S: Space> {
    memory_limit: usize,
    memory_budget: usize,
    items: Vec<Entity>,
    priorities: HashMap<Entity, f32>,
    collapse_buffer: HashSet<Entity>,
    space_phantom_data: PhantomData<S>,
}

impl<S: Space> Trade<S> {
    fn new(memory_limit: usize) -> Self {
        Self {
            memory_limit,
            memory_budget: memory_limit,
            items: Vec::new(),
            priorities: HashMap::new(),
            collapse_buffer: HashSet::new(),
            space_phantom_data: PhantomData,
        }
    }

    fn refresh(
        &mut self,
        nodes: QueryLens<(Entity, &S::GlobalTransform)>,
        viewers: QueryLens<(&Viewer<S>, &S::GlobalTransform)>,
    ) {
        let nodes = nodes.query_inner();
        let viewers = viewers.query_inner();
        self.items.retain(|entity| nodes.contains(*entity));
        self.priorities.retain(|entity, _| nodes.contains(*entity));

        for (entity, transform) in nodes {
            if !self.priorities.contains_key(&entity) {
                self.items.push(entity);
            }

            self.priorities.insert(
                entity,
                viewers
                    .iter()
                    .map(|(viewer, viewer_transform)| {
                        S::noticeability(transform, viewer_transform) * viewer.priority()
                    })
                    .sum::<f32>(),
            );
        }

        self.items.sort_unstable_by(|entity_a, entity_b| {
            self.priorities
                .get(entity_a)
                .unwrap()
                .partial_cmp(self.priorities.get(entity_b).unwrap())
                .unwrap_or(Ordering::Equal)
        })
    }

    // TODO: Implement trade logic - currently has undefined variables
    // fn trade(
    //     &mut self,
    //     mut commands: Commands,
    //     leaves: QueryLens<(Entity, One<&dyn ProceduralNode<S>>, &S::GlobalTransform)>,
    //     hierarchy: QueryLens<(Entity, &Children)>,
    // ) {
    //     let subdivide = subdivide.query_inner();
    //     let mut collapse = collapse.query_inner().iter();
    //     let mut collapse_buffer = vec![];
    //     for item in self.items.iter().rev() {
    //         let (subdivider, subdivide_node, subdivide_transform) = subdivide.get(*item).unwrap();
    //
    //         while subdivide_size > self.budget {
    //             if let Some((collapser, collapse_node)) = collapse.next() {
    //                 self.budget += size_of_val(collapse_node.into_inner());
    //                 collapse_buffer.push(collapser);
    //             } else {
    //                 return;
    //             }
    //         }
    //
    //         for collapser in collapse_buffer {
    //             commands.entity(collapser).despawn_children();
    //         }
    //
    //         subdivide_node.subdivide();
    //
    //         self.budget -= subdivide_size;
    //     }
    // }
}
