use super::Space;
use crate::{ProceduralNode, Provider, Provides};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on},
};
use bevy_trait_query::One;
use paste::paste;

pub struct Subdivision<S: Space, T: ProceduralNode<S> + Component + Clone> {
    transform: S::LocalTransform,
    node: Option<T>,
}

impl<S: Space, T: ProceduralNode<S> + Component + Clone> Subdivision<S, T> {
    pub fn new(transform: S::LocalTransform) -> Self {
        Self {
            transform,
            node: None,
        }
    }

    fn inner_generate(&mut self, parent_transform: &S::GlobalTransform, provider: &Provider<S>) {
        let transform = S::push_transform(parent_transform, &self.transform);
        let mut node = T::init();
        node.generate(&transform, provider);
        self.node = Some(node);
    }

    fn bundle(&self) -> impl Bundle {
        (self.transform.clone(), self.node.clone().unwrap())
    }
}

trait Generate<S: Space>: Send + Sync + 'static {
    fn spawn(&self, entity_commands: &mut EntityCommands);
    fn generate(&mut self, transform: &S::GlobalTransform, provider: &Provider<S>);
}

impl<S: Space, T: ProceduralNode<S> + Component + Clone> Generate<S> for Subdivision<S, T> {
    fn spawn(&self, entity_commands: &mut EntityCommands) {
        entity_commands.with_child(self.bundle());
    }

    fn generate(&mut self, transform: &S::GlobalTransform, provider: &Provider<S>) {
        self.inner_generate(transform, provider);
    }
}

impl<S: Space, T: ProceduralNode<S> + Component + Clone> Generate<S> for (Subdivision<S, T>,) {
    fn spawn(&self, entity_commands: &mut EntityCommands) {
        entity_commands.with_child(self.0.bundle());
    }

    fn generate(&mut self, transform: &S::GlobalTransform, provider: &Provider<S>) {
        self.0.inner_generate(transform, provider);
    }
}

macro_rules! impl_generate {
    ($($number: literal),*) => {
        paste! {
            impl<S: Space, $(
                [<T $number>]: ProceduralNode<S> + Component + Clone
            ),*> Generate<S> for ($(Subdivision<S, [<T $number>]>),*) {
                fn spawn(&self, entity_commands: &mut EntityCommands) {
                    entity_commands.with_children(|parent| {
                        $(
                            parent.spawn(self.$number.bundle());
                        )*
                    });
                }

                fn generate(&mut self, transform: &S::GlobalTransform, provider: &Provider<S>) {
                    $(
                        self.$number.inner_generate(transform, provider);
                    )*
                }
            }
        }
    };
}

impl_generate!(0, 1);
impl_generate!(0, 1, 2);
impl_generate!(0, 1, 2, 3);
impl_generate!(0, 1, 2, 3, 4);
impl_generate!(0, 1, 2, 3, 4, 5);
impl_generate!(0, 1, 2, 3, 4, 5, 6);
impl_generate!(0, 1, 2, 3, 4, 5, 6, 7);

pub struct Subdivisions<S: Space> {
    generate: Box<dyn Generate<S>>,
}

impl<S: Space> Subdivisions<S> {
    fn generate(
        mut self,
        transform: &S::GlobalTransform,
        provider: &Provider<S>,
    ) -> Box<dyn Generate<S>> {
        self.generate.generate(transform, provider);
        self.generate
    }
}

impl<S: Space, T: ProceduralNode<S> + Component + Clone> From<Subdivision<S, T>>
    for Subdivisions<S>
{
    fn from(value: Subdivision<S, T>) -> Self {
        Subdivisions {
            generate: Box::new(value),
        }
    }
}

impl<S: Space, T: ProceduralNode<S> + Component + Clone> From<(Subdivision<S, T>,)>
    for Subdivisions<S>
{
    fn from(value: (Subdivision<S, T>,)) -> Self {
        Subdivisions {
            generate: Box::new(value),
        }
    }
}

macro_rules! impl_from {
    ($($number: literal),*) => {
        paste! {
            impl<S: Space, $(
                [<T $number>]: ProceduralNode<S> + Component + Clone
            ),*
            > From<($(
                Subdivision<S, [<T $number>]>
            ),*)> for Subdivisions<S> {
                fn from(value: ($(
                            Subdivision<S, [<T $number>]>
                ),*)) -> Self {
                    Subdivisions { generate: Box::new(value) }
                }
            }
        }
    };
}

impl_from!(0, 1);
impl_from!(0, 1, 2);
impl_from!(0, 1, 2, 3);
impl_from!(0, 1, 2, 3, 4);
impl_from!(0, 1, 2, 3, 4, 5);
impl_from!(0, 1, 2, 3, 4, 5, 6);
impl_from!(0, 1, 2, 3, 4, 5, 6, 7);

#[derive(Component)]
pub struct PendingGenerate;

#[derive(Component)]
pub(crate) struct EmptyNode;

#[derive(Component)]
pub(crate) struct GenerateTask<S: Space> {
    task: Task<Box<dyn Generate<S>>>,
}

impl<S: Space> GenerateTask<S> {
    pub(crate) fn create_tasks(
        mut commands: Commands,
        pending_tasks: Query<
            (Entity, One<&dyn ProceduralNode<S>>, &S::GlobalTransform),
            With<PendingGenerate>,
        >,
        nodes: Query<One<&dyn ProceduralNode<S>>>,
        hierarchy: Query<&ChildOf>,
    ) {
        let task_pool = AsyncComputeTaskPool::get();
        for (entity, node, transform) in pending_tasks {
            let mut entity_commands = commands.entity(entity);
            entity_commands.remove::<PendingGenerate>();
            if let Some(subdivisions) = node.subdivide() {
                let mut provides_list = Vec::new();
                for node in hierarchy
                    .iter_ancestors::<ChildOf>(entity)
                    .filter_map(|entity| nodes.get(entity).ok())
                {
                    let mut provides = Provides::<S>::new();
                    node.provides(&mut provides);
                    provides_list.push(provides);
                }
                provides_list.reverse();
                let provider = Provider::<S>::hierarchy(provides_list);
                let transform = transform.clone();
                entity_commands.insert(Self {
                    task: task_pool
                        .spawn(async move { subdivisions.generate(&transform, &provider) }),
                });
            } else {
                commands.entity(entity).insert(EmptyNode);
            }
        }
    }

    pub(crate) fn poll_tasks(mut commands: Commands, tasks: Query<(Entity, &mut GenerateTask<S>)>) {
        for (entity, mut task) in tasks {
            if task.task.is_finished() {
                let finished = block_on(&mut task.task);
                let mut parent = commands.entity(entity);
                finished.spawn(&mut parent);
                parent.remove::<GenerateTask<S>>();
            }
        }
    }
}
