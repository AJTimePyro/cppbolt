use super::opcodes::*;
use crate::wasm::*;

/// Emits the integer conversions `%d`, `%i`, `%u`, `%o`, `%x`, and `%X`.
pub(super) fn emit_integer_conversion(code: &mut Vec<u8>, func_print_int: u32) {
    emit_signed_integer(code, func_print_int);
    emit_unsigned_integer(code, func_print_int);
    emit_hex_integer(code, func_print_int);
    emit_octal_integer(code, func_print_int);
}

fn emit_signed_integer(code: &mut Vec<u8>, func_print_int: u32) {
    // 'd' (100) or 'i' (105)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 100);
    code.push(OP_I32_EQ);
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 105);
    code.push(OP_I32_EQ);
    code.push(OP_I32_OR);
    code.push(OP_IF);
    code.push(0x40);
    emit_load_integer_argument(code, true);
    code.push(OP_LOCAL_SET);
    write_u32(code, 8);
    // call print_int(val_i64, 10, width, precision, flags)
    code.push(OP_LOCAL_GET);
    write_u32(code, 8);
    code.push(OP_I32_CONST);
    write_i32(code, 10);
    code.push(OP_LOCAL_GET);
    write_u32(code, 4);
    code.push(OP_LOCAL_GET);
    write_u32(code, 5);
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_CALL);
    write_u32(code, func_print_int);
    super::emit_add_to_char_count(code, super::PRINTF_CHAR_COUNT_LOCAL);
    code.push(OP_BR);
    write_u32(code, 1); // next in outer loop
    code.push(OP_END);
}

fn emit_unsigned_integer(code: &mut Vec<u8>, func_print_int: u32) {
    // 'u' (117)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 117);
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    // flags |= 32 (is_unsigned)
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_I32_CONST);
    write_i32(code, 32);
    code.push(OP_I32_OR);
    code.push(OP_LOCAL_SET);
    write_u32(code, 3);

    emit_load_integer_argument(code, false);
    code.push(OP_LOCAL_SET);
    write_u32(code, 8);
    code.push(OP_LOCAL_GET);
    write_u32(code, 8);
    code.push(OP_I32_CONST);
    write_i32(code, 10);
    code.push(OP_LOCAL_GET);
    write_u32(code, 4);
    code.push(OP_LOCAL_GET);
    write_u32(code, 5);
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_CALL);
    write_u32(code, func_print_int);
    super::emit_add_to_char_count(code, super::PRINTF_CHAR_COUNT_LOCAL);
    code.push(OP_BR);
    write_u32(code, 1);
    code.push(OP_END);
}

fn emit_hex_integer(code: &mut Vec<u8>, func_print_int: u32) {
    // 'x' (120) or 'X' (88)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 120);
    code.push(OP_I32_EQ);
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 88);
    code.push(OP_I32_EQ);
    code.push(OP_I32_OR);
    code.push(OP_IF);
    code.push(0x40);
    // if 'X', flags |= 16
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 88);
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_I32_CONST);
    write_i32(code, 16);
    code.push(OP_I32_OR);
    code.push(OP_LOCAL_SET);
    write_u32(code, 3);
    code.push(OP_END);
    // flags |= 32 (is_unsigned)
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_I32_CONST);
    write_i32(code, 32);
    code.push(OP_I32_OR);
    code.push(OP_LOCAL_SET);
    write_u32(code, 3);

    emit_load_integer_argument(code, false);
    code.push(OP_LOCAL_SET);
    write_u32(code, 8);
    code.push(OP_LOCAL_GET);
    write_u32(code, 8);
    code.push(OP_I32_CONST);
    write_i32(code, 16); // radix 16
    code.push(OP_LOCAL_GET);
    write_u32(code, 4);
    code.push(OP_LOCAL_GET);
    write_u32(code, 5);
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_CALL);
    write_u32(code, func_print_int);
    super::emit_add_to_char_count(code, super::PRINTF_CHAR_COUNT_LOCAL);
    code.push(OP_BR);
    write_u32(code, 1);
    code.push(OP_END);
}

fn emit_octal_integer(code: &mut Vec<u8>, func_print_int: u32) {
    // 'o' (111): octal, always unsigned.
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 111);
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    // flags |= 32 (is_unsigned)
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_I32_CONST);
    write_i32(code, 32);
    code.push(OP_I32_OR);
    code.push(OP_LOCAL_SET);
    write_u32(code, 3);

    emit_load_integer_argument(code, false);
    code.push(OP_LOCAL_SET);
    write_u32(code, 8);
    code.push(OP_LOCAL_GET);
    write_u32(code, 8);
    code.push(OP_I32_CONST);
    write_i32(code, 8); // radix 8
    code.push(OP_LOCAL_GET);
    write_u32(code, 4);
    code.push(OP_LOCAL_GET);
    write_u32(code, 5);
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_CALL);
    write_u32(code, func_print_int);
    super::emit_add_to_char_count(code, super::PRINTF_CHAR_COUNT_LOCAL);
    code.push(OP_BR);
    write_u32(code, 1);
    code.push(OP_END);
}

/// Loads one integer vararg, honoring `ll`, `h`, and `hh` length modifiers.
///
/// Varargs wider than `int` are passed in the argument block as 8-byte
/// values. Narrow values are passed as 4-byte `int` values and truncated here
/// according to the C default-argument-promotion rules.
fn emit_load_integer_argument(code: &mut Vec<u8>, signed: bool) {
    code.push(OP_LOCAL_GET);
    write_u32(code, 6);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x7E); // result i64
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_I64_LOAD);
    write_u32(code, 2);
    write_u32(code, 0);
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_I32_CONST);
    write_i32(code, 8);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 1);
    code.push(OP_ELSE);
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_I32_LOAD);
    write_u32(code, 2);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 2);
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_I32_CONST);
    write_i32(code, 4);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 1);
    emit_load_narrow_integer_argument(code, signed);
}

/// Loads a narrow vararg, applying the `h` and `hh` length modifiers.
fn emit_load_narrow_integer_argument(code: &mut Vec<u8>, signed: bool) {
    // `h` truncates to 16 bits; `hh` truncates to 8 bits.
    code.push(OP_LOCAL_GET);
    write_u32(code, 6);
    code.push(OP_I32_CONST);
    write_i32(code, 2);
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x7F); // result i32
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 0xFFFF);
    code.push(OP_I32_AND);
    if signed {
        code.push(OP_I32_EXTEND16_S);
    }
    code.push(OP_ELSE);
    code.push(OP_LOCAL_GET);
    write_u32(code, 6);
    code.push(OP_I32_CONST);
    write_i32(code, 3);
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x7F); // result i32
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 0xFF);
    code.push(OP_I32_AND);
    if signed {
        code.push(OP_I32_EXTEND8_S);
    }
    code.push(OP_ELSE);
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_END);
    code.push(OP_END);

    if signed {
        code.push(OP_I64_EXTEND_I32_S);
    } else {
        code.push(OP_I64_EXTEND_I32_U);
    }
    code.push(OP_END);
}
