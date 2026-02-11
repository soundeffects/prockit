use super::{NameQuery, Names, Pod, ProceduralNode, Space};
use crate::placement::{Placement, SpacePlacement};
use bevy::{
    prelude::*,
    utils::{TypeIdMap, TypeIdMapExt},
};
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

/// A collection of spatial sampling functions that can be looked up by function name and type,
/// created from ancestor [`ProceduralNode`]s by the [`prockit_framework`] crate automatically.
/// An instantiated `Provider` is given to the child [`ProceduralNode`] during the call to
/// [`ProceduralNode::place`].
///
/// The `Provider` also stores the [`Placement`] that created it, allowing nodes to access
/// placement information for any space they operate in.
///
/// # Examples
/// ```
/// # use prockit_framework::{Provider, RealSpace};
/// # use bevy::prelude::*;
/// fn accept_placement(provider: &Provider) -> bool {
///     // Check placement data for RealSpace
///     if let Some(space_placement) = provider.space_placement::<RealSpace>() {
///         space_placement.detail_scale > 0.1
///     } else {
///         false
///     }
/// }
///
/// fn query_ancestor(provider: &Provider) -> i32 {
///     let count = provider.query::<RealSpace, i32>("count").unwrap();
///     count(&Vec3::ZERO) + 1
/// }
/// ```
pub struct Provider {
    entries: Vec<(Names, SpaceType, ReturnType, ErasedClosure)>,
    transforms: TypeIdMap<ErasedTransform>,
    placement: Placement,
}

impl Provider {
    /// Create a provider for a placement, inheriting samplers from a parent provider.
    ///
    /// This is the primary way to create a `Provider` for child nodes during the
    /// generation process.
    pub fn for_placement(placement: Placement, parent: &Provider) -> Self {
        Self {
            entries: parent.entries.clone(),
            transforms: parent.transforms.clone(),
            placement,
        }
    }

    /// Create an empty root provider with no placement data.
    ///
    /// Used for root nodes that have no parent placement.
    pub fn root() -> Self {
        Self {
            entries: vec![],
            transforms: TypeIdMap::default(),
            placement: Placement::new(),
        }
    }

    /// Access the placement that created this provider.
    pub fn placement(&self) -> &Placement {
        &self.placement
    }

    /// Convenience method to get space-specific placement data.
    ///
    /// # Example
    /// ```
    /// # use prockit_framework::{Provider, RealSpace};
    /// fn check_detail(provider: &Provider) -> bool {
    ///     if let Some(space_placement) = provider.space_placement::<RealSpace>() {
    ///         space_placement.detail_scale > 0.1
    ///     } else {
    ///         false
    ///     }
    /// }
    /// ```
    pub fn space_placement<S: Space>(&self) -> Option<&SpacePlacement<S>> {
        self.placement.get::<S>()
    }

    /// Add a named sampler, expecting the data the sampler is accessing to be curried in the
    /// first argument using an environment-capturing closure.
    fn add<S: Space, R: 'static>(
        &mut self,
        names: Names,
        function: impl Fn(&S::Position) -> R + 'static,
    ) {
        self.entries.push((
            names,
            TypeId::of::<S>(),
            TypeId::of::<R>(),
            Box::new(function),
        ));
    }

    /// Queries for a sampling function of the specified space (specifying the spatial positions
    /// the function should receive as input), return type, and names.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provides, NameQuery, RealSpace, Provider};
    /// # use bevy::prelude::*;
    /// fn generate(provider: &Provider) {
    ///     let result = provider.query::<RealSpace, f32>(NameQuery::exact("opacity"));
    ///     if let Some(sampler) = result {
    ///         // do something
    ///     }
    /// }
    /// ```
    pub fn query<SpaceType: Space, ReturnType: 'static>(
        &self,
        name_query: impl Into<NameQuery>,
    ) -> Option<&dyn Fn(&SpaceType::Position) -> ReturnType> {
        let name_query = name_query.into();
        self.entries
            .iter()
            .find(|(names, space_type, return_type, _closure)| {
                *space_type == TypeId::of::<SpaceType>()
                    && *return_type == TypeId::of::<ReturnType>()
                    && name_query.matches(names)
            })
            .and_then(|(_names, _space_type, _return_type, closure)| {
                closure.downcast_ref::<Box<dyn Fn(&SpaceType::Position) -> ReturnType>>()
            })
            .map(|boxed| boxed.as_ref())
    }

    /// Collect the samplers from all ancestor `ProceduralNode`s to the one specified by
    /// the `entity` argument into a new `Provider`.
    pub(crate) fn collect(
        entity: Entity,
        placement: Placement,
        pod_provides: Query<&PodProvides>,
        hierarchy: Query<&ChildOf>,
    ) -> Self {
        let mut provider = Provider {
            entries: vec![],
            transforms: TypeIdMap::default(),
            placement,
        };

        // TODO: collect transforms
        for pod_provides in hierarchy
            .iter_ancestors(entity)
            .filter_map(|entity| pod_provides.get(entity).ok())
        {
            pod_provides.curry(&mut provider);
        }

        provider
    }
}

