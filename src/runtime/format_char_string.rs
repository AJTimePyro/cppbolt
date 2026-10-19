use super::opcodes::*;
use crate::wasm::*;

/// Emits the `%c` and `%s` conversions.
pub(super) fn emit_char_string_conversion(code: &mut Vec<u8>, func_print_char: u32) {
    // 'c' (99)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 99);
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_I32_LOAD);
    write_u32(code, 2);
    write_u32(code, 0);
    code.push(OP_CALL);
    write_u32(code, func_print_char);
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_I32_CONST);
    write_i32(code, 4);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 1);
    code.push(OP_BR);
    write_u32(code, 1);
    code.push(OP_END);

    // 's' (115)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 115);
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_I32_LOAD);
    write_u32(code, 2);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 7); // str_ptr
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_I32_CONST);
    write_i32(code, 4);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 1);

    // Print until the NUL terminator.
    code.push(OP_BLOCK);
    code.push(0x40);
    code.push(OP_LOOP);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 7);
    code.push(OP_I32_LOAD8_U);
    write_u32(code, 0);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 2);

    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_EQZ);
    code.push(OP_BR_IF);
    write_u32(code, 1);

    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_CALL);
    write_u32(code, func_print_char);

    code.push(OP_LOCAL_GET);
    write_u32(code, 7);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 7);
    code.push(OP_BR);
    write_u32(code, 0);

    code.push(OP_END);
    code.push(OP_END);

    code.push(OP_BR);
    write_u32(code, 1);
    code.push(OP_END); // end 's'
}
