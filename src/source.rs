//! Source identity and location primitives shared by every stage.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub file: FileId,
    pub line: u32,
    pub col: u32,
    pub len: u32,
}

impl Span {
    pub const fn new(file: FileId, line: u32, col: u32, len: u32) -> Self {
        Self {
            file,
            line,
            col,
            len,
        }
    }

    pub const fn point(file: FileId, line: u32, col: u32) -> Self {
        Self::new(file, line, col, 1)
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file{}", self.0)
    }
}

/// Supplies non-embedded include files.
///
/// The core never touches the filesystem; the native CLI installs a provider
/// and browser/Node callers simply leave it unset.
pub trait IncludeProvider {
    fn read_include(&self, name: &str) -> Option<String>;

    /// Resolves an include directive and returns its canonical name together
    /// with its contents.
    ///
    /// `including_file` is the resolved name of the file containing the
    /// directive and `quoted` distinguishes `"..."` from `<...>`. The returned
    /// name is used for diagnostics, `#pragma once`, and cycle detection.
    /// Providers that only look names up globally can keep the default.
    fn resolve_include(
        &self,
        name: &str,
        _including_file: Option<&str>,
        _quoted: bool,
    ) -> Option<ResolvedInclude> {
        let content = self.read_include(name)?;
        Some(ResolvedInclude {
            name: name.to_string(),
            content,
        })
    }
}

/// One include file resolved by an [`IncludeProvider`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedInclude {
    pub name: String,
    pub content: String,
}

/// A provider that resolves nothing outside the embedded headers.
pub struct NoIncludes;

impl IncludeProvider for NoIncludes {
    fn read_include(&self, _name: &str) -> Option<String> {
        None
    }
}
