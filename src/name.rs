//! `::`-qualified names.
//!
//! The compiler treats the `std` namespace as implicit: a declaration written
//! as `std::string` is also visible as `string`, and vice versa. Every symbol
//! table therefore has to reason about both spellings. `QualifiedName`
//! centralises that rule so lookups cannot drift apart.

use std::borrow::Cow;

const STD: &str = "std::";

/// Storage name for a function-local static. The source location keeps statics
/// with the same spelling in different scopes from sharing one global.
pub fn static_local_name(function: &str, line: usize, col: usize, name: &str) -> String {
    format!("__static_{function}_{line}_{col}_{name}")
}

/// A name as written in the source, with helpers for the implicit `std`
/// namespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct QualifiedName<'a>(&'a str);

impl<'a> QualifiedName<'a> {
    /// Creates a new qualified name reference from raw text.
    pub const fn new(raw: &'a str) -> Self {
        Self(raw)
    }

    /// Returns the underlying raw name string slice.
    pub const fn as_str(self) -> &'a str {
        self.0
    }

    /// Returns true if this name begins with `std::`.
    pub fn in_std(self) -> bool {
        self.0.starts_with(STD)
    }

    /// True for names that do not contain a `::` qualifier.
    pub fn is_global(self) -> bool {
        !self.0.contains("::")
    }

    /// The name with the implicit `std::` qualifier removed.
    pub fn plain(self) -> &'a str {
        if self.in_std() {
            &self.0[STD.len()..]
        } else {
            self.0
        }
    }

    /// The spelling in the `std` namespace, adding the qualifier when absent.
    pub fn std_name(self) -> Cow<'a, str> {
        if self.in_std() {
            Cow::Borrowed(self.0)
        } else {
            Cow::Owned(format!("{STD}{}", self.0))
        }
    }

    /// The other accepted spelling: `plain` for a `std::` name, otherwise the
    /// `std::`-qualified name.
    pub fn alternate(self) -> Cow<'a, str> {
        if self.in_std() {
            Cow::Borrowed(self.plain())
        } else {
            self.std_name()
        }
    }

    /// The `std::` alias if this name can be spelled both ways. Class-scoped
    /// names other than `std::` have no implicit alias.
    pub fn implicit_alias(self) -> Option<Cow<'a, str>> {
        if self.0.is_empty() {
            return None;
        }
        if self.in_std() {
            let plain = self.plain();
            if plain.is_empty() {
                return None;
            }
            Some(Cow::Borrowed(plain))
        } else if self.is_global() {
            Some(self.std_name())
        } else {
            None
        }
    }

    /// True when both names refer to the same symbol.
    pub fn matches(self, other: Self) -> bool {
        self.plain() == other.plain()
    }
}

#[cfg(test)]
mod tests {
    use super::{QualifiedName, static_local_name};

    #[test]
    fn strips_and_adds_the_std_qualifier() {
        let qualified = QualifiedName::new("std::string");
        assert!(qualified.in_std());
        assert_eq!(qualified.plain(), "string");
        assert_eq!(qualified.alternate(), "string");
        assert_eq!(qualified.std_name(), "std::string");

        let plain = QualifiedName::new("string");
        assert!(!plain.in_std());
        assert!(plain.is_global());
        assert_eq!(plain.plain(), "string");
        assert_eq!(plain.alternate(), "std::string");
    }

    #[test]
    fn leaves_class_scoped_names_alone() {
        let name = QualifiedName::new("Widget::draw");
        assert!(!name.in_std());
        assert!(!name.is_global());
        assert_eq!(name.implicit_alias(), None);
        assert_eq!(name.alternate(), "std::Widget::draw");
    }

    #[test]
    fn matches_across_the_implicit_qualifier() {
        assert!(QualifiedName::new("std::printf").matches(QualifiedName::new("printf")));
        assert!(!QualifiedName::new("printf").matches(QualifiedName::new("puts")));
    }

    #[test]
    fn static_names_distinguish_function_and_source_location() {
        assert_ne!(
            static_local_name("first", 3, 5, "value"),
            static_local_name("second", 3, 5, "value")
        );
        assert_ne!(
            static_local_name("first", 3, 5, "value"),
            static_local_name("first", 4, 5, "value")
        );
    }
}
