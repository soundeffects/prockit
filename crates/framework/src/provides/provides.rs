use super::{NameQuery, NamedType, NamedTypeQuery, Names, Signature, SignatureQuery};
use std::any::Any;

/// Internal storage element of a `Provides` struct, encapsulating a registered
/// function along with its signature. The function is type-erased and stored
/// as a trait object, to be downcasted when retrieved via queries. If the function
/// was registered as a self-referential function, it will have a `self_type`
/// `TypeId`.
struct FunctionEntry {
    signature: Signature,
    function: Box<dyn Any + Send + Sync>,
}

/// A registry for functions that can be looked up by function signature with
/// many names associated with each type.
///
/// It is designed for maximum discoverability, where authors and users of
/// functions who are not necessarily aware of each other are given fallbacks
/// for finding desired functions.
///
/// # Examples
///
/// ```
/// # use prockit_framework::{Provides, Names, NameQuery};
/// let mut provides = Provides::new();
///
/// fn add(x: i32, y: i32) -> i32 { x + y }
///
/// provides.add_2(
///     add,
///     Names::new(["sum", "add", "addition"]), // First is the return/function names
///     Names::new(["x", "a", "first"]),        // Following are each argument's names...
///     Names::new(["y", "b", "second"])
/// );
///
/// let func = provides.query_2::<i32, i32, i32>(
///     NameQuery::exact("sum"),                    // First is the return/function query
///     NameQuery::from("x"),                       // Following are each argument's queries...
///     NameQuery::from_pattern("sec.*").unwrap(),  // `NameQuery` can be a regular expression!
/// ).unwrap();
///
/// assert_eq!(func(3, 4), 7);
/// ```
pub struct Provides {
    entries: Vec<FunctionEntry>,
}

