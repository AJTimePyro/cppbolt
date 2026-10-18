//! The linear-memory ABI shared by code generation and runtime emission.

pub const PAGE_SIZE: u32 = 64 * 1024;
pub const NULL_RESERVED_BYTES: u32 = PAGE_SIZE;
pub const STACK_BYTES: u32 = 8 * 1024 * 1024;
pub const DATA_BASE: u32 = NULL_RESERVED_BYTES + STACK_BYTES;
pub const DEFAULT_MEMORY_PAGES: u32 = 256;
pub const VARARGS_OFFSET: u32 = 128;
pub const DEFAULT_FRAME_BYTES: i32 = 1024;
pub const LOCAL_FRAME_GUARD: i32 = NULL_RESERVED_BYTES as i32;

#[cfg(test)]
pub const fn align_up(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

pub const fn pages_for(bytes: u32) -> u32 {
    bytes.div_ceil(PAGE_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_constants_are_consistent() {
        assert_eq!(DATA_BASE, 8_454_144);
        assert_eq!(pages_for(DATA_BASE), 129);
        assert_eq!(align_up(DATA_BASE + 1, 16), 8_454_160);
    }
}
