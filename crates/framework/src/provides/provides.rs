use super::{NewSignatureQuery, Signature, Signatured};
use std::any::Any;

/// Internal storage element of a `Provides` struct, encapsulating a registered
/// function along with its signature. The function is type-erased and stored
/// as a trait object, to be downcasted when retrieved via queries.
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
/// # use framework::provides::{Provides, Names, NameQuery};
/// let mut provides = Provides::new();
///
/// fn add(x: i32, y: i32) -> i32 { x + y }
///
/// provides.add(
///     add as fn(i32, i32) -> i32,
///     (
///         Names::new(["sum", "add", "addition"]), // First is the return/function names
///         Names::new(["x", "a", "first"]),        // Following are each argument's names...
///         Names::new(["y", "b", "second"])
///     ),
/// );
///
/// let func = provides.query::<fn(i32, i32) -> i32, _>((
///     NameQuery::exact("sum"),            // First is the return/function query
///     NameQuery::from("x"),               // Following are each argument's queries...
///     NameQuery::from_pattern("sec.*"),   // `NameQuery` can be a regular expression!
/// )).unwrap();
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
    /// # use framework::provides::Provides;
    /// let provides = Provides::new();
    /// ```
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Registers a function. This method expects a set of names for the return type or
    /// function first, and each of its arguments following in their respective order.
    /// You may only register functions with up to four arguments.
    ///
    /// Note for `Provides` registries in `ProceduralNode::provides`: if the first argument
    /// is the same type (or corresponding reference) as the `ProceduralNode` type, it is
    /// interpreted to be a `self` argument.
    ///
    /// # Examples
    ///
    /// ```
    /// # use framework::provides::{Provides, Names};
    /// let mut provides = Provides::new();
    ///
    /// fn multiply(a: f64, b: f64) -> f64 { a * b }
    ///
    /// provides.add(
    ///     multiply as fn(f64, f64) -> f64,
    ///     (
    ///         Names::from("product"),     // First is the return/function names
    ///         Names::new(["a", "x"]),     // Following are each argument's names...
    ///         Names::new(["b", "y"])),
    ///     )
    /// );
    /// ```
    pub fn add<Function, Names>(&mut self, function: Function, names: Names)
    where
        Function: 'static + Send + Sync + Signatured<Names>,
    {
        let signature = Function::signature(names);
        self.entries.push(FunctionEntry {
            signature,
            function: Box::new(function),
        });
    }

    /// Queries the registry for a function matching the specified signature and names.
    /// A set of `NameQuery` types corresponding to the function/return value and each
    /// argument is expected.
    ///
    /// Returns a reference to the function if found, or `None` if no match exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use framework::provides::{Provides, Names};
    /// let mut provides = Provides::new();
    ///
    /// fn subtract(x: i32, y: i32) -> i32 { x - y }
    ///
    /// provides.add(
    ///     subtract as fn(i32, i32) -> i32,
    ///     (Names::from("difference"), Names::from("x"), Names::from("y")),
    /// );
    ///
    /// let func = provides.query::<fn(i32, i32) -> i32, _>(
    ///     ("difference", "x", "y"),
    /// ).unwrap();
    ///
    /// assert_eq!(func(10, 3), 7);
    /// ```
    pub fn query<Function, NameQueries>(&self, name_queries: NameQueries) -> Option<&Function>
    where
        Function: 'static + NewSignatureQuery<NameQueries>,
    {
        let query = Function::new_query(name_queries);

        self.entries
            .iter()
            .find(|entry| query.matches(&entry.signature))
            .and_then(|entry| entry.function.downcast_ref::<Function>())
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

        provides.add(
            add as fn(i32, i32) -> i32,
            (Names::from("sum"), Names::from("x"), Names::from("y")),
        );

        let result = provides.query::<fn(i32, i32) -> i32, _>((
            NameQuery::from("sum"),
            NameQuery::from("x"),
            NameQuery::from("y"),
        ));

        assert!(result.is_some());
        let func = result.unwrap();
        assert_eq!(func(2, 3), 5);
    }

    #[test]
    fn test_provides_query_not_found() {
        let provides = Provides::new();

        let result = provides.query::<fn(i32, i32) -> i32, _>((
            NameQuery::from("sum"),
            NameQuery::from("x"),
            NameQuery::from("y"),
        ));

        assert!(result.is_none());
    }

    #[test]
    fn test_self_reference_matching() {
        let mut provides = Provides::new();

        fn test_fn() -> i32 {
            42
        }

        provides.add(test_fn as fn() -> i32, Names::from("answer"));

        let result = provides.query::<fn() -> i32, _>((NameQuery::from("answer"),));
        assert!(result.is_some());

        let result = provides.query::<fn() -> i32, _>((NameQuery::from("answer"),));
        assert!(result.is_none());
    }
}
