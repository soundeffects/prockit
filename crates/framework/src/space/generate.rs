use super::Space;
use crate::{ProceduralNode, Provider, Provides};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on},
};
use bevy_trait_query::One;
use paste::paste;

/// A description of one procedural node as the subdivision of another procedural node with a
/// transform relative to the parent node.
///
/// # Example
/// ```
/// # use prockit_framework::{
/// #   Subdivision, RealSpace, ProceduralNode, Subdivisions, Provider, Provides
/// # };
/// # use bevy::prelude::*;
/// #
/// #[derive(Component, Clone)]
/// struct Node;
///
/// # impl ProceduralNode<RealSpace> for Node {
/// #     fn provides(&self, _: &mut Provides<RealSpace>) {}
/// #     fn subdivide(&self) -> Option<Subdivisions<RealSpace>> { None }
/// #     fn init() -> Self { Node }
/// #     fn generate(&mut self, _: &GlobalTransform, _: &Provider<RealSpace>) {}
/// # }
///
/// let subdivision = Subdivision::<RealSpace, Node>::new(
///     Transform::from_translation(Vec3::new(1.0, 0.0, 0.0))
/// );
/// ```
pub struct Subdivision<S: Space, T: ProceduralNode<S> + Component + Clone> {
    transform: S::LocalTransform,
    node: Option<T>,
}

impl<S: Space, T: ProceduralNode<S> + Component + Clone> Subdivision<S, T> {
    /// Describes a new subdivision with the given local transform.
    ///
    /// # Example
    /// ```
    /// # use prockit_framework::{
    /// #   Subdivision, RealSpace, ProceduralNode, Subdivisions, Provider, Provides
    /// # };
    /// # use bevy::prelude::*;
    /// # #[derive(Component, Clone)]
    /// # struct Node;
    /// # impl ProceduralNode<RealSpace> for Node {
    /// #     fn provides(&self, _: &mut Provides<RealSpace>) {}
    /// #     fn subdivide(&self) -> Option<Subdivisions<RealSpace>> { None }
    /// #     fn init() -> Self { Node }
    /// #     fn generate(&mut self, _: &GlobalTransform, _: &Provider<RealSpace>) {}
    /// # }
    /// #
    /// let subdivision = Subdivision::<RealSpace, Node>::new(
    ///     Transform::from_translation(Vec3::new(1.0, 0.0, 0.0))
    /// );
    /// ```
    pub fn new(transform: S::LocalTransform) -> Self {
        Self {
            transform,
            node: None,
        }
    }

    /// Generates the child node using the parent's global transform and provider.
    fn inner_generate(&mut self, parent_transform: &S::GlobalTransform, provider: &Provider<S>) {
        let transform = S::push_transform(parent_transform, &self.transform);
        let mut node = T::init();
        node.generate(&transform, provider);
        self.node = Some(node);
    }

    /// Creates a bundle containing this subdivision's transform and generated node
    /// for spawning as an entity.
    fn bundle(&self) -> impl Bundle {
        (self.transform.clone(), self.node.clone().unwrap())
    }
}

/// This trait allows collections of different subdivision types to be stored
/// together and processed uniformly by the generation system.
trait Generate<S: Space>: Send + Sync + 'static {
    /// Spawns the generated subdivision(s) as children of the given entity.
    fn spawn(&self, entity_commands: &mut EntityCommands);

    /// Generates all subdivisions using the parent's transform and provider.
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

