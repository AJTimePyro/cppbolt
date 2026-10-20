//! Shared recursion-depth accounting for the recursive compiler passes.
//!
//! Parser, semantic analysis, code generation, and the preprocessor all use
//! the same counter and the same limit so deeply nested input produces a
//! normal diagnostic in every pass instead of overflowing the native stack or
//! trapping the WebAssembly-hosted build.
//!
//! The limit is measured against the WebAssembly-hosted build under Node/V8,
//! whose call stack is much smaller than a native thread with an explicitly
//! large stack. The wasm-hosted compiler is linked with an 8 MiB linear-memory
//! stack (see `.cargo/config.toml`), while the native CLI runs the compiler on
//! a 64 MiB thread. With those stacks, nested calls, ternaries, namespaces and
//! blocks were observed to compile at depth 1000 and to be rejected cleanly
//! below depth 5000 (the V8 call stack is exceeded around 4000 frames of this
//! compiler). 2000 leaves a factor-of-two margin below the measured limit
//! while still accepting the 1000-deep programs that compiled before.

use crate::diagnostics::{CompileError, ErrorKind};
use std::cell::Cell;
use std::rc::Rc;

/// Maximum AST/expression depth accepted by any recursive compiler pass.
pub const MAX_AST_DEPTH: usize = 2000;

/// Diagnostic text emitted when the shared depth limit is exceeded.
pub const DEPTH_DIAGNOSTIC: &str = "Nesting depth exceeds the supported limit";

/// A cloneable, shared depth counter.
#[derive(Clone, Debug)]
pub struct DepthCounter {
    depth: Rc<Cell<usize>>,
}

impl DepthCounter {
    pub fn new() -> Self {
        Self {
            depth: Rc::new(Cell::new(0)),
        }
    }

    /// Reserves one depth level. The returned guard releases it when dropped.
    pub fn enter(&self, line: usize, col: usize) -> Result<DepthGuard, CompileError> {
        let depth = self.depth.get() + 1;
        if depth > MAX_AST_DEPTH {
            return Err(CompileError::new(DEPTH_DIAGNOSTIC, line, col).with_kind(ErrorKind::Syntax));
        }
        self.depth.set(depth);
        Ok(DepthGuard {
            depth: Rc::clone(&self.depth),
        })
    }
}

impl Default for DepthCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Releases one depth level when dropped, including on an early error return.
pub struct DepthGuard {
    depth: Rc<Cell<usize>>,
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        let depth = self.depth.get();
        self.depth.set(depth.saturating_sub(1));
    }
}
