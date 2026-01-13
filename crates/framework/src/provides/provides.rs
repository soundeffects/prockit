use super::{NameQuery, Names};
use crate::Space;
use std::{any::Any, any::TypeId, marker::PhantomData, sync::Arc};

/// Internal storage element of a `Provides` struct, encapsulating a registered function along
/// with its signature. The function is stored as an Arc-wrapped trait object for cloning support.
struct FunctionEntry<S: Space> {
    names: Names,
    return_type: TypeId,
    /// Stores `Arc<dyn Fn(&S::Position) -> R + Send + Sync>` as a type-erased Any
    function: Arc<dyn Any + Send + Sync>,
    _marker: PhantomData<S>,
}

impl<S: Space> Clone for FunctionEntry<S> {
    fn clone(&self) -> Self {
        Self {
            names: self.names.clone(),
            return_type: self.return_type,
            function: Arc::clone(&self.function),
            _marker: PhantomData,
        }
    }
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
pub struct Provides<S: Space> {
    entries: Vec<FunctionEntry<S>>,
}

impl<S: Space> Clone for Provides<S> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}

impl<S: Space> Provides<S> {
    /// Creates a new, empty `Provides` registry.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provides, RealSpace};
    /// let provides = Provides::<RealSpace>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Registers any function type. Expects a set of names for the return type/function.
    /// The function must be `'static` (own all captured data) to allow cloning and async transfer.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provides, Names, RealSpace};
    /// # use bevy::prelude::*;
    /// let mut provides = Provides::<RealSpace>::new();
    ///
    /// fn constant_zero(_position: &Vec3) -> f64 { 0.0 }
    ///
    /// provides.add(
    ///     "constant zero",
    ///     constant_zero
    /// );
    /// ```
    pub fn add<R: 'static>(
        &mut self,
        names: impl Into<Names>,
        function: impl Fn(&S::Position) -> R + Send + Sync + 'static,
    ) {
        // Wrap the function in Arc for clone support
        let arc_fn: Arc<dyn Fn(&S::Position) -> R + Send + Sync> = Arc::new(function);

        self.entries.push(FunctionEntry {
            names: names.into(),
            return_type: TypeId::of::<R>(),
            // Store the Arc<dyn Fn> as Arc<dyn Any> for type erasure
            function: Arc::new(arc_fn) as Arc<dyn Any + Send + Sync>,
            _marker: PhantomData,
        });
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
    pub fn query<R: 'static>(
        &self,
        name_query: &NameQuery,
    ) -> Option<Arc<dyn Fn(&S::Position) -> R + Send + Sync>> {
        self.entries
            .iter()
            .find(|entry| {
                entry.return_type == TypeId::of::<R>() && name_query.matches(&entry.names)
            })
            .and_then(|entry| {
                // Downcast from Arc<dyn Any> back to Arc<Arc<dyn Fn...>>
                entry
                    .function
                    .downcast_ref::<Arc<dyn Fn(&S::Position) -> R + Send + Sync>>()
                    .cloned()
            })
    }
}

impl<S: Space> Default for Provides<S> {
    fn default() -> Self {
        Self::new()
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
