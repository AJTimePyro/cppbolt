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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub id: FileId,
    pub name: String,
    pub text: String,
}

#[derive(Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn add_file(&mut self, name: impl Into<String>, text: impl Into<String>) -> FileId {
        let id = FileId(self.files.len() as u32);
        self.files.push(SourceFile {
            id,
            name: name.into(),
            text: text.into(),
        });
        id
    }

    pub fn file(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(id.0 as usize)
    }

    pub fn name(&self, id: FileId) -> Option<&str> {
        self.file(id).map(|file| file.name.as_str())
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
}

/// A provider that resolves nothing outside the embedded headers.
pub struct NoIncludes;

impl IncludeProvider for NoIncludes {
    fn read_include(&self, _name: &str) -> Option<String> {
        None
    }
}
