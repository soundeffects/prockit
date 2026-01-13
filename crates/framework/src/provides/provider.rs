use super::{NameQuery, Provides};
use crate::Space;

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
pub struct Provider<'a, S: Space> {
    hierarchy: Vec<Provides<'a, S>>,
}

impl<'a, S: Space> Provider<'a, S> {
    /// Creates an empty `Provider` for root nodes with no ancestors.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provider, RealSpace};
    /// let provider = Provider::<RealSpace>::empty();
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
    /// # use prockit_framework::{Provider, Provides, RealSpace};
    /// let parent_provides = Provides::<RealSpace>::new();
    /// let grandparent_provides = Provides::<RealSpace>::new();
    /// let provider = Provider::hierarchy(vec![grandparent_provides, parent_provides]);
    /// ```
    pub fn hierarchy(hierarchy: Vec<Provides<'a, S>>) -> Self {
        Self { hierarchy }
    }

    /// Extends this `Provider` with a new `Provides` at the lowest hierarchical spot, meaning
    /// it will be considered the new closest ancestor.
    ///
    /// # Examples
    /// ```
    /// # use prockit_framework::{Provider, Provides, Names, RealSpace};
    /// let parent_provides = Provides::<RealSpace>::new();
    /// let mut provider = Provider::empty();
    /// provider.push(parent_provides);
    /// ```
    pub fn push(&mut self, provides: Provides<'a, S>) {
        self.hierarchy.push(provides);
    }

    /// Queries for a function with the given function names and return type. If function
    /// signatures from multiple ancestor's `Provides` overlap, the function registered by the
    /// closest ancestor will be returned.
    pub fn query<R: 'static>(
        &self,
        names: impl Into<NameQuery>,
    ) -> Option<&(dyn Fn(&S::Position) -> R + Send + Sync + 'a)> {
        let names = names.into();
        self.hierarchy
            .iter()
            .rev()
            .find_map(|provides| provides.query::<R>(&names))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RealSpace;
    use bevy::prelude::*;

    #[test]
    fn test_provider_empty() {
        let provider = Provider::<RealSpace>::empty();
        let result = provider.query::<()>("something");
        assert!(result.is_none());
    }

    #[test]
    fn test_provider_single_provides() {
        let mut provides = Provides::<RealSpace>::new();
        provides.add("constant answer", |_position: &Vec3| 42);
        let provider = Provider::hierarchy(vec![provides]);
        let result = provider.query::<i32>("constant answer");
        assert!(result.is_some());
        let constant_answer = result.unwrap();
        assert_eq!(constant_answer(&Vec3::ZERO), 42);
    }

    #[test]
    fn test_provider_precedence() {
        let mut grandparent_provides = Provides::<RealSpace>::new();
        grandparent_provides.add("constant answer", |_position: &Vec3| 100);
        let mut parent_provides = Provides::new();
        parent_provides.add("constant answer", |_position: &Vec3| 42);
        let provider = Provider::hierarchy(vec![grandparent_provides, parent_provides]);
        let result = provider.query::<i32>("constant answer");
        assert!(result.is_some());
        let constant_answer = result.unwrap();
        assert_eq!(constant_answer(&Vec3::ZERO), 42);
    }

    #[test]
    fn test_provider_fallback_to_ancestor() {
        let mut grandparent_provides = Provides::<RealSpace>::new();
        grandparent_provides.add("zero", |_position: &Vec3| 0);
        let mut parent_provides = Provides::new();
        parent_provides.add("zero", |_position: &Vec3| 0.0);
        let provider = Provider::hierarchy(vec![grandparent_provides, parent_provides]);
        let result = provider.query::<i32>("zero");
        assert!(result.is_some());
        let zero = result.unwrap();
        assert_eq!(zero(&Vec3::ZERO), 0);
    }

    #[test]
    fn test_provider_push() {
        let mut provider = Provider::<RealSpace>::empty();
        let mut grandparent_provides = Provides::new();
        grandparent_provides.add("constant answer", |_position: &Vec3| 100);
        provider.push(grandparent_provides);
        let mut parent_provides = Provides::new();
        parent_provides.add("constant answer", |_position: &Vec3| 42);
        provider.push(parent_provides);
        let result = provider.query::<i32>("constant answer");
        assert!(result.is_some());
        let constant_answer = result.unwrap();
        assert_eq!(constant_answer(&Vec3::ZERO), 42);
    }
}
