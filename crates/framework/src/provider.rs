use super::{NameQuery, Names, Pod, ProceduralNode, Space};
use bevy::prelude::*;
use std::{
    any::{Any, TypeId},
    sync::Arc,
};

//TODO: Add transforms to provider

/// Internal storage element of a `Provider`, for collections of functions with type-erased
/// signatures which can be queried and returned.
#[derive(Clone)]
struct FunctionEntry {
    names: Names,
    return_type: TypeId,
    space: TypeId,
    /// Stores a type-erased closure of type `Box<dyn Fn(&S::Position) -> R + Send + Sync>`,
    /// with arbitrary return type `R`.
    function: Arc<dyn Any + Send + Sync>,
}

/// A registry for spatial sampling functions that can be looked up by function name and type.
/// Functions are stored with owned data (no lifetime parameter) to allow cloning and transfer
/// to async tasks.
///
/// # Examples
/// ```
/// # use prockit_framework::{Provides, Names, NameQuery, RealSpace};
/// # use bevy::prelude::*;
/// let mut provides = Provides::<RealSpace>::new();
///
/// fn increment(position: &Vec3) -> Vec3 { *position + Vec3::splat(1.0) }
///
/// provides.add(
///     "increment",
///     increment
/// );
///
/// let increment = provides.query::<Vec3>(&NameQuery::exact("increment")).unwrap();
///
/// assert_eq!(increment(&Vec3::splat(1.0)), Vec3::splat(2.0));
/// ```
#[derive(Clone, Component, Default)]
pub struct Provider {
    entries: Vec<FunctionEntry>,
}

impl Provider {
    /// Initializes an empty `Provides` registry.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provides};
    /// struct SomeType;
    /// let provides = Provides::<SomeType>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn provides<'a, T: ProceduralNode>(&'a mut self, pod: &'a Pod<T>) -> Provides<'a, T> {
        Provides {
            borrowed: self,
            pod,
        }
    }

    /// Queries the registry for a function of the specified return type and names. Returns a
    /// cloned Arc to the function if found, or `None` if no match exists.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provides, Names, NameQuery, RealSpace};
    /// # use bevy::prelude::*;
    /// let mut provides = Provides::<RealSpace>::new();
    ///
    /// fn constant_zero(_position: &Vec3) -> f64 { 0.0 }
    ///
    /// provides.add(
    ///     "constant zero",
    ///     constant_zero,
    /// );
    ///
    /// let queried_constant_zero = provides
    ///     .query::<f64>(&NameQuery::exact("constant zero"))
    ///     .unwrap();
    ///
    /// assert_eq!(queried_constant_zero(&Vec3::ZERO), 0.0);
    /// ```
    pub fn query<S: Space, R: 'static>(
        &self,
        name_query: &NameQuery,
    ) -> Option<&dyn Fn(&S::Position) -> R> {
        self.entries
            .iter()
            .find(|entry| {
                entry.return_type == TypeId::of::<R>()
                    && entry.space == TypeId::of::<S>()
                    && name_query.matches(&entry.names)
            })
            .and_then(|entry| {
                entry
                    .function
                    .downcast_ref::<Box<dyn Fn(&S::Position) -> R>>()
            })
            .map(|boxed| boxed.as_ref())
    }

    pub(crate) fn merge(&mut self, other: &Provider) {
        self.entries
            .extend(other.entries.iter().map(|entry| entry.clone()));
    }
}

pub struct Provides<'a, T: ProceduralNode> {
    pod: &'a Pod<T>,
    borrowed: &'a mut Provider,
}

impl<T: ProceduralNode> Provides<'_, T> {
    pub fn add<S: Space, R: 'static>(
        &mut self,
        names: impl Into<Names>,
        function: fn(&T, &S::Position) -> R,
    ) {
        let pod = self.pod.clone();
        let boxed: Box<dyn Fn(&S::Position) -> R + Send + Sync> =
            Box::new(move |input| pod.curry::<S, R>(function, input));
        self.borrowed.entries.push(FunctionEntry {
            names: names.into(),
            return_type: TypeId::of::<R>(),
            space: TypeId::of::<S>(),
            function: Arc::new(boxed),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::RealSpace;
    use bevy::prelude::*;

    #[test]
    fn test_provides_add_and_query() {
        let mut provides = Provides::<RealSpace>::new();

        fn distance(position: &Vec3) -> f32 {
            position.distance(Vec3::ZERO)
        }

        provides.add("distance", distance);

        let result = provides.query::<f32>(&NameQuery::exact("distance"));
        assert!(result.is_some());

        let func = result.unwrap();
        assert_eq!(func(&Vec3::new(5.0, 0.0, 0.0)), 5.0);
    }

    #[test]
    fn test_provides_query_not_found() {
        let provides = Provides::<RealSpace>::new();
        let result = provides.query::<()>(&NameQuery::exact("something"));
        assert!(result.is_none());
    }

    #[test]
    fn test_provides_lifetimes_and_closure() {
        fn take_provides_and_test(provides: &Provides<RealSpace>) {
            let function = provides.query::<f32>(&NameQuery::exact("distance"));
            assert!(function.is_some());
            assert_eq!(function.unwrap()(&Vec3::ZERO), 4.0);
        }

        fn create_provides() -> Provides<RealSpace> {
            struct Spot {
                coordinates: Vec3,
            }

            impl Spot {
                fn distance(&self, input: &Vec3) -> f32 {
                    input.distance(self.coordinates)
                }
            }

            let mut provides = Provides::<RealSpace>::new();
            let spot = Spot {
                coordinates: Vec3::new(4.0, 0.0, 0.0),
            };
            // Use move closure to transfer ownership
            provides.add("distance", move |input| spot.distance(input));
            provides
        }

        take_provides_and_test(&create_provides());
    }

    #[test]
    fn test_conflict() {
        #[derive(Clone)]
        struct Multiplier {
            value: f32,
        }

        impl Multiplier {
            fn multiply(&self, position: &Vec3) -> Vec3 {
                *position * self.value
            }
        }

        let mut provides = Provides::<RealSpace>::new();

        let grandparent = Multiplier { value: 2.0 };
        provides.add("multiply", move |position| grandparent.multiply(position));

        let parent = Multiplier { value: 3.0 };
        provides.add("multiply", move |position| parent.multiply(position));

        // First match wins (grandparent was added first)
        let multiply = provides
            .query::<Vec3>(&NameQuery::exact("multiply"))
            .unwrap();

        assert_eq!(multiply(&Vec3::splat(1.0)), Vec3::splat(2.0));
    }

    #[test]
    fn test_clone() {
        let mut provides = Provides::<RealSpace>::new();
        provides.add("answer", |_: &Vec3| 42i32);

        let cloned = provides.clone();

        let original_fn = provides.query::<i32>(&NameQuery::exact("answer")).unwrap();
        let cloned_fn = cloned.query::<i32>(&NameQuery::exact("answer")).unwrap();

        assert_eq!(original_fn(&Vec3::ZERO), 42);
        assert_eq!(cloned_fn(&Vec3::ZERO), 42);
    }
}
