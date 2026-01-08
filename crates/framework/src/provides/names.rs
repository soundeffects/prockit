use std::any::TypeId;

/// A collection of string names that can be associated with types or functions.
/// A single entity may be referenced by multiple aliases, improving discoverability
/// of `Provides` entries.
///
/// # Examples
///
/// ```
/// # use prockit_framework::Names;
/// let names = Names::new(["add", "sum", "plus"]);
/// assert!(names.contains("add"));
/// assert!(names.contains("sum"));
/// assert!(!names.contains("subtract"));
///
/// // Can also be created from a single string
/// let single = Names::from("multiply");
/// assert!(single.contains("multiply"));
/// ```
#[derive(Clone, Debug)]
pub struct Names {
    names: Vec<String>,
}

impl Names {
    /// Creates a new `Names` collection from an iterable of string-like items.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::Names;
    /// let names = Names::new(["foo", "bar"]);
    /// assert!(names.contains("foo"));
    ///
    /// let from_strings = Names::new(vec!["a".to_string(), "b".to_string()]);
    /// assert!(from_strings.contains("b"));
    /// ```
    pub fn new(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            names: names.into_iter().map(|s| s.into()).collect(),
        }
    }

    /// Returns an iterator over the names as string slices.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::Names;
    /// let names = Names::new(["alpha", "beta", "gamma"]);
    /// let collected: Vec<&str> = names.iter().collect();
    /// assert_eq!(collected, vec!["alpha", "beta", "gamma"]);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.names.iter().map(|s| s.as_str())
    }

    /// Checks if the collection contains a specific name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::Names;
    /// let names = Names::new(["read", "write", "execute"]);
    /// assert!(names.contains("read"));
    /// assert!(!names.contains("delete"));
    /// ```
    pub fn contains(&self, name: &str) -> bool {
        self.names.iter().any(|n| n == name)
    }
}

impl From<&str> for Names {
    fn from(name: &str) -> Self {
        Self::new([name])
    }
}

impl From<String> for Names {
    fn from(name: String) -> Self {
        Self::new([name])
    }
}

impl<const N: usize> From<[&str; N]> for Names {
    fn from(names: [&str; N]) -> Self {
        Self::new(names)
    }
}

impl<const N: usize> From<[String; N]> for Names {
    fn from(names: [String; N]) -> Self {
        Self::new(names)
    }
}

/// A `TypeId` paired with a set of `Names`, enabling type-safe name-based
/// lookups for `Provides`.
#[derive(Clone, Debug)]
pub struct NamedType {
    type_id: TypeId,
    names: Names,
}

impl NamedType {
    /// Creates a new `NamedType` for type `T` with the given names.
    pub fn new<T: 'static>(names: impl Into<Names>) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            names: names.into(),
        }
    }

    /// Returns the `TypeId` of the associated type.
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns a reference to the names collection.
    pub fn names(&self) -> &Names {
        &self.names
    }
}

/// Represents a function signature with `NamedType`s for the return value and arguments.
#[derive(Clone, Debug)]
pub struct Signature {
    return_type: NamedType,
    arg_types: Vec<NamedType>,
}

impl Signature {
    /// Creates a new function signature with the specified return and argument
    /// `NamedType`s.
    pub fn new(return_type: NamedType, arg_types: Vec<NamedType>) -> Self {
        Self {
            return_type,
            arg_types,
        }
    }

    /// Returns the return type of this function signature.
    pub fn return_type(&self) -> &NamedType {
        &self.return_type
    }

    /// Returns a slice of the argument types for this function signature.
    pub fn arg_types(&self) -> &[NamedType] {
        &self.arg_types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    #[test]
    fn test_single_name() {
        let names = Names::from("foo");
        assert!(names.contains("foo"));
        assert!(!names.contains("bar"));
    }

    #[test]
    fn test_multiple_names() {
        let names = Names::new(["add", "sum", "plus"]);
        assert!(names.contains("add"));
        assert!(names.contains("sum"));
        assert!(names.contains("plus"));
        assert!(!names.contains("subtract"));
    }

    #[test]
    fn test_named_type() {
        let named_type = NamedType::new::<i32>(Names::from("integer"));
        assert_eq!(named_type.type_id(), TypeId::of::<i32>());
        assert!(named_type.names().contains("integer"));
    }

    #[test]
    fn test_signature_zero_args() {
        let sig = Signature::new(NamedType::new::<f32>(Names::from("something")), vec![]);

        assert_eq!(sig.return_type().type_id(), TypeId::of::<f32>());
        assert_eq!(sig.arg_types().len(), 0);
    }
}
