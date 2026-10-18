//! The single diagnostic type used across the compiler.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Lexical,
    Syntax,
    Semantic,
    Unsupported,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    pub kind: ErrorKind,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub col: usize,
}

impl CompileError {
    pub fn new(message: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            kind: ErrorKind::Semantic,
            message: message.into(),
            file: String::new(),
            line,
            col,
        }
    }

    pub fn with_kind(mut self, kind: ErrorKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn at(mut self, file: impl Into<String>, line: usize, col: usize) -> Self {
        self.file = file.into();
        self.line = line;
        self.col = col;
        self
    }

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
