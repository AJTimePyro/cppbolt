use super::opcodes::*;

pub fn emit_pure_virtual_trap() -> (Vec<(u32, u8)>, Vec<u8>) {
    (vec![], vec![OP_UNREACHABLE, OP_END])
}
