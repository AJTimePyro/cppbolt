//! The linear-memory ABI shared by code generation and runtime emission.

pub const PAGE_SIZE: u32 = 64 * 1024;
pub const NULL_RESERVED_BYTES: u32 = PAGE_SIZE;
pub const STACK_BYTES: u32 = 8 * 1024 * 1024;
pub const DATA_BASE: u32 = NULL_RESERVED_BYTES + STACK_BYTES;
pub const DEFAULT_MEMORY_PAGES: u32 = 256;
pub const LOCAL_FRAME_GUARD: i32 = NULL_RESERVED_BYTES as i32;

pub const fn pages_for(bytes: u32) -> u32 {
    bytes.div_ceil(PAGE_SIZE)
}
