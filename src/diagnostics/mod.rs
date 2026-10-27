//! The single diagnostic type used across the compiler.

use std::fmt;

/// Categorizes the origin of a compilation diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Tokenization or preprocessor error.
    Lexical,
    /// Grammar or parser error.
    Syntax,
    /// Type checking, resolution, or layout error.
    Semantic,
    /// Feature outside the supported C++ subset.
    Unsupported,
    /// Internal invariant failure or resource limit reached.
    Internal,
}

/// A compilation diagnostic with source coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    /// The diagnostic category.
    pub kind: ErrorKind,
    /// The human-readable diagnostic message.
    pub message: String,
    /// The originating source file path.
    pub file: String,
    /// 1-based source line.
    pub line: usize,
    /// 1-based source column.
    pub col: usize,
}

impl CompileError {
    /// Creates a new semantic error at the given line and column.
    pub fn new(message: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            kind: ErrorKind::Semantic,
            message: message.into(),
            file: String::new(),
            line,
            col,
        }
    }

    /// Sets the diagnostic kind.
    pub fn with_kind(mut self, kind: ErrorKind) -> Self {
        self.kind = kind;
        self
    }

    /// Attaches a source file path to the diagnostic.
    pub fn at(mut self, file: impl Into<String>, line: usize, col: usize) -> Self {
        self.file = file.into();
        self.line = line;
        self.col = col;
        self
    }

    /// Creates an internal compiler error diagnostic.
    pub fn internal(message: impl Into<String>, line: usize, col: usize) -> Self {
        Self::new(
            format!("internal compiler error: {}", message.into()),
            line,
            col,
        )
        .with_kind(ErrorKind::Internal)
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.file.is_empty() {
            write!(f, "{}:{}: error: {}", self.line, self.col, self.message)
        } else {
            write!(
                f,
                "{}:{}:{}: error: {}",
                self.file, self.line, self.col, self.message
            )
        }
    }
}

impl std::error::Error for CompileError {}

/// Reports a broken internal invariant through the same diagnostic path as
/// user errors so browser builds never unwind on an `unwrap`.
#[macro_export]
macro_rules! internal_error {
    ($line:expr, $col:expr, $($arg:tt)*) => {
        $crate::diagnostics::CompileError::internal(format!($($arg)*), $line, $col)
    };
}
