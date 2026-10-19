use super::opcodes::*;
use crate::wasm::*;

/// Emits the floating-point conversions `%f`, `%e`/`%E`, and `%g`/`%G`.
pub(super) fn emit_float_conversion(code: &mut Vec<u8>, func_print_double: u32) {
    // 'f' (102)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 102);
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_F64_LOAD);
    write_u32(code, 3);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 9);
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_I32_CONST);
    write_i32(code, 8);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 1);
    code.push(OP_LOCAL_GET);
    write_u32(code, 9);
    code.push(OP_LOCAL_GET);
    write_u32(code, 4); // width
    code.push(OP_LOCAL_GET);
    write_u32(code, 5); // precision
    code.push(OP_I32_CONST);
    write_i32(code, 1); // mode 1 = fixed (%f)
    code.push(OP_LOCAL_GET);
    write_u32(code, 3); // flags
    code.push(OP_CALL);
    write_u32(code, func_print_double);
    super::emit_add_to_char_count(code, super::PRINTF_CHAR_COUNT_LOCAL);
    code.push(OP_BR);
    write_u32(code, 1);
    code.push(OP_END);

    // 'e' (101) or 'E' (69)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 101);
    code.push(OP_I32_EQ);
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 69);
    code.push(OP_I32_EQ);
    code.push(OP_I32_OR);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_F64_LOAD);
    write_u32(code, 3);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 9);
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_I32_CONST);
    write_i32(code, 8);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 1);
    code.push(OP_LOCAL_GET);
    write_u32(code, 9);
    code.push(OP_LOCAL_GET);
    write_u32(code, 4); // width
    code.push(OP_LOCAL_GET);
    write_u32(code, 5); // precision
    code.push(OP_I32_CONST);
    write_i32(code, 2); // mode 2 = scientific (%e)
    code.push(OP_LOCAL_GET);
    write_u32(code, 3); // flags
    code.push(OP_CALL);
    write_u32(code, func_print_double);
    super::emit_add_to_char_count(code, super::PRINTF_CHAR_COUNT_LOCAL);
    code.push(OP_BR);
    write_u32(code, 1);
    code.push(OP_END);

    // 'g' (103) or 'G' (71)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 103);
    code.push(OP_I32_EQ);
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 71);
    code.push(OP_I32_EQ);
    code.push(OP_I32_OR);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_F64_LOAD);
    write_u32(code, 3);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 9);
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_I32_CONST);
    write_i32(code, 8);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 1);
    code.push(OP_LOCAL_GET);
    write_u32(code, 9);
    code.push(OP_LOCAL_GET);
    write_u32(code, 4); // width
    code.push(OP_LOCAL_GET);
    write_u32(code, 5); // precision
    code.push(OP_I32_CONST);
    write_i32(code, 0); // mode 0 = default (%g)
    code.push(OP_LOCAL_GET);
    write_u32(code, 3); // flags
    code.push(OP_CALL);
    write_u32(code, func_print_double);
    super::emit_add_to_char_count(code, super::PRINTF_CHAR_COUNT_LOCAL);
    code.push(OP_BR);
    write_u32(code, 1);
    code.push(OP_END);
}
