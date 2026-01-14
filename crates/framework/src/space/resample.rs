use super::{EmptyNode, PendingGenerate, Space};
use crate::ProceduralNode;
use bevy::{platform::collections::HashSet, prelude::*};
use bevy_trait_query::One;
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Component)]
pub struct Viewer<S: Space> {
    priority: f32,
    space_phantom_data: PhantomData<S>,
}

impl<S: Space> Viewer<S> {
    pub fn new(priority: f32) -> Self {
        Self {
            priority,
            space_phantom_data: PhantomData,
        }
    }

    pub fn priority(&self) -> f32 {
        self.priority
    }
}

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
    pub(crate) const HISTORY_SIZE: usize = 16;

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

    pub(crate) fn add(&mut self, amount: usize) {
        self.current += amount;
    }

    pub(crate) fn sub(&mut self, amount: usize) {
        self.current -= amount;
    }

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
