//! Emitted runtime helpers: printf formatting, the allocator, and traps.
//!
//! See the mdBook guide at `docs/book/internals/runtime.md`.

use crate::wasm::*;

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

/// Local index that holds the number of characters written by `printf`.
pub(super) const PRINTF_CHAR_COUNT_LOCAL: u32 = 10;

/// Adds a value already on the Wasm stack to the printf character counter.
pub(super) fn emit_add_to_char_count(code: &mut Vec<u8>, counter_local: u32) {
    code.push(OP_LOCAL_GET);
    write_u32(code, counter_local);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, counter_local);
}

/// Adds one character to the printf character counter.
pub(super) fn emit_increment_char_count(code: &mut Vec<u8>, counter_local: u32) {
    code.push(OP_LOCAL_GET);
    write_u32(code, counter_local);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, counter_local);
}