/// `Subdivisions` allows a procedural node to return any number of child subdivisions
/// (from 1 to 8) of potentially different node types.
///
/// # Example
/// ```
/// # use prockit_framework::{Subdivision, Subdivisions, RealSpace, ProceduralNode, Provider, Provides};
/// # use bevy::prelude::*;
///
/// #[derive(Component, Clone)]
/// struct Node;
///
/// #[derive(Component, Clone)]
/// struct OtherNode;
///
/// # impl ProceduralNode<RealSpace> for Node {
/// #     fn provides(&self, _: &mut Provides<RealSpace>) {}
/// #     fn subdivide(&self) -> Option<Subdivisions<RealSpace>> { None }
/// #     fn init() -> Self { Node }
/// #     fn generate(&mut self, _: &GlobalTransform, _: &Provider<RealSpace>) {}
/// # }
///
/// # impl ProceduralNode<RealSpace> for OtherNode {
/// #     fn provides(&self, _: &mut Provides<RealSpace>) {}
/// #     fn subdivide(&self) -> Option<Subdivisions<RealSpace>> { None }
/// #     fn init() -> Self { OtherNode }
/// #     fn generate(&mut self, _: &GlobalTransform, _: &Provider<RealSpace>) {}
/// # }
///
/// // Single subdivision
/// let single: Subdivisions<RealSpace> = Subdivision::<_, Node>::new(
///     Transform::from_translation(Vec3::X)
/// ).into();
///
/// // Multiple subdivisions (as tuple)
/// let multiple: Subdivisions<RealSpace> = (
///     Subdivision::<_, Node>::new(Transform::from_translation(Vec3::X)),
///     Subdivision::<_, OtherNode>::new(Transform::from_translation(Vec3::NEG_X)),
/// ).into();
/// ```
pub struct Subdivisions<S: Space> {
    generate: Box<dyn Generate<S>>,
}

impl<S: Space> Subdivisions<S> {
    /// Generates all child nodes in this collection and returns the boxed generator
    /// for spawning.
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

/// Marker component indicating that an entity should be subdivided and its subdivisions
/// generated by the [`FrameworkPlugin`], which will assign an asynchronous generation task
/// at the next opportunity.
#[derive(Component)]
pub struct PendingGenerate;

/// Marker component for leaf nodes that cannot subdivide further. Applied to nodes whose
/// [`ProceduralNode::subdivide`] returns `None`, indicating they are at maximum detail level.
#[derive(Component)]
pub(crate) struct EmptyNode;

/// This component is attached to entities while their children are being generated
/// in the background using Bevy's async compute task pool. Once the task completes,
/// the generated children are spawned and this component is removed.
#[derive(Component)]
pub(crate) struct GenerateTask<S: Space> {
    task: Task<Box<dyn Generate<S>>>,
}

impl<S: Space> GenerateTask<S> {
    /// System that creates async generation tasks for entities with [`PendingGenerate`]. It
    /// will handle transform and [`Provider`] propogation. For nodes that return `None` from
    /// `subdivide()`, an [`EmptyNode`] marker is added instead.
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