/// An interface by which a [`ProceduralNode`] exports the spatial sampling functions that can
/// be called on it. These functions are later collected into a [`Provider`], which is given to
/// any child [`ProceduralNode`] during the call to [`ProceduralNode::place`].
///
/// `Provides<T>` is space-agnostic: a node can add samplers for multiple spaces by calling
/// [`Provides::add`] with different space type parameters.
///
/// # Examples
/// ```
/// # use prockit_framework::{Provides, ProceduralNode, Provider, Subdivide, Placement, RealSpace};
/// # use bevy::prelude::*;
/// # use get_size2::GetSize;
/// # #[derive(Component, Clone, Default, GetSize)]
/// struct MyNode { value: f32 };
///
/// impl MyNode {
///     fn get_value(&self, _position: &Vec3) -> f32 {
///         self.value
///     }
/// }
///
/// impl ProceduralNode for MyNode {
///     fn provides(&self, provides: &mut Provides<Self>) {
///         // Add a sampler for RealSpace
///         provides.add::<RealSpace, _>("get value", MyNode::get_value);
///     }
///
///     fn subdivide(&self) -> Option<Subdivide> { None }
///     fn place(_: &Provider) -> Option<Self> { Some(Self::default()) }
/// }
/// ```
pub struct Provides<T: ProceduralNode> {
    provides: ErasedProvides,
    type_data: PhantomData<T>,
}

impl<T: ProceduralNode> Provides<T> {
    /// Create an empty `Provides` struct.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provides, ProceduralNode, Provider, Subdivide};
    /// # use bevy::prelude::*;
    /// # use get_size2::GetSize;
    /// # #[derive(Component, Clone, Default, GetSize)]
    /// struct MyNode;
    /// // impl ProceduralNode for MyNode...
    ///
    /// let provides = Provides::<MyNode>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            provides: ErasedProvides { entries: vec![] },
            type_data: PhantomData,
        }
    }

    /// Add a spatial sampling function to this instance of `Provides`.
    ///
    /// As its first argument, the sampler must operate on a reference to the [`ProceduralNode`]
    /// type which is providing this function (usually an `&self`). As its second argument, the
    /// sampler must take a reference to a position in the space it operates on. The function
    /// may have any return type.
    ///
    /// A node can add samplers for multiple spaces by calling this method with different
    /// space type parameters.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provides, ProceduralNode, Provider, Subdivide, RealSpace};
    /// # use bevy::prelude::*;
    /// # use get_size2::GetSize;
    /// # #[derive(Component, Clone, Default, GetSize)]
    /// struct MyNode { value: f32 };
    ///
    /// impl MyNode {
    ///     fn get_value(&self, _position: &Vec3) -> f32 {
    ///         self.value
    ///     }
    /// }
    ///
    /// // impl ProceduralNode for MyNode..
    ///
    /// let mut provides = Provides::<MyNode>::new();
    /// provides.add::<RealSpace, _>("get value", MyNode::get_value);
    /// ```
    pub fn add<S: Space, R: 'static>(
        &mut self,
        names: impl Into<Names>,
        sampler: fn(&T, &S::Position) -> R,
    ) {
        self.provides.entries.push((
            names.into(),
            Box::new(sampler),
            ErasedProvides::currier::<T, S, R>,
        ));
    }

    /// Return a `Provides` instance that has added a spatial sampling function. This method
    /// takes the same arguments as [`Provides::add`]; see the documentation on
    /// [`Provides::add`] for more details.
    pub fn with<S: Space, R: 'static>(
        mut self,
        names: impl Into<Names>,
        sampler: fn(&T, &S::Position) -> R,
    ) -> Self {
        self.add::<S, R>(names, sampler);
        self
    }

    /// Convert to type-erased form for storage and currying.
    pub(crate) fn into_erased(self) -> ErasedProvides {
        self.provides
    }
}

impl<T: ProceduralNode> Default for Provides<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// This struct is a type-erased way to query `Pod<T>` components in Bevy ECS, such that a
/// `Provider` can collect all `Provides` of all ancestor nodes, regardless of type.
#[derive(Component)]
pub(crate) struct PodProvides {
    pod: ErasedPod,
    provides: ErasedProvides,
    /// Type-erased function to call subdivide on the pod
    subdivide_fn: fn(&ErasedPod) -> Option<super::Subdivide>,
}

