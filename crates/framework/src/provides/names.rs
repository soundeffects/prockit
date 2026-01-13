use regex::Regex;

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

/// A query for matching against name collections using regex patterns or exact
/// matches.
///
/// # Examples
///
/// ```
/// # use prockit_framework::{NameQuery, Names};
/// let exact = NameQuery::exact("add");
/// let names = Names::from("add");
/// assert!(exact.matches(&names));
///
/// let pattern = NameQuery::from_pattern("get_.*").unwrap();
/// let getter_names = Names::from("get_value");
/// assert!(pattern.matches(&getter_names));
/// ```
#[derive(Clone)]
pub struct NameQuery {
    regex: Regex,
}

impl NameQuery {
    /// Creates a new name query from a compiled regex pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prockit_framework::{NameQuery, Names};
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
    /// # use prockit_framework::{NameQuery, Names};
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
    /// # use prockit_framework::{NameQuery, Names};
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
    /// # use prockit_framework::{NameQuery, Names};
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