impl Provides {
    /// Creates a new, empty `Provides` registry.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::Provides;
    /// let provides = Provides::new();
    /// ```
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Registers any function type with zero arguments. Expects a set of names for
    /// the return type/function.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::{Provides, Names};
    /// let mut provides = Provides::new();
    ///
    /// fn zero() -> f64 { 0.0 }
    ///
    /// provides.add_0(
    ///     zero,
    ///     Names::from("zero"),
    /// );
    /// ```
    pub fn add_0<R: 'static>(
        &mut self,
        function: impl Fn() -> R + Send + Sync + 'static,
        r_names: Names,
    ) {
        let trait_object: Box<dyn Fn() -> R + Send + Sync> = Box::new(function);
        self.entries.push(FunctionEntry {
            signature: Signature::new(NamedType::new::<R>(r_names), vec![]),
            function: Box::new(trait_object),
        });
    }

    /// Registers any function type with one argument. Expects a set of names for the
    /// return type, followed by the argument type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::{Provides, Names};
    /// let mut provides = Provides::new();
    ///
    /// // Static function
    /// fn double(x: i32) -> i32 { x * 2 }
    /// provides.add_1(double, Names::from("double"), Names::from("x"));
    ///
    /// // Closure with captured environment
    /// let multiplier = 3;
    /// provides.add_1(
    ///     move |x: i32| x * multiplier,
    ///     Names::from("triple"),
    ///     Names::from("x")
    /// );
    /// ```
    pub fn add_1<R: 'static, A: 'static>(
        &mut self,
        function: impl Fn(A) -> R + Send + Sync + 'static,
        r_names: Names,
        a_names: Names,
    ) {
        let trait_object: Box<dyn Fn(A) -> R + Send + Sync> = Box::new(function);
        self.entries.push(FunctionEntry {
            signature: Signature::new(
                NamedType::new::<R>(r_names),
                vec![NamedType::new::<A>(a_names)],
            ),
            function: Box::new(trait_object),
        });
    }

    /// Registers any function type with two arguments. Expects a set of names for the
    /// return type, followed by names for the two argument types.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::{Provides, Names};
    /// let mut provides = Provides::new();
    ///
    /// // Static function
    /// fn add(x: i32, y: i32) -> i32 { x + y }
    /// provides.add_2(add, Names::from("sum"), Names::from("x"), Names::from("y"));
    ///
    /// // Closure with captured environment
    /// let offset = 10;
    /// provides.add_2(
    ///     move |x: i32, y: i32| x + y + offset,
    ///     Names::from("sum_with_offset"),
    ///     Names::from("x"),
    ///     Names::from("y")
    /// );
    /// ```
    pub fn add_2<R: 'static, A1: 'static, A2: 'static>(
        &mut self,
        function: impl Fn(A1, A2) -> R + Send + Sync + 'static,
        r_names: Names,
        a1_names: Names,
        a2_names: Names,
    ) {
        let trait_object: Box<dyn Fn(A1, A2) -> R + Send + Sync> = Box::new(function);
        self.entries.push(FunctionEntry {
            signature: Signature::new(
                NamedType::new::<R>(r_names),
                vec![
                    NamedType::new::<A1>(a1_names),
                    NamedType::new::<A2>(a2_names),
                ],
            ),
            function: Box::new(trait_object),
        });
    }

    /// Registers any function type with three arguments. Expects a set of names for the
    /// return type, followed by names for the argument types.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::{Provides, Names};
    /// let mut provides = Provides::new();
    ///
    /// fn multiply_add(x: i32, y: i32, z: i32) -> i32 { x * y + z }
    /// provides.add_3(
    ///     multiply_add,
    ///     Names::from("fma"),
    ///     Names::from("x"),
    ///     Names::from("y"),
    ///     Names::from("z")
    /// );
    /// ```
    pub fn add_3<R: 'static, A1: 'static, A2: 'static, A3: 'static>(
        &mut self,
        function: impl Fn(A1, A2, A3) -> R + Send + Sync + 'static,
        r_names: Names,
        a1_names: Names,
        a2_names: Names,
        a3_names: Names,
    ) {
        let trait_object: Box<dyn Fn(A1, A2, A3) -> R + Send + Sync> = Box::new(function);
        self.entries.push(FunctionEntry {
            signature: Signature::new(
                NamedType::new::<R>(r_names),
                vec![
                    NamedType::new::<A1>(a1_names),
                    NamedType::new::<A2>(a2_names),
                    NamedType::new::<A3>(a3_names),
                ],
            ),
            function: Box::new(trait_object),
        });
    }

    /// Registers any function type with four arguments. Expects a set of names for the
    /// return type, followed by names for all arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::{Provides, Names};
    /// let mut provides = Provides::new();
    ///
    /// fn blend(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ///     ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    /// }
    /// provides.add_4(
    ///     blend,
    ///     Names::from("color"),
    ///     Names::from("r"),
    ///     Names::from("g"),
    ///     Names::from("b"),
    ///     Names::from("a")
    /// );
    /// ```
    pub fn add_4<R: 'static, A1: 'static, A2: 'static, A3: 'static, A4: 'static>(
        &mut self,
        function: impl Fn(A1, A2, A3, A4) -> R + Send + Sync + 'static,
        r_names: Names,
        a1_names: Names,
        a2_names: Names,
        a3_names: Names,
        a4_names: Names,
    ) {
        let trait_object: Box<dyn Fn(A1, A2, A3, A4) -> R + Send + Sync> = Box::new(function);
        self.entries.push(FunctionEntry {
            signature: Signature::new(
                NamedType::new::<R>(r_names),
                vec![
                    NamedType::new::<A1>(a1_names),
                    NamedType::new::<A2>(a2_names),
                    NamedType::new::<A3>(a3_names),
                    NamedType::new::<A4>(a4_names),
                ],
            ),
            function: Box::new(trait_object),
        });
    }

    /// Queries the registry for a function that takes zero arguments. Expects a `NameQuery`
    /// for the return type names.
    ///
    /// Returns a reference to the function if found, or `None` if no match exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::{Provides, Names, NameQuery};
    /// let mut provides = Provides::new();
    ///
    /// fn zero() -> i32 { 0 }
    ///
    /// provides.add_0(
    ///     zero,
    ///     Names::from("zero")
    /// );
    ///
    /// let zero = provides.query_0::<i32>(
    ///     NameQuery::exact("zero"),
    /// ).unwrap();
    ///
    /// assert_eq!(zero(), 0);
    /// ```
    pub fn query_0<R: 'static>(
        &self,
        r_query: NameQuery,
    ) -> Option<&(dyn Fn() -> R + Send + Sync)> {
        let query = SignatureQuery::new(NamedTypeQuery::new::<R>(r_query), vec![]);
        self.entries
            .iter()
            .find(|entry| query.matches(&entry.signature))
            .and_then(|entry| {
                entry
                    .function
                    .downcast_ref::<Box<dyn Fn() -> R + Send + Sync>>()
                    .map(|arc| arc.as_ref())
            })
    }

    /// Queries the registry for a function with one argument matching the specified
    /// return type and argument type names.
    ///
    /// Returns a reference to the function if found, or `None` if no match exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::{Provides, Names, NameQuery};
    /// let mut provides = Provides::new();
    ///
    /// fn square(x: i32) -> i32 { x * x }
    /// provides.add_1(square, Names::from("square"), Names::from("x"));
    ///
    /// let func = provides.query_1::<i32, i32>(
    ///     NameQuery::exact("square"),
    ///     NameQuery::exact("x")
    /// ).unwrap();
    ///
    /// assert_eq!(func(5), 25);
    /// ```
    pub fn query_1<R: 'static, A: 'static>(
        &self,
        r_query: NameQuery,
        a_query: NameQuery,
    ) -> Option<&(dyn Fn(A) -> R + Send + Sync)> {
        let query = SignatureQuery::new(
            NamedTypeQuery::new::<R>(r_query),
            vec![NamedTypeQuery::new::<A>(a_query)],
        );
        self.entries
            .iter()
            .find(|entry| query.matches(&entry.signature))
            .and_then(|entry| {
                entry
                    .function
                    .downcast_ref::<Box<dyn Fn(A) -> R + Send + Sync>>()
                    .map(|arc| arc.as_ref())
            })
    }

    /// Queries the registry for a function with two arguments matching the specified
    /// return names followed by argument names.
    ///
    /// Returns a reference to the function if found, or `None` if no match exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::{Provides, Names, NameQuery};
    /// let mut provides = Provides::new();
    ///
    /// fn add(x: i32, y: i32) -> i32 { x + y }
    /// provides.add_2(add, Names::from("sum"), Names::from("x"), Names::from("y"));
    ///
    /// let func = provides.query_2::<i32, i32, i32>(
    ///     NameQuery::exact("sum"),
    ///     NameQuery::exact("x"),
    ///     NameQuery::exact("y")
    /// ).unwrap();
    ///
    /// assert_eq!(func(3, 4), 7);
    /// ```
    pub fn query_2<R: 'static, A1: 'static, A2: 'static>(
        &self,
        r_query: NameQuery,
        a1_query: NameQuery,
        a2_query: NameQuery,
    ) -> Option<&(dyn Fn(A1, A2) -> R + Send + Sync)> {
        let query = SignatureQuery::new(
            NamedTypeQuery::new::<R>(r_query),
            vec![
                NamedTypeQuery::new::<A1>(a1_query),
                NamedTypeQuery::new::<A2>(a2_query),
            ],
        );
        self.entries
            .iter()
            .find(|entry| query.matches(&entry.signature))
            .and_then(|entry| {
                entry
                    .function
                    .downcast_ref::<Box<dyn Fn(A1, A2) -> R + Send + Sync>>()
                    .map(|arc| arc.as_ref())
            })
    }

    /// Queries the registry for a function with three arguments matching the specified
    /// return names followed by argument names.
    ///
    /// Returns a reference to the function if found, or `None` if no match exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::{Provides, Names, NameQuery};
    /// let mut provides = Provides::new();
    ///
    /// fn multiply_add(x: i32, y: i32, z: i32) -> i32 { x * y + z }
    /// provides.add_3(
    ///     multiply_add,
    ///     Names::from("fma"),
    ///     Names::from("x"),
    ///     Names::from("y"),
    ///     Names::from("z")
    /// );
    ///
    /// let func = provides.query_3::<i32, i32, i32, i32>(
    ///     NameQuery::exact("fma"),
    ///     NameQuery::exact("x"),
    ///     NameQuery::exact("y"),
    ///     NameQuery::exact("z")
    /// ).unwrap();
    ///
    /// assert_eq!(func(3, 4, 5), 17);
    /// ```
    pub fn query_3<R: 'static, A1: 'static, A2: 'static, A3: 'static>(
        &self,
        r_query: NameQuery,
        a1_query: NameQuery,
        a2_query: NameQuery,
        a3_query: NameQuery,
    ) -> Option<&(dyn Fn(A1, A2, A3) -> R + Send + Sync)> {
        let query = SignatureQuery::new(
            NamedTypeQuery::new::<R>(r_query),
            vec![
                NamedTypeQuery::new::<A1>(a1_query),
                NamedTypeQuery::new::<A2>(a2_query),
                NamedTypeQuery::new::<A3>(a3_query),
            ],
        );
        self.entries
            .iter()
            .find(|entry| query.matches(&entry.signature))
            .and_then(|entry| {
                entry
                    .function
                    .downcast_ref::<Box<dyn Fn(A1, A2, A3) -> R + Send + Sync>>()
                    .map(|arc| arc.as_ref())
            })
    }

    /// Queries the registry for a function with four arguments matching the specified
    /// return names followed by argument names.
    ///
    /// Returns a reference to the function if found, or `None` if no match exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::{Provides, Names, NameQuery};
    /// let mut provides = Provides::new();
    ///
    /// fn blend(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ///     ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    /// }
    /// provides.add_4(
    ///     blend,
    ///     Names::from("color"),
    ///     Names::from("r"),
    ///     Names::from("g"),
    ///     Names::from("b"),
    ///     Names::from("a")
    /// );
    ///
    /// let func = provides.query_4::<u32, u8, u8, u8, u8>(
    ///     NameQuery::exact("color"),
    ///     NameQuery::exact("r"),
    ///     NameQuery::exact("g"),
    ///     NameQuery::exact("b"),
    ///     NameQuery::exact("a")
    /// ).unwrap();
    ///
    /// assert_eq!(func(255, 128, 64, 200), 3372187712);
    /// ```
    pub fn query_4<R: 'static, A1: 'static, A2: 'static, A3: 'static, A4: 'static>(
        &self,
        r_query: NameQuery,
        a1_query: NameQuery,
        a2_query: NameQuery,
        a3_query: NameQuery,
        a4_query: NameQuery,
    ) -> Option<&(dyn Fn(A1, A2, A3, A4) -> R + Send + Sync)> {
        let query = SignatureQuery::new(
            NamedTypeQuery::new::<R>(r_query),
            vec![
                NamedTypeQuery::new::<A1>(a1_query),
                NamedTypeQuery::new::<A2>(a2_query),
                NamedTypeQuery::new::<A3>(a3_query),
                NamedTypeQuery::new::<A4>(a4_query),
            ],
        );
        self.entries
            .iter()
            .find(|entry| query.matches(&entry.signature))
            .and_then(|entry| {
                entry
                    .function
                    .downcast_ref::<Box<dyn Fn(A1, A2, A3, A4) -> R + Send + Sync>>()
                    .map(|arc| arc.as_ref())
            })
    }
}

