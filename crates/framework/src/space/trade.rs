use crate::ProceduralNode;

use super::{Space, Viewer};
use bevy::{
    ecs::system::QueryLens,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use bevy_trait_query::One;
use std::{cmp::Ordering, marker::PhantomData};

#[derive(Resource)]
pub(crate) struct Trade<S: Space> {
    memory_limit: usize,
    memory_budget: usize,
    memory_pool: usize,
    pool_budget: usize,
    items: Vec<Entity>,
    priorities: HashMap<Entity, f32>,
    collapse_buffer: HashSet<Entity>,
    space_phantom_data: PhantomData<S>,
}

impl<S: Space> Trade<S> {
    fn new(memory_limit: usize, memory_pool: usize) -> Self {
        Self {
            memory_limit,
            memory_budget: memory_limit,
            memory_pool,
            pool_budget: memory_pool,
            items: Vec::new(),
            priorities: HashMap::new(),
            collapse_buffer: HashSet::new(),
            space_phantom_data: PhantomData,
        }
    }

    fn refresh(
        &mut self,
        nodes: QueryLens<(Entity, &S::Transform)>,
        viewers: QueryLens<(&Viewer<S>, &S::Transform)>,
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

    fn trade(
        &mut self,
        mut commands: Commands,
        leaves: QueryLens<(Entity, One<&dyn ProceduralNode<S>>, &S::Transform)>,
        hierarchy: QueryLens<(Entity, &Children)>,
    ) {
        let subdivide = subdivide.query_inner();
        let mut collapse = collapse.query_inner().iter();
        let mut collapse_buffer = vec![];
        for item in self.items.iter().rev() {
            let (subdivider, subdivide_node, subdivide_transform) = subdivide.get(*item).unwrap();

            while subdivide_size > self.budget {
                if let Some((collapser, collapse_node)) = collapse.next() {
                    self.budget += size_of_val(collapse_node.into_inner());
                    collapse_buffer.push(collapser);
                } else {
                    return;
                }
            }

            for collapser in collapse_buffer {
                commands.entity(collapser).despawn_children();
            }

            subdivide_node.subdivide();

            self.budget -= subdivide_size;
        }
    }
}
