use super::{NamedType, Names, Signature};
use regex::Regex;
use std::any::TypeId;

/// A query for matching against name collections using regex patterns or exact
/// matches.
///
/// # Examples
///
/// ```
/// # use framework::provides::{NameQuery, Names};
/// let exact = NameQuery::exact("add");
/// let names = Names::from("add");
/// assert!(exact.matches(&names));
///
/// let pattern = NameQuery::from_pattern("get_.*").unwrap();
/// let getter_names = Names::from("get_value");
/// assert!(pattern.matches(&getter_names));
/// ```
pub struct NameQuery {
    regex: Regex,
}

impl NameQuery {
    /// Creates a new name query from a compiled regex pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # use framework::provides::{NameQuery, Names};
    /// # use regex::Regex;
    /// // using regex::Regex
    /// let regex = Regex::new("test.*").unwrap();
    /// let query = NameQuery::new(regex);
    /// assert!(query.matches(&Names::from("testing")));
    /// ```
    pub fn new(regex: Regex) -> Self {
        Self { regex }
    }

    /// Creates a name query from a regex pattern string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use framework::provides::{NameQuery, Names};
    /// let query = NameQuery::from_pattern("calc_.*").unwrap();
    /// assert!(query.matches(&Names::from("calc_sum")));
    /// assert!(!query.matches(&Names::from("compute_sum")));
    /// ```
    pub fn from_pattern(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            regex: Regex::new(pattern)?,
        })
    }

    /// Creates a query that matches a name exactly.
    ///
    /// # Examples
    ///
    /// ```
    /// # use framework::provides::{NameQuery, Names};
    /// let query = NameQuery::exact("divide");
    /// assert!(query.matches(&Names::from("divide")));
    /// assert!(!query.matches(&Names::from("division")));
    /// ```
    pub fn exact(name: &str) -> Self {
        Self {
            regex: Regex::new(&format!("^{}$", regex::escape(name)))
                .expect("Escaped exact name should be valid regex"),
        }
    }

    /// Checks if this query matches any name in the provided `Names` collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use framework::provides::{NameQuery, Names};
    /// let query = NameQuery::exact("process");
    /// let names = Names::new(["handle", "process", "execute"]);
    /// assert!(query.matches(&names));
    /// ```
    pub fn matches(&self, names: &Names) -> bool {
        names.iter().any(|name| self.regex.is_match(name))
    }
}

impl From<&str> for NameQuery {
    fn from(name: &str) -> Self {
        Self::exact(name)
    }
}

impl From<String> for NameQuery {
    fn from(name: String) -> Self {
        Self::exact(&name)
    }
}

impl From<Regex> for NameQuery {
    fn from(regex: Regex) -> Self {
        Self::new(regex)
    }
}

/// A query for matching against `NamedType` instances, by checking both `TypeId`
/// and name strings.
pub struct NamedTypeQuery {
    type_id: TypeId,
    name_query: NameQuery,
}

impl NamedTypeQuery {
    /// Creates a query for type `T` with the specified name query.
    pub fn new<T: 'static>(name_query: impl Into<NameQuery>) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            name_query: name_query.into(),
        }
    }

    /// Checks if this query matches the given `NamedType`.
    pub fn matches(&self, named_type: &NamedType) -> bool {
        self.type_id == named_type.type_id() && self.name_query.matches(named_type.names())
    }
}

/// A query for matching against function `Signature` instances.
pub struct SignatureQuery {
    return_query: NamedTypeQuery,
    arg_queries: Vec<NamedTypeQuery>,
}

impl SignatureQuery {
    /// Creates a new signature query.
    pub fn new(return_query: NamedTypeQuery, arg_queries: Vec<NamedTypeQuery>) -> Self {
        Self {
            return_query,
            arg_queries,
        }
    }

    /// Checks if this query matches the given `Signature`.
    /// Returns `true` only if all names and types of arguments and return values match.
    pub fn matches(&self, signature: &Signature) -> bool {
        if !self.return_query.matches(signature.return_type()) {
            return false;
        }

        if self.arg_queries.len() != signature.arg_types().len() {
            return false;
        }

        self.arg_queries
            .iter()
            .zip(signature.arg_types())
            .all(|(query, arg_type)| query.matches(arg_type))
    }
}

/// Trait for function types to generate a `SignatureQuery`. See `Signatured`
/// for producing `Signature`s.
pub trait NewSignatureQuery<NameQueries> {
    /// Creates a signature query for this function type.
    fn new_query(queries: NameQueries) -> SignatureQuery;
}

