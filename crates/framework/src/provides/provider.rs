use super::{NameQuery, Provides};

/// A read-only collection of `Provides` from ancestors in a procedural node hierarchy. Each
/// `Provides` allows for querying the functions registered by that ancestor. If any function
/// signatures overlap, the function returned will be that registered by the closest ancestor.
///
/// # Examples
/// ```
/// # use prockit_framework::{Provider, Provides, Names, NameQuery};
/// let mut parent_provides = Provides::new();
/// parent_provides.add_0(|| 42i32, Names::from("value"));
/// let mut grandparent_provides = Provides::new();
/// grandparent_provides.add_0(|| 100i32, Names::from("value"));
///
/// let provider = Provider::hierarchy(vec![grandparent_provides, parent_provides]);
///
/// let value = provider.query_0::<i32>(NameQuery::exact("value")).unwrap();
/// assert_eq!(value(), 42);
/// ```
pub struct Provider<'a> {
    hierarchy: Vec<Provides<'a>>,
}

impl<'a> Provider<'a> {
    /// Creates an empty `Provider` for root nodes with no ancestors.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::Provider;
    /// let provider = Provider::empty();
    /// ```
    pub fn empty() -> Self {
        Self {
            hierarchy: Vec::new(),
        }
    }

    /// Creates a new `Provider` from a chain of `Provides`, in order of increasing closeness of
    /// ancestor `Provides`.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provider, Provides};
    /// let parent_provides = Provides::new();
    /// let grandparent_provides = Provides::new();
    /// let provider = Provider::hierarchy(vec![grandparent_provides, parent_provides]);
    /// ```
    pub fn hierarchy(hierarchy: Vec<Provides<'a>>) -> Self {
        Self { hierarchy }
    }

    /// Extends this `Provider` with a new `Provides` at the lowest hierarchical spot, meaning
    /// it will be considered the new closest ancestor.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provider, Provides, Names};
    /// let mut parent_provides = Provides::new();
    /// parent_provides.add_0(|| 42i32, Names::from("value"));
    /// let mut provider = Provider::empty();
    /// provider.push(parent_provides);
    /// ```
    pub fn push(&mut self, provides: Provides<'a>) {
        self.hierarchy.push(provides);
    }

    /// Queries for a function with zero arguments. If function signatures from multiple
    /// ancestor's `Provides` overlap, the function registered by the closest ancestor will be
    /// returned.
    pub fn query_0<R: 'static>(
        &self,
        r_query: NameQuery,
    ) -> Option<&(dyn Fn() -> R + Send + Sync + 'a)> {
        self.hierarchy
            .iter()
            .rev()
            .find_map(|provides| provides.query_0::<R>(r_query.clone()))
    }

    /// Queries for a function with one argument. If function signatures from multiple
    /// ancestor's `Provides` overlap, the function registered by the closest ancestor will be
    /// returned.
    pub fn query_1<R: 'static, A: 'static>(
        &self,
        r_query: NameQuery,
        a_query: NameQuery,
    ) -> Option<&(dyn Fn(A) -> R + Send + Sync + 'a)> {
        self.hierarchy
            .iter()
            .rev()
            .find_map(|provides| provides.query_1::<R, A>(r_query.clone(), a_query.clone()))
    }

    /// Queries for a function with two arguments. If function signatures from multiple
    /// ancestor's `Provides` overlap, the function registered by the closest ancestor will be
    /// returned.
    pub fn query_2<R: 'static, A1: 'static, A2: 'static>(
        &self,
        r_query: NameQuery,
        a1_query: NameQuery,
        a2_query: NameQuery,
    ) -> Option<&(dyn Fn(A1, A2) -> R + Send + Sync + 'a)> {
        self.hierarchy.iter().rev().find_map(|provides| {
            provides.query_2::<R, A1, A2>(r_query.clone(), a1_query.clone(), a2_query.clone())
        })
    }

    /// Queries for a function with three arguments. If function signatures from multiple
    /// ancestor's `Provides` overlap, the function registered by the closest ancestor will be
    /// returned.
    pub fn query_3<R: 'static, A1: 'static, A2: 'static, A3: 'static>(
        &self,
        r_query: NameQuery,
        a1_query: NameQuery,
        a2_query: NameQuery,
        a3_query: NameQuery,
    ) -> Option<&(dyn Fn(A1, A2, A3) -> R + Send + Sync + 'a)> {
        self.hierarchy.iter().rev().find_map(|provides| {
            provides.query_3::<R, A1, A2, A3>(
                r_query.clone(),
                a1_query.clone(),
                a2_query.clone(),
                a3_query.clone(),
            )
        })
    }

    /// Queries for a function with four arguments. If function signatures from multiple
    /// ancestor's `Provides` overlap, the function registered by the closest ancestor will be
    /// returned.
    pub fn query_4<R: 'static, A1: 'static, A2: 'static, A3: 'static, A4: 'static>(
        &self,
        r_query: NameQuery,
        a1_query: NameQuery,
        a2_query: NameQuery,
        a3_query: NameQuery,
        a4_query: NameQuery,
    ) -> Option<&(dyn Fn(A1, A2, A3, A4) -> R + Send + Sync + 'a)> {
        self.hierarchy.iter().rev().find_map(|provides| {
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

impl<'a> Default for Provider<'a> {
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

        let provider = Provider::hierarchy(vec![provides]);
        let result = provider.query_0::<i32>(NameQuery::exact("value"));
        assert!(result.is_some());
        assert_eq!(result.unwrap()(), 42);
    }

    #[test]
    fn test_provider_precedence() {
        let mut grandparent_provides = Provides::new();
        grandparent_provides.add_0(|| 100i32, Names::from("value"));

        let mut parent_provides = Provides::new();
        parent_provides.add_0(|| 42i32, Names::from("value"));

        let provider = Provider::hierarchy(vec![grandparent_provides, parent_provides]);
        let result = provider.query_0::<i32>(NameQuery::exact("value"));
        assert!(result.is_some());
        assert_eq!(result.unwrap()(), 42);
    }

    #[test]
    fn test_provider_fallback_to_ancestor() {
        let mut grandparent_provides = Provides::new();
        grandparent_provides.add_0(|| 0i32, Names::from("zero"));

        let mut parent_provides = Provides::new();
        parent_provides.add_1(|x: i32| x * 0, Names::from("zero"), Names::from("x"));

        let provider = Provider::hierarchy(vec![grandparent_provides, parent_provides]);
        let result = provider.query_0::<i32>(NameQuery::exact("zero"));
        assert!(result.is_some());
        assert_eq!(result.unwrap()(), 0);
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
        assert_eq!(result.unwrap()(), 42);
    }
}
