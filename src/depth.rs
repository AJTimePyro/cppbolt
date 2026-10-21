//! Per-construct recursion-depth accounting for the recursive compiler
//! passes.
//!
//! The parser, the semantic analyzer, the code generator and the
//! preprocessor all recurse over the syntax tree.  Every pass keeps one
//! counter *per nesting construct*, so that one construct can no longer
//! exhaust the budget of another: a parenthesis costs one level of the
//! parenthesis budget, a compound block costs one level of the block budget,
//! and a nested call or ternary costs one level of the expression budget.
//!
//! The limits are measured on the WebAssembly-hosted build under Node/V8,
//! whose engine shadow stack is much smaller than a native thread with an
//! explicitly large stack, and are set so that the diagnostic always comes
//! before the stack runs out.  The measurements on the WebAssembly build are
//! in `docs/known-limitations.md`; the native build runs the compiler on a
//! 64 MiB thread (src/main.rs) and the WebAssembly build is linked with an
//! 8 MiB linear-memory stack (`.cargo/config.toml`), so the same limits apply
//! to both hosts.

use crate::diagnostics::{CompileError, ErrorKind};
use std::cell::Cell;
use std::rc::Rc;

/// Maximum nesting depth of expressions.
///
/// A parenthesized ternary costs two expression levels (the conditional and
/// the expression it contains), so the limit is twice the nesting limit of
/// the parenthesized forms.
pub const MAX_EXPRESSION_DEPTH: usize = 4000;

/// Maximum nesting depth of parentheses.
pub const MAX_PARENTHESIS_DEPTH: usize = 1600;

/// Maximum nesting depth of call argument lists.
pub const MAX_CALL_DEPTH: usize = 1500;

/// Maximum nesting depth of statements.
pub const MAX_STATEMENT_DEPTH: usize = 1600;

/// Maximum nesting depth of compound blocks.
pub const MAX_BLOCK_DEPTH: usize = 1600;

/// Maximum nesting depth of declarations.
pub const MAX_DECLARATION_DEPTH: usize = 1600;

/// Maximum nesting depth of type expressions.
pub const MAX_TYPE_DEPTH: usize = 1600;

/// Maximum nesting depth of a preprocessor condition.
pub const MAX_PREPROCESSOR_DEPTH: usize = 1600;

/// Diagnostic text emitted when a depth limit is exceeded.
pub const DEPTH_DIAGNOSTIC: &str = "Nesting depth exceeds the supported limit";

/// One nesting counter with its own limit.
#[derive(Clone, Debug)]
pub struct DepthCounter {
    depth: Rc<Cell<usize>>,
    limit: usize,
}

impl DepthCounter {
    /// Creates a counter that accepts `limit` levels of nesting.
    pub fn new(limit: usize) -> Self {
        Self {
            depth: Rc::new(Cell::new(0)),
            limit,
        }
    }

    /// Reserves one level of this counter.
    ///
    /// The returned guard releases the level when it is dropped, including on
    /// an early error return.
    pub fn enter(&self, line: usize, col: usize) -> Result<DepthGuard, CompileError> {
        let depth = self.depth.get() + 1;
        if depth > self.limit {
            return Err(CompileError::new(DEPTH_DIAGNOSTIC, line, col).with_kind(ErrorKind::Syntax));
        }
        self.depth.set(depth);
        Ok(DepthGuard {
            depth: Rc::clone(&self.depth),
        })
    }
}

/// Releases one level of a counter, including on an early error return.
pub struct DepthGuard {
    depth: Rc<Cell<usize>>,
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        let depth = self.depth.get();
        self.depth.set(depth.saturating_sub(1));
    }
}

/// The nesting counters of one compiler pass.
///
/// Each construct has its own counter, so a deeply nested parenthesis cannot
/// use up the budget of the statements or of the expressions.
#[derive(Clone, Debug)]
pub struct DepthCounters {
    expressions: DepthCounter,
    parentheses: DepthCounter,
    calls: DepthCounter,
    statements: DepthCounter,
    blocks: DepthCounter,
    declarations: DepthCounter,
    types: DepthCounter,
}

impl DepthCounters {
    /// Creates counters with the documented limits.
    pub fn new() -> Self {
        let expressions = DepthCounter::new(MAX_EXPRESSION_DEPTH);
        let parentheses = DepthCounter::new(MAX_PARENTHESIS_DEPTH);
        let calls = DepthCounter::new(MAX_CALL_DEPTH);
        let statements = DepthCounter::new(MAX_STATEMENT_DEPTH);
        let blocks = DepthCounter::new(MAX_BLOCK_DEPTH);
        let declarations = DepthCounter::new(MAX_DECLARATION_DEPTH);
        let types = DepthCounter::new(MAX_TYPE_DEPTH);
        Self {
            expressions,
            parentheses,
            calls,
            statements,
            blocks,
            declarations,
            types,
        }
    }

    /// The counter of nested expressions.
    pub fn expressions(&self) -> &DepthCounter {
        &self.expressions
    }

    /// The counter of nested parentheses.
    pub fn parentheses(&self) -> &DepthCounter {
        &self.parentheses
    }

    /// The counter of nested call argument lists.
    pub fn calls(&self) -> &DepthCounter {
        &self.calls
    }

    /// The counter of nested statements.
    pub fn statements(&self) -> &DepthCounter {
        &self.statements
    }

    /// The counter of nested compound blocks.
    pub fn blocks(&self) -> &DepthCounter {
        &self.blocks
    }

    /// The counter of nested declarations.
    pub fn declarations(&self) -> &DepthCounter {
        &self.declarations
    }

    /// The counter of nested type expressions.
    pub fn types(&self) -> &DepthCounter {
        &self.types
    }
}

impl Default for DepthCounters {
    fn default() -> Self {
        Self::new()
    }
}
