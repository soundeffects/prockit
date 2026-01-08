use super::{NameQuery, Provides};

/// A read-only collection of `Provides` from ancestors in a procedural node hierarchy.
/// Each `Provides` allows for querying the functions registered by that ancestor.
///
/// When querying functions, the `Provider` searches through the chain of ancestor
/// `Provides` in order, returning the first match found. This means functions from
/// closer ancestors (direct parent) take precedence over those from more distant
/// ancestors (root).
///
/// # Examples
/// ```
/// # use prockit_framework::{Provider, Provides, Names, NameQuery};
/// // Create provides from different ancestors
/// let mut parent_provides = Provides::new();
/// parent_provides.add_0(|| 42i32, Names::from("value"));
///
/// let mut grandparent_provides = Provides::new();
/// grandparent_provides.add_0(|| 100i32, Names::from("value"));
///
/// // Parent's provides comes first (closer ancestor)
/// let provider = Provider::new(vec![parent_provides, grandparent_provides]);
///
/// // Query returns the parent's version (42) because it's closer
/// let value = provider.query_0::<i32>(NameQuery::exact("value")).unwrap();
/// assert_eq!(value(), 42);
/// ```
pub struct Provider {
    /// Ordered from closest ancestor (index 0) to root ancestor (last index)
    provides_chain: Vec<Provides>,
}

impl Provider {
    /// Creates an empty `Provider` for root nodes with no ancestors.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::Provider;
    /// let provider = Provider::empty();
    /// ```
    pub fn empty() -> Self {
        Self {
            provides_chain: Vec::new(),
        }
    }

    /// Creates a new `Provider` from a chain of `Provides`.
    ///
    /// The last element should be the direct parent's `Provides`,
    /// with preceding elements being progressively more distant ancestors,
    /// until the root at the first position.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provider, Provides};
    /// let parent_provides = Provides::new();
    /// let grandparent_provides = Provides::new();
    /// let provider = Provider::new(vec![parent_provides, grandparent_provides]);
    /// ```
    pub fn new(provides_chain: Vec<Provides>) -> Self {
        Self { provides_chain }
    }

    /// Extends this `Provider` with a new `Provides` with priority, meaning it will be
    /// considered the closest ancestor `Provides` and will satisfy queries first.
    ///
    /// This is useful for incremental construction during hierarchy traversal,
    /// where you build up the `Provider` as you descend through the node tree.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::{Provider, Provides, Names};
    /// let provider = Provider::empty();
    /// let mut parent_provides = Provides::new();
    /// parent_provides.add_0(|| 42i32, Names::from("value"));
    /// provider.push(parent_provides);
    ///
    /// // Now parent_provider has the parent's provides at the front
    /// ```
    pub fn push(&mut self, provides: Provides) {
        self.provides_chain.push(provides);
    }