impl Default for Provides {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_provides_add_and_query() {
        let mut provides = Provides::new();

        fn add(x: i32, y: i32) -> i32 {
            x + y
        }

        provides.add_2(add, Names::from("sum"), Names::from("x"), Names::from("y"));

        let result = provides.query_2::<i32, i32, i32>(
            NameQuery::from("sum"),
            NameQuery::from("x"),
            NameQuery::from("y"),
        );

        assert!(result.is_some());
        let func = result.unwrap();
        assert_eq!(func(2, 3), 5);
    }

    #[test]
    fn test_provides_query_not_found() {
        let provides = Provides::new();

        let result = provides.query_2::<i32, i32, i32>(
            NameQuery::from("sum"),
            NameQuery::from("x"),
            NameQuery::from("y"),
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_provides_lifetimes_and_closure() {
        fn take_provides_and_test(provides: &Provides) {
            let function = provides
                .query_1::<i32, i32>(NameQuery::exact("multiply"), NameQuery::exact("input"));
            assert!(function.is_some());
            assert_eq!(function.unwrap()(2), 4);
        }

        fn create_provides() -> Provides {
            let mut provides = Provides::new();

            struct Multiplier {
                value: i32,
            }

            let multiplier = Multiplier { value: 2 };
            provides.add_1(
                move |input: i32| multiplier.value * input,
                Names::from("multiply"),
                Names::from("input"),
            );
            provides
        }

        take_provides_and_test(&create_provides());
    }
}