        println!("found tasks: {}", pending_tasks.count());
        for (entity, node, transform) in pending_tasks {
            println!("found task");
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

    /// System that polls pending generation tasks and spawns completed children.
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

#[cfg(test)]
mod tests {
    use std::any::Any;

    use bevy::tasks::TaskPool;
    use bevy_trait_query::RegisterExt;

    use super::*;
    use crate::RealSpace;

    #[derive(Component, Clone, Debug, PartialEq)]
    struct TestNode {
        value: i32,
    }

    impl ProceduralNode<RealSpace> for TestNode {
        fn provides(&self, _instance: &mut Provides<RealSpace>) {}
        fn subdivide(&self) -> Option<Subdivisions<RealSpace>> {
            None
        }
        fn init() -> Self {
            TestNode { value: 0 }
        }
        fn generate(&mut self, _transform: &GlobalTransform, _provider: &Provider<RealSpace>) {
            self.value = 42;
        }
    }

    #[test]
    fn test_subdivision_new_creates_ungenerated_subdivision() {
        let subdivision = Subdivision::<RealSpace, TestNode>::new(Transform::from_translation(
            Vec3::new(1.0, 2.0, 3.0),
        ));

        // Node should be None before generation
        assert!(subdivision.node.is_none());
    }

    #[test]
    fn test_subdivision_inner_generate_populates_node() {
        let mut subdivision =
            Subdivision::<RealSpace, TestNode>::new(Transform::from_translation(Vec3::X));

        let parent_transform = GlobalTransform::IDENTITY;
        let provider = Provider::<RealSpace>::empty();

        subdivision.inner_generate(&parent_transform, &provider);

        assert!(subdivision.node.is_some());
        assert_eq!(subdivision.node.as_ref().unwrap().value, 42);
    }

    #[test]
    fn test_subdivisions_from_single_subdivision() {
        let subdivision =
            Subdivision::<RealSpace, TestNode>::new(Transform::from_translation(Vec3::X));
        let _subdivisions: Subdivisions<RealSpace> = subdivision.into();
    }

    #[test]
    fn test_subdivisions_from_tuple_of_two() {
        let sub1 = Subdivision::<RealSpace, TestNode>::new(Transform::from_translation(Vec3::X));
        let sub2 =
            Subdivision::<RealSpace, TestNode>::new(Transform::from_translation(Vec3::NEG_X));
        let _subdivisions: Subdivisions<RealSpace> = (sub1, sub2).into();
    }

    #[test]
    fn test_subdivisions_from_tuple_of_three() {
        let sub1 = Subdivision::<RealSpace, TestNode>::new(Transform::from_translation(Vec3::X));
        let sub2 = Subdivision::<RealSpace, TestNode>::new(Transform::from_translation(Vec3::Y));
        let sub3 = Subdivision::<RealSpace, TestNode>::new(Transform::from_translation(Vec3::Z));
        let _subdivisions: Subdivisions<RealSpace> = (sub1, sub2, sub3).into();
    }

    #[test]
    fn test_subdivisions_from_single_element_tuple() {
        let subdivision =
            Subdivision::<RealSpace, TestNode>::new(Transform::from_translation(Vec3::X));
        let _subdivisions: Subdivisions<RealSpace> = (subdivision,).into();
    }

    #[test]
    fn test_generate_trait_single_subdivision() {
        let mut subdivision =
            Subdivision::<RealSpace, TestNode>::new(Transform::from_translation(Vec3::X));

        let parent_transform = GlobalTransform::IDENTITY;
        let provider = Provider::<RealSpace>::empty();

        Generate::generate(&mut subdivision, &parent_transform, &provider);

        assert!(subdivision.node.is_some());
        assert_eq!(subdivision.node.as_ref().unwrap().value, 42);
    }

    #[test]
    fn test_generate_trait_tuple_of_subdivisions() {
        let mut subdivisions = (
            Subdivision::<RealSpace, TestNode>::new(Transform::from_translation(Vec3::X)),
            Subdivision::<RealSpace, TestNode>::new(Transform::from_translation(Vec3::NEG_X)),
        );

        let parent_transform = GlobalTransform::IDENTITY;
        let provider = Provider::<RealSpace>::empty();

        Generate::generate(&mut subdivisions, &parent_transform, &provider);

        assert!(subdivisions.0.node.is_some());
        assert!(subdivisions.1.node.is_some());
        assert_eq!(subdivisions.0.node.as_ref().unwrap().value, 42);
        assert_eq!(subdivisions.1.node.as_ref().unwrap().value, 42);
    }

    #[derive(Component, Clone, Debug)]
    struct CountingNode {
        count: i32,
    }

    impl ProceduralNode<RealSpace> for CountingNode {
        fn provides(&self, instance: &mut Provides<RealSpace>) {
            let count = self.count;
            instance.add("count", move |_location| count);
        }

        fn subdivide(&self) -> Option<Subdivisions<RealSpace>> {
            if self.count < 2 {
                Some(
                    Subdivision::<RealSpace, CountingNode>::new(Transform::from_translation(
                        Vec3::X,
                    ))
                    .into(),
                )
            } else {
                None
            }
        }

        fn init() -> Self {
            CountingNode { count: 0 }
        }

        fn generate(&mut self, _transform: &GlobalTransform, provider: &Provider<RealSpace>) {
            let count = provider.query::<i32>("count").unwrap();
            self.count = count(&Vec3::ZERO) + 1;
        }
    }
    //
    // fn test_system(
    //     mut commands: Commands,
    //     query: Query<(Entity, One<&dyn ProceduralNode<RealSpace>>)>,
    // ) {
    //     for (entity, _) in query {
    //         commands.entity(entity).insert(EmptyNode);
    //     }
    // }
    //
    // #[test]
    // fn app_testing() {
    //     let mut app = App::new();
    //     app.register_component_as::<dyn ProceduralNode<RealSpace>, TestNode>()
    //         .add_systems(Update, test_system);
    //     AsyncComputeTaskPool::get_or_init(|| TaskPool::new());
    //
    //     let test_entity = app.world_mut().spawn(TestNode { value: 0 }).id();
    //     app.update();
    //     let empty_result = app.world().get::<EmptyNode>(test_entity);
    //     assert!(empty_result.is_some());
    // }

    #[test]
    fn test_create_task_empty_node() {
        let mut app = App::new();
        app.register_component_as::<dyn ProceduralNode<RealSpace>, CountingNode>()
            .add_systems(Update, GenerateTask::<RealSpace>::create_tasks);

        AsyncComputeTaskPool::get_or_init(|| TaskPool::new());

        let pending_entity = app
            .world_mut()
            .spawn((
                TestNode { value: 0 },
                GlobalTransform::default(),
                PendingGenerate,
            ))
            .id();
        for _ in 0..10 {
            app.update();
        }
        // app.world_mut().flush();

        let pending_result = app.world().get::<PendingGenerate>(pending_entity);
        assert!(pending_result.is_none());

        let empty_result = app.world().get::<EmptyNode>(pending_entity);
        assert!(empty_result.is_some());

        let task_result = app.world().get::<GenerateTask<RealSpace>>(pending_entity);
        assert!(task_result.is_none());
    }

    // #[test]
    // fn test_create_single_task() {
    //     let mut app = App::new();
    //     app.add_systems(Update, GenerateTask::<RealSpace>::create_tasks);
    //     AsyncComputeTaskPool::get_or_init(|| TaskPool::new());
    //
    //     let task_entity = app
    //         .world_mut()
    //         .spawn((
    //             CountingNode { count: 0 },
    //             GlobalTransform::default(),
    //             PendingGenerate,
    //         ))
    //         .id();
    //
    //     app.update();
    //     app.world_mut().flush();
    //
    //     let world = app.world_mut();
    //     let task_result = world.get_mut::<GenerateTask<RealSpace>>(task_entity);
    //     assert!(task_result.is_some());
    //
    //     let generate = block_on(&mut task_result.unwrap().into_inner().task);
    //     let expected_type: Box<dyn Generate<RealSpace>> =
    //         Box::new(Subdivision::<RealSpace, CountingNode>::new(
    //             Transform::default(),
    //         ));
    //     assert_eq!(generate.type_id(), expected_type.type_id());
    //
    //     generate.spawn(&mut world.commands().entity(task_entity));
    //     world.flush();
    //
    //     let children_result = app.world().get::<Children>(task_entity);
    //     assert!(children_result.is_some());
    //
    //     let child_result = children_result.unwrap().first();
    //     assert!(child_result.is_some());
    //
    //     let child_node = app.world().get::<CountingNode>(*child_result.unwrap());
    //     assert!(child_node.is_some());
    //     assert_eq!(child_node.unwrap().count, 1);
    //
    //     let pending_result = app.world().get::<PendingGenerate>(task_entity);
    //     assert!(pending_result.is_none());
    // }

    #[test]
    fn test_poll_tasks() {}
}
