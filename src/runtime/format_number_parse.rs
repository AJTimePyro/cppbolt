use super::opcodes::*;
use crate::wasm::*;

/// Parses an optional decimal field width.
pub(super) fn emit_width_parse(code: &mut Vec<u8>) {
    code.push(OP_BLOCK);
    code.push(0x40);
    code.push(OP_LOOP);
    code.push(0x40);

    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_LOAD8_U);
    write_u32(code, 0);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 2);

    // if ch >= '0' && ch <= '9'
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 48); // '0'
    code.push(OP_I32_GE_U);
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 57); // '9'
    code.push(OP_I32_LE_U);
    code.push(OP_I32_AND);
    code.push(OP_IF);
    code.push(0x40);
    // width = width * 10 + (ch - '0')
    code.push(OP_LOCAL_GET);
    write_u32(code, 4);
    code.push(OP_I32_CONST);
    write_i32(code, 10);
    code.push(OP_I32_MUL);
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 48);
    code.push(OP_I32_SUB);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 4);
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
    code.push(OP_BR);
    write_u32(code, 1); // continue width loop
    code.push(OP_END);

    code.push(OP_BR);
    write_u32(code, 1); // break width loop
    code.push(OP_END);
    code.push(OP_END);
}

/// Parses `.precision`, including `*` which consumes an integer argument.
pub(super) fn emit_precision_parse(code: &mut Vec<u8>) {
    // A '.' introduces an explicit precision; without one it stays -1.
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_LOAD8_U);
    write_u32(code, 0);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 2);

    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 46); // '.'
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
    // A bare '.' means zero digits of precision.
    code.push(OP_I32_CONST);
    write_i32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 5);

    // Check if next char is '*' (42)
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_LOAD8_U);
    write_u32(code, 0);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 2);

    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 42); // '*'
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    emit_star_precision(code);
    code.push(OP_ELSE);

    emit_decimal_precision(code);
    code.push(OP_END); // end if/else for '*'

    code.push(OP_END); // end if '.'
}

/// Parses an `h`, `hh`, `l`, `ll`, or `z` length modifier.
pub(super) fn emit_length_parse(code: &mut Vec<u8>) {
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_LOAD8_U);
    write_u32(code, 0);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 2);

    emit_z_length(code);
    emit_l_length(code);
    emit_h_length(code);

    // Read the conversion specifier and consume it.
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_LOAD8_U);
    write_u32(code, 0);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 2);

    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
}

fn emit_star_precision(code: &mut Vec<u8>) {
    // precision = mem[args_ptr]
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_I32_LOAD);
    write_u32(code, 2);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 5);
    // args_ptr += 4
    code.push(OP_LOCAL_GET);
    write_u32(code, 1);
    code.push(OP_I32_CONST);
    write_i32(code, 4);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 1);
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
}

fn emit_decimal_precision(code: &mut Vec<u8>) {
    // Accumulate decimal precision digits.
    code.push(OP_BLOCK);
    code.push(0x40);
    code.push(OP_LOOP);
    code.push(0x40);

    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_LOAD8_U);
    write_u32(code, 0);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 2);

    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 48);
    code.push(OP_I32_GE_U);
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 57);
    code.push(OP_I32_LE_U);
    code.push(OP_I32_AND);
    code.push(OP_IF);
    code.push(0x40);
    // precision = precision * 10 + (ch - '0')
    code.push(OP_LOCAL_GET);
    write_u32(code, 5);
    code.push(OP_I32_CONST);
    write_i32(code, 10);
    code.push(OP_I32_MUL);
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 48);
    code.push(OP_I32_SUB);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 5);
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
    code.push(OP_BR);
    write_u32(code, 1);
    code.push(OP_END);

    code.push(OP_BR);
    write_u32(code, 1);
    code.push(OP_END);
    code.push(OP_END);
}

fn emit_z_length(code: &mut Vec<u8>) {
    // 'z' (122)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 122);
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
    code.push(OP_END);
}

fn emit_l_length(code: &mut Vec<u8>) {
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 108); // 'l'
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);

    // `ll` widens integer arguments to 64 bits.
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_LOAD8_U);
    write_u32(code, 0);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 108); // 'l'
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_LOCAL_SET);
    write_u32(code, 6); // is_long_long = 1
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
    code.push(OP_END);

    code.push(OP_END); // end if 'l'
}

fn emit_h_length(code: &mut Vec<u8>) {
    // 'h' (104): `hd`/`hu` truncate to 16 bits, `hhd`/`hhu` to 8 bits.
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 104); // 'h'
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_I32_CONST);
    write_i32(code, 2);
    code.push(OP_LOCAL_SET);
    write_u32(code, 6);
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_LOAD8_U);
    write_u32(code, 0);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 104); // 'h'
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_I32_CONST);
    write_i32(code, 3);
    code.push(OP_LOCAL_SET);
    write_u32(code, 6);
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
    code.push(OP_END);
    code.push(OP_END);
}