impl<R: 'static> NewSignatureQuery<NameQuery> for fn() -> R {
    fn new_query(queries: NameQuery) -> SignatureQuery {
        SignatureQuery::new(NamedTypeQuery::new::<R>(queries), vec![])
    }
}

impl<R: 'static> NewSignatureQuery<(NameQuery,)> for fn() -> R {
    fn new_query(queries: (NameQuery,)) -> SignatureQuery {
        SignatureQuery::new(NamedTypeQuery::new::<R>(queries.0), vec![])
    }
}

impl<R: 'static, A: 'static> NewSignatureQuery<(NameQuery, NameQuery)> for fn(A) -> R {
    fn new_query(queries: (NameQuery, NameQuery)) -> SignatureQuery {
        SignatureQuery::new(
            NamedTypeQuery::new::<R>(queries.0),
            vec![NamedTypeQuery::new::<A>(queries.1)],
        )
    }
}

impl<R: 'static, A1: 'static, A2: 'static> NewSignatureQuery<(NameQuery, NameQuery, NameQuery)>
    for fn(A1, A2) -> R
{
    fn new_query(queries: (NameQuery, NameQuery, NameQuery)) -> SignatureQuery {
        SignatureQuery::new(
            NamedTypeQuery::new::<R>(queries.0),
            vec![
                NamedTypeQuery::new::<A1>(queries.1),
                NamedTypeQuery::new::<A2>(queries.2),
            ],
        )
    }
}

impl<R: 'static, A1: 'static, A2: 'static, A3: 'static>
    NewSignatureQuery<(NameQuery, NameQuery, NameQuery, NameQuery)> for fn(A1, A2, A3) -> R
{
    fn new_query(queries: (NameQuery, NameQuery, NameQuery, NameQuery)) -> SignatureQuery {
        SignatureQuery::new(
            NamedTypeQuery::new::<R>(queries.0),
            vec![
                NamedTypeQuery::new::<A1>(queries.1),
                NamedTypeQuery::new::<A2>(queries.2),
                NamedTypeQuery::new::<A3>(queries.3),
            ],
        )
    }
}

impl<R: 'static, A1: 'static, A2: 'static, A3: 'static, A4: 'static>
    NewSignatureQuery<(NameQuery, NameQuery, NameQuery, NameQuery, NameQuery)>
    for fn(A1, A2, A3, A4) -> R
{
    fn new_query(
        queries: (NameQuery, NameQuery, NameQuery, NameQuery, NameQuery),
    ) -> SignatureQuery {
        SignatureQuery::new(
            NamedTypeQuery::new::<R>(queries.0),
            vec![
                NamedTypeQuery::new::<A1>(queries.1),
                NamedTypeQuery::new::<A2>(queries.2),
                NamedTypeQuery::new::<A3>(queries.3),
                NamedTypeQuery::new::<A4>(queries.4),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn test_name_query_exact() {
        let query = NameQuery::exact("test");
        let names = Names::from("test");
        assert!(query.matches(&names));

        let other_names = Names::from("other");
        assert!(!query.matches(&other_names));
    }

    #[test]
    fn test_name_query_regex() {
        let query = NameQuery::from_pattern("test.*").unwrap();
        let names1 = Names::from("test123");
        let names2 = Names::from("testing");
        let names3 = Names::from("other");

        assert!(query.matches(&names1));
        assert!(query.matches(&names2));
        assert!(!query.matches(&names3));
    }

    #[test]
    fn test_named_type_query() {
        let query = NamedTypeQuery::new::<i32>("number");
        let named_type1 = NamedType::new::<i32>(Names::from("number"));
        let named_type2 = NamedType::new::<f32>(Names::from("number"));
        let named_type3 = NamedType::new::<i32>(Names::from("other"));

        assert!(query.matches(&named_type1));
        assert!(!query.matches(&named_type2));
        assert!(!query.matches(&named_type3));
    }

    #[test]
    fn test_signature_query_matching() {
        let sig: Signature = <fn(i32, i32) -> i32>::signature((
            Names::from("sum"),
            Names::from("x"),
            Names::from("y"),
        ));

        let matching_query: SignatureQuery = <fn(i32, i32) -> i32>::new_query((
            NameQuery::from("sum"),
            NameQuery::from("x"),
            NameQuery::from("y"),
        ));

        let non_matching_query: SignatureQuery = <fn(i32, i32) -> i32>::new_query((
            NameQuery::from("sum"),
            NameQuery::from("a"),
            NameQuery::from("b"),
        ));

        assert!(matching_query.matches(&sig));
        assert!(!non_matching_query.matches(&sig));
    }
}
