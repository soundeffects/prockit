use super::{NameQuery, Names, Pod, ProceduralNode, Space};
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
/// [`ProceduralNode::generate`].
///
/// # Examples
/// ```
/// # use prockit_framework::{Provider, RealSpace};
/// # use bevy::prelude::*;
/// fn generate_count(provider: &Provider) -> i32 {
///     let count = provider.query::<RealSpace, i32>("count").unwrap();
///     count(&Vec3::ZERO) + 1;
/// }
/// ```
pub struct Provider {
    entries: Vec<(Names, SpaceType, ReturnType, ErasedClosure)>,
    transforms: TypeIdMap<ErasedTransform>,
}

impl Provider {
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
    /// # use prockit_framework::{Provides, NameQuery, RealSpace};
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
        pod_provides: Query<&PodProvides>,
        hierarchy: Query<&ChildOf>,
        provide_map: &ProvideMap,
    ) -> Self {
        let mut provider = Provider {
            entries: vec![],
            transforms: TypeIdMap::default(),
        };

        // TODO: collect transforms
        for pod_provides in hierarchy
            .iter_ancestors(entity)
            .filter_map(|entity| pod_provides.get(entity).ok())
        {
            provide_map.curry(pod_provides, &mut provider);
        }

        provider
    }
}

/// An interface by which a [`ProceduralNode`] exports the spatial sampling functions that can
/// be called on it. These functions are later collected into a [`Provider`], which is given to
/// any child [`ProceduralNode`] to the node that offers this instance of `Provides` during the
/// call to [`ProceduralNode::generate`].
///
/// # Examples
/// ```
/// # use prockit_framework::{Provides, ProceduralNode, Subdivide, Provider};
/// # use get_size2::GetSize;
/// # #[derive(Default, GetSize)]
/// struct MyNode { value: f32 };
///
/// impl MyNode {
///     fn get_value(&self, _position: &Vec3) -> f32 {
///         self.value
///     }
/// }
///
/// impl ProceduralNode for MyNode {
///     fn provides() -> Provides<Self> {
///         Provides::<Self>::new()
///             .with<RealSpace, _>("get value", TestNode::get_value)
///     }
///     // ...
/// # fn subdivide(&self) -> Option<Subdivide> { None }
/// # fn generate(&mut self, provider: &Provider) {}
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
    /// # use prockit_framework::Provides;
    /// # use get_size2::GetSize;
    /// # #[derive(Default, GetSize)]
    /// struct MyNode;
    /// // impl ProceduralNode for MyNode...
    /// # impl ProceduralNode for MyNode {
    /// # fn provides() -> Provides<Self> { Provides::<Self>::new() }
    /// # fn subdivide(&self) -> Option<Subdivide> { None }
    /// # fn generate(&mut self, provider: &Provider) {}
    /// # }
    /// let provides = Provides::<MyNode>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            provides: ErasedProvides { entries: vec![] },
            type_data: PhantomData,
        }
    }

    /// Add a spatial sampling function to this instance of `Provides`. As its first argument,
    /// the sampler must operate on a reference to the [`ProceduralNode`] type which is
    /// providing this function (usually an `&self`). As its second argument, the sampler must
    /// take a reference to a position in the space it operates on. The function may have any
    /// return type.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provides, ProceduralNode, Subdivide, Provider};
    /// # use get_size2::GetSize;
    /// # #[derive(Default, GetSize)]
    /// struct MyNode { value: f32 };
    ///
    /// impl MyNode {
    ///     fn get_value(&self, _position: &Vec3) -> f32 {
    ///         self.value
    ///     }
    /// }
    ///
    /// // impl ProceduralNode for MyNode..
    /// # impl ProceduralNode for MyNode {
    /// # fn provides() -> Provides<Self> { Provides::<Self>::new() }
    /// # fn subdivide(&self) -> Option<Subdivide> { None }
    /// # fn generate(&mut self, provider: &Provider) {}
    /// # }
    ///
    /// let provides = Provides::<MyNode>::new();
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
}

/// This struct is a type-erased way to query `Pod<T>` components in Bevy ECS, such that a
/// `Provider` can collect all `Provides` of all ancestor nodes, regardless of type.
#[derive(Component)]
pub(crate) struct PodProvides {
    node_id: TypeId,
    pod: ErasedPod,
}

/// This holds (type-erased) `Provides` values corresponding to all `ProceduralNode`
/// types registered to the Bevy app at startup using `ProceduralNode::provides`.
///
/// Given a `PodProvides` and a mutable `Provider`, this struct will add all sampling functions
/// (with the appropriate `Pod<T>` curried in for the sampler to read) to the `Provider`.
#[derive(Resource)]
pub(crate) struct ProvideMap {
    map: TypeIdMap<ErasedProvides>,
}

impl ProvideMap {
    /// Register a `Provides` value for the given `ProceduralNode` type. This method should
    /// only be called at app startup.
    pub(crate) fn register<T: ProceduralNode>(&mut self) {
        self.map.insert_type::<T>(T::provides().provides);
    }

    /// With a `PodProvides` and a mutable `Provider`, this will find the `Provides`
    /// corresponding to the type held in the `PodProvides`, and will delegate the task of
    /// adding the functions and currying them with the `Pod<T>` to the `Provides` struct.
    fn curry(&self, pod_provides: &PodProvides, provider: &mut Provider) {
        if let Some(provides) = self.map.get(&pod_provides.node_id) {
            provides.curry(&pod_provides.pod, provider);
        }
    }
}

/// After a `Provides` is constructed for a type, it must be stored with all other types'
/// `Provides` in the `ProvideMap`. Thus, the generic argument of the `Provides` must be
/// type-erased.
///
/// Note that `ProceduralNode::provides` enforces the use of `Provides<Self>` (rather than
/// `ErasedProvides`) because it ensures that all added sampling functions read the
/// `ProceduralNode` they were registered with.
struct ErasedProvides {
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

    #[derive(Default, GetSize)]
    struct TestNode {
        value: f32,
    }

    impl ProceduralNode for TestNode {
        fn provides() -> Provides<Self> {
            Provides::<Self>::new()
        }

        fn subdivide(&self) -> Option<Subdivide> {
            None
        }

        fn generate(&mut self, _provider: &Provider) {}
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
