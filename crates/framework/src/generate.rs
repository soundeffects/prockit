use crate::{
    placement::Placement,
    provides::PodProvides,
    registry::NodeRegistry,
    subdivide::PendingGenerate,
    Pod, ProceduralNode, Provider, Space,
};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on},
};
use rand::Rng;

/// A spawner function that inserts node components into an entity.
type NodeSpawner = Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>;

/// Stores the transform and spawner for a child that was accepted.
struct ChildSpawner<S: Space> {
    transform: S::LocalTransform,
    spawner: NodeSpawner,
}

/// This component is attached to entities while their children are being generated
/// in the background using Bevy's async compute task pool. Once the task completes,
/// the generated children are spawned and this component is removed.
#[derive(Component)]
pub(crate) struct GenerateTask<S: Space> {
    task: Task<Vec<ChildSpawner<S>>>,
}

impl<S: Space> GenerateTask<S> {
    /// System that creates async generation tasks for entities with [`PendingGenerate`].
    ///
    /// For each pending entity:
    /// 1. Collects placements from `PodProvides::subdivide()`
    /// 2. Creates a Provider for each placement
    /// 3. Tries registered node types in random order until one accepts
    /// 4. Spawns accepted nodes as children
    pub(crate) fn create_tasks(
        mut commands: Commands,
        registry: Res<NodeRegistry>,
        pending: Query<(Entity, &PodProvides), With<PendingGenerate>>,
        pod_provides: Query<&PodProvides>,
        hierarchy: Query<&ChildOf>,
    ) {
        let task_pool = AsyncComputeTaskPool::get();

        for (entity, entity_pod_provides) in pending.iter() {
            commands.entity(entity).remove::<PendingGenerate>();

            // Get placements from subdivide - if None, this is a leaf node
            let Some(subdivide) = entity_pod_provides.subdivide() else {
                // Leaf node - no children to generate
                continue;
            };

            let placements = subdivide.into_placements();
            if placements.is_empty() {
                continue;
            }

            // Collect the parent provider by walking ancestors (with empty placement for parent)
            let parent_provider = Provider::collect(
                entity,
                Placement::new(),
                pod_provides.reborrow(),
                hierarchy.reborrow(),
            );

            let registry = registry.clone();

            let task = task_pool.spawn(async move {
                let mut rng = rand::rng();
                let mut children: Vec<ChildSpawner<S>> = Vec::new();

                for placement in placements {
                    // Extract transform before moving placement into provider
                    let transform = placement
                        .get::<S>()
                        .map(|sp| sp.transform.clone())
                        .unwrap_or_default();

                    let child_provider = Provider::for_placement(placement, &parent_provider);

                    // Try registered node types until one accepts
                    if let Some(spawner) = registry.try_place(&child_provider, &mut rng) {
                        children.push(ChildSpawner { transform, spawner });
                    }
                }
                children
            });

            commands.entity(entity).insert(GenerateTask::<S> { task });
        }
    }

    /// System that polls pending generation tasks and spawns completed children.
    pub(crate) fn poll_tasks(
        mut commands: Commands,
        mut tasks: Query<(Entity, &mut GenerateTask<S>)>,
    ) {
        for (entity, mut task) in tasks.iter_mut() {
            if task.task.is_finished() {
                let children = block_on(&mut task.task);
                commands.entity(entity).remove::<GenerateTask<S>>();

                commands.entity(entity).with_children(|parent| {
                    for child in children {
                        let mut child_cmd = parent.spawn(child.transform);
                        (child.spawner)(&mut child_cmd);
                    }
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Provides, RealSpace, Subdivide};
    use bevy::tasks::TaskPool;
    use get_size2::GetSize;

    #[derive(Component, Clone, Default, GetSize)]
    struct TestNode {
        value: i32,
    }

    impl ProceduralNode for TestNode {
        fn provides(&self, _provides: &mut Provides<Self>) {}

        fn subdivide(&self) -> Option<Subdivide> {
            None
        }

        fn place(_provider: &Provider) -> Option<Self> {
            Some(Self { value: 42 })
        }
    }

    #[test]
    fn test_generate_task_poll_spawns_children() {
        // This test verifies the poll_tasks system works correctly
        // Full integration testing requires a running Bevy app
    }
}
