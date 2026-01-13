use super::{NameQuery, Names};
use crate::Space;
use std::{any::TypeId, marker::PhantomData};

/// Internal storage element of a `Provides` struct, encapsulating a registered function along
/// with its signature. The function is type-erased and stored as a raw pointer, to be cast back
/// when retrieved via queries.
///
/// SAFETY: The `function_ptr` is a leaked Box (`Box<dyn Fn(...) -> ...>`) and must be dropped
/// manually.
struct FunctionEntry<'a> {
    names: Names,
    return_type: TypeId,
    function: *const (),
    drop: fn(*const ()),
    _lifetime: PhantomData<&'a ()>,
}

// SAFETY: The underlying function is Send + Sync, and we only access it safely
unsafe impl Send for FunctionEntry<'_> {}
unsafe impl Sync for FunctionEntry<'_> {}

impl Drop for FunctionEntry<'_> {
    fn drop(&mut self) {
        (self.drop)(self.function);
    }
}

/// A registry for spatial sampling functions that can be looked up by function name and type.
///
/// # Examples
/// ```
/// # use prockit_framework::{Provides, Names, NameQuery, RealSpace};
/// let mut provides = Provides::<RealSpace>::new();
///
/// fn increment(position: &Vec3) -> Vec3 { position + Vec3::splat(1.0) }
///
/// provides.add(
///     "increment",
///     increment
/// );
///
/// let increment = provides.query::<Vec3>("increment").unwrap();
///
/// assert_eq!(increment(&Vec3::splat(1.0)), Vec3::splat(2.0));
/// ```
pub struct Provides<'a, S: Space> {
    entries: Vec<FunctionEntry<'a>>,
    space_type_data: PhantomData<S>,
}

impl<'a, S: Space> Provides<'a, S> {
    /// Creates a new, empty `Provides` registry.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::Provides;
    /// let provides = Provides::new();
    /// ```
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            space_type_data: PhantomData,
        }
    }

    /// Registers any function type with zero arguments. Expects a set of names for
    /// the return type/function.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provides, Names, RealSpace};
    /// let mut provides = Provides::<RealSpace>::new();
    ///
    /// fn constant_zero(position: &Vec3) -> f64 { 0.0 }
    ///
    /// provides.add(
    ///     "constant zero",
    ///     constant_zero
    /// );
    /// ```
    pub fn add<R: 'static>(
        &mut self,
        names: impl Into<Names>,
        function: impl Fn(&S::Position) -> R + Send + Sync + 'a,
    ) {
        let boxed: Box<dyn Fn(&S::Position) -> R + Send + Sync + 'a> = Box::new(function);

        fn drop<R: 'static>(function: *const ()) {
            // SAFETY: function was created from Box::into_raw of matching function signature
            unsafe {
                let _ = Box::from_raw(function as *mut Box<dyn Fn() -> R + Send + Sync>);
            }
        }

        self.entries.push(FunctionEntry {
            names: names.into(),
            return_type: TypeId::of::<R>(),
            function: Box::into_raw(Box::new(boxed)) as *const (),
            drop: drop::<R>,
            _lifetime: std::marker::PhantomData,
        });
    }

    /// Queries the registry for a function of the specified return type and names. Returns a
    /// reference to the function if found, or `None` if no match exists.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provides, Names, NameQuery, RealSpace};
    /// let mut provides = Provides::<RealSpace>::new();
    ///
    /// fn constant_zero(position: &Vec3) -> f64 { 0.0 }
    ///
    /// provides.add(
    ///     "constant zero"
    ///     constant_zero,
    /// );
    ///
    /// let queried_constant_zero = provides
    ///     .query::<f64>(&NameQuery::exact("constant zero"))
    ///     .unwrap();
    ///
    /// assert_eq!(queried_constant_zero(), 0.0);
    /// ```
    pub fn query<R: 'static>(
        &self,
        name_query: &NameQuery,
    ) -> Option<&(dyn Fn(&S::Position) -> R + Send + Sync + 'a)> {
        self.entries
            .iter()
            .find(|entry| {
                entry.return_type == TypeId::of::<R>() && name_query.matches(&entry.names)
            })
            .map(|entry| {
                // SAFETY: match guarantees the function pointer follows the casted signature
                unsafe {
                    let boxed = &*(entry.function
                        as *const Box<dyn Fn(&S::Position) -> R + Send + Sync + 'a>);
                    boxed.as_ref()
                }
            })
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
        fn take_provides_and_test(provides: &Provides<'_, RealSpace>) {
            let function = provides.query::<f32>(&NameQuery::exact("distance"));
            assert!(function.is_some());
            assert_eq!(function.unwrap()(&Vec3::ZERO), 4.0);
        }

        fn create_provides() -> Provides<'static, RealSpace> {
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
            provides.add("distance", move |input| spot.distance(input));
            provides
        }

        take_provides_and_test(&create_provides());
    }

    #[test]
    fn test_conflict() {
        struct Multiplier {
            value: f32,
        }

        impl Multiplier {
            fn multiply(&self, position: &Vec3) -> Vec3 {
                position * self.value
            }
        }

        let mut provides = Provides::<RealSpace>::new();

        let grandparent = Multiplier { value: 2.0 };
        provides.add("multiply", move |position| grandparent.multiply(position));

        let parent = Multiplier { value: 3.0 };
        provides.add("multiply", move |position| parent.multiply(position));

        let multiply = provides
            .query::<Vec3>(&NameQuery::exact("multiply"))
            .unwrap();

        assert_eq!(multiply(&Vec3::splat(1.0)), Vec3::splat(3.0));
    }
}
