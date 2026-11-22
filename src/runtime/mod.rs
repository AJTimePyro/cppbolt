//! Emitted runtime helpers: printf formatting, the allocator, and traps.
//!
//! See the mdBook guide at `docs/book/internals/runtime.md`.

pub mod allocator;
pub mod format;
pub mod memory_layout;
pub mod opcodes;
pub mod trap;

pub use allocator::{emit_free, emit_malloc};
pub use format::emit_printf;
pub use opcodes::*;
pub use trap::emit_pure_virtual_trap;