impl PodProvides {
    /// Create a new PodProvides by calling `provides()` on the node instance.
    pub(crate) fn new<T: ProceduralNode>(pod: Pod<T>) -> Self {
        let mut provides = Provides::<T>::new();
        pod.read().provides(&mut provides);
        Self {
            pod: Box::new(pod),
            provides: provides.into_erased(),
            subdivide_fn: |erased_pod| {
                erased_pod
                    .downcast_ref::<Pod<T>>()
                    .and_then(|pod| pod.subdivide())
            },
        }
    }

    /// Curry all samplers with the pod data and add to the provider.
    fn curry(&self, provider: &mut Provider) {
        self.provides.curry(&self.pod, provider);
    }

    /// Get the subdivisions from this node (type-erased).
    pub(crate) fn subdivide(&self) -> Option<super::Subdivide> {
        (self.subdivide_fn)(&self.pod)
    }
}

/// After a `Provides` is constructed for a type, it must be stored with all other types'
/// `Provides` in type-erased form. This allows the framework to handle nodes of any type
/// uniformly.
///
/// Note that `ProceduralNode::provides` enforces the use of `Provides<Self>` (rather than
/// `ErasedProvides`) because it ensures that all added sampling functions read the
/// `ProceduralNode` they were registered with.
pub(crate) struct ErasedProvides {
    entries: Vec<(Names, ErasedSampler, Currier)>,
}

impl ErasedProvides {
    /// This function is compiled and a pointer stored with every entry in the
    /// `ErasedProvides` such that downcasting (which is necessary to curry in the appropriate
    /// `Pod<T>`) can be performed on demand, even after types for the held sampling functions
    /// and pods have been erased.
    fn currier<T: ProceduralNode, S: Space, R: 'static>(
        names: &Names,
        sampler: &ErasedSampler,
        pod: &ErasedPod,
        provider: &mut Provider,
    ) {
        let sampler = sampler
            .downcast_ref::<fn(&T, &S::Position) -> R>()
            .unwrap()
            .clone();
        let pod = pod.downcast_ref::<Pod<T>>().unwrap().clone();
        let closure = move |position: &S::Position| pod.curry::<S, R>(sampler, position);
        provider.add::<S, R>(names.clone(), closure);
    }

    /// For all entries, downcast and curry in the appropriate `Pod<T>` using the stored
    /// `currier`, and add those curried functions to the `Provider`.
    fn curry(&self, pod: &ErasedPod, provider: &mut Provider) {
        for (names, sampler, currier) in self.entries.iter() {
            currier(names, sampler, pod, provider);
        }
    }
}

// Type aliases used to make struct definitions and function signatures more readable
type SpaceType = TypeId;
type ReturnType = TypeId;
type ErasedPod = Box<dyn Any + Send + Sync>;
type ErasedSampler = Box<dyn Any + Send + Sync>;
type ErasedClosure = Box<dyn Any>;
type ErasedTransform = Box<dyn Any>;
type Currier = fn(&Names, &ErasedSampler, &ErasedPod, &mut Provider);

#[cfg(test)]
mod tests {
    use super::super::*;
    use bevy::prelude::*;
    use get_size2::GetSize;

    #[derive(Component, Clone, Default, GetSize)]
    struct TestNode {
        value: f32,
    }

    impl ProceduralNode for TestNode {
        fn provides(&self, _provides: &mut Provides<Self>) {}

        fn subdivide(&self) -> Option<Subdivide> {
            None
        }

        fn place(_provider: &Provider) -> Option<Self> {
            Some(Self::default())
        }
    }

    #[test]
    fn test_provides() {
        impl TestNode {
            fn double(&self, position: &Vec3) -> Vec3 {
                position * 2.0
            }

            fn multiply(&self, position: &Vec3) -> Vec3 {
                position * self.value
            }
        }

        let provides = Provides::<TestNode>::new()
            .with::<RealSpace, _>("double", TestNode::double)
            .with::<RealSpace, _>("multiply", TestNode::multiply);

        let entry_result = provides.provides.entries.get(0);
        let entry_result_2 = provides.provides.entries.get(1);
        assert!(entry_result.is_some());
        assert!(entry_result_2.is_some());

        let (names, sampler, _) = entry_result.unwrap();
        let (names_2, sampler_2, _) = entry_result_2.unwrap();
        assert_eq!(*names, Names::from("double"));
        assert_eq!(*names_2, Names::from("multiply"));

        let sampler_result = sampler.downcast_ref::<fn(&TestNode, &Vec3) -> Vec3>();
        let sampler_result_2 = sampler_2.downcast_ref::<fn(&TestNode, &Vec3) -> Vec3>();
        assert!(sampler_result.is_some());
        assert!(sampler_result_2.is_some());

        let test_node = TestNode { value: 3.0 };
        let sampler = sampler_result.unwrap();
        let sampler_2 = sampler_result_2.unwrap();
        assert_eq!(sampler(&test_node, &Vec3::ONE), Vec3::splat(2.0));
        assert_eq!(sampler_2(&test_node, &Vec3::ONE), Vec3::splat(3.0));
    }
}