    /// Queries for a function with zero arguments. Searches through the ancestor chain in
    /// order, returning the first match.
    pub fn query_0<R: 'static>(
        &self,
        r_query: NameQuery,
    ) -> Option<&(dyn Fn() -> R + Send + Sync)> {
        self.provides_chain
            .iter()
            .rev()
            .find_map(|provides| provides.query_0::<R>(r_query.clone()))
    }

    /// Queries for a function with one argument. Searches through the ancestor chain in order,
    /// returning the first match.
    pub fn query_1<R: 'static, A: 'static>(
        &self,
        r_query: NameQuery,
        a_query: NameQuery,
    ) -> Option<&(dyn Fn(A) -> R + Send + Sync)> {
        self.provides_chain
            .iter()
            .rev()
            .find_map(|provides| provides.query_1::<R, A>(r_query.clone(), a_query.clone()))
    }

    /// Queries for a function with two arguments. Searches through the ancestor chain in order
    /// returning the first match.
    pub fn query_2<R: 'static, A1: 'static, A2: 'static>(
        &self,
        r_query: NameQuery,
        a1_query: NameQuery,
        a2_query: NameQuery,
    ) -> Option<&(dyn Fn(A1, A2) -> R + Send + Sync)> {
        self.provides_chain.iter().rev().find_map(|provides| {
            provides.query_2::<R, A1, A2>(r_query.clone(), a1_query.clone(), a2_query.clone())
        })
    }

    /// Queries for a function with three arguments. Searches through the ancestor chain in
    /// order, returning the first match.
    pub fn query_3<R: 'static, A1: 'static, A2: 'static, A3: 'static>(
        &self,
        r_query: NameQuery,
        a1_query: NameQuery,
        a2_query: NameQuery,
        a3_query: NameQuery,
    ) -> Option<&(dyn Fn(A1, A2, A3) -> R + Send + Sync)> {
        self.provides_chain.iter().rev().find_map(|provides| {
            provides.query_3::<R, A1, A2, A3>(
                r_query.clone(),
                a1_query.clone(),
                a2_query.clone(),
                a3_query.clone(),
            )
        })
    }

    /// Queries for a function with four arguments. Searches through the ancestor chain in
    /// order, returning the first match.
    pub fn query_4<R: 'static, A1: 'static, A2: 'static, A3: 'static, A4: 'static>(
        &self,
        r_query: NameQuery,
        a1_query: NameQuery,
        a2_query: NameQuery,
        a3_query: NameQuery,
        a4_query: NameQuery,
    ) -> Option<&(dyn Fn(A1, A2, A3, A4) -> R + Send + Sync)> {
        self.provides_chain.iter().rev().find_map(|provides| {
            provides.query_4::<R, A1, A2, A3, A4>(
                r_query.clone(),
                a1_query.clone(),
                a2_query.clone(),
                a3_query.clone(),
                a4_query.clone(),
            )
        })
    }
}

impl Default for Provider {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_provider_empty() {
        let provider = Provider::empty();
        let result = provider.query_0::<i32>(NameQuery::exact("value"));
        assert!(result.is_none());
    }

    #[test]
    fn test_provider_single_provides() {
        let mut provides = Provides::new();
        provides.add_0(|| 42i32, Names::from("value"));

        let provider = Provider::new(vec![provides]);
        let result = provider.query_0::<i32>(NameQuery::exact("value"));
        assert!(result.is_some());
        assert_eq!(result.unwrap()(), 42);
    }

    #[test]
    fn test_provider_precedence() {
        let mut parent_provides = Provides::new();
        parent_provides.add_0(|| 42i32, Names::from("value"));

        let mut grandparent_provides = Provides::new();
        grandparent_provides.add_0(|| 100i32, Names::from("value"));

        let provider = Provider::new(vec![grandparent_provides, parent_provides]);
        let result = provider.query_0::<i32>(NameQuery::exact("value"));
        assert!(result.is_some());
        // Should get parent's value (42), not grandparent's (100)
        assert_eq!(result.unwrap()(), 42);
    }

    #[test]
    fn test_provider_fallback_to_ancestor() {
        let parent_provides = Provides::new(); // Empty, no "value" function

        let mut grandparent_provides = Provides::new();
        grandparent_provides.add_0(|| 100i32, Names::from("value"));

        let provider = Provider::new(vec![grandparent_provides, parent_provides]);
        let result = provider.query_0::<i32>(NameQuery::exact("value"));
        assert!(result.is_some());
        // Should get grandparent's value since parent doesn't have it
        assert_eq!(result.unwrap()(), 100);
    }

    #[test]
    fn test_provider_push() {
        let mut provider = Provider::empty();

        let mut grandparent_provides = Provides::new();
        grandparent_provides.add_0(|| 100i32, Names::from("value"));
        provider.push(grandparent_provides);

        let mut parent_provides = Provides::new();
        parent_provides.add_0(|| 42i32, Names::from("value"));
        provider.push(parent_provides);

        let result = provider.query_0::<i32>(NameQuery::exact("value"));
        assert!(result.is_some());
        // Should get parent's value (42) since it's at the front
        assert_eq!(result.unwrap()(), 42);
    }
}
