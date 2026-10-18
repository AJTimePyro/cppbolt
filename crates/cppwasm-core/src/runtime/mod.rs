//! Emitted runtime helpers: printf formatting, the allocator, and traps.

pub mod allocator;
pub mod format;
mod format_char_string;
mod format_flags;
mod format_float;
mod format_integer;
mod format_number_parse;
pub mod memory_layout;
pub mod opcodes;
pub mod trap;

pub use allocator::{emit_free, emit_malloc};
pub use format::emit_printf;
pub use opcodes::*;
pub use trap::emit_pure_virtual_trap;
