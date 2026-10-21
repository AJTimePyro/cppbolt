use super::opcodes::*;
use crate::wasm::*;

/// Emits the outer format-string loop, including literal text and `%%`.
pub(super) fn emit_format_loop_start(code: &mut Vec<u8>, func_print_char: u32) {
    // Walk the format string one byte at a time until the NUL terminator.
    code.push(OP_BLOCK);
    code.push(0x40);
    code.push(OP_LOOP);
    code.push(0x40);

    // ch = mem[fmt]
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_LOAD8_U);
    write_u32(code, 0);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 2);

    // if ch == 0: break
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_EQZ);
    code.push(OP_BR_IF);
    write_u32(code, 1);

    // if ch != '%': print_char(ch); fmt++; continue
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 37); // '%'
    code.push(OP_I32_NE);
    code.push(OP_IF);
    code.push(0x40);
    super::emit_increment_char_count(code, super::PRINTF_CHAR_COUNT_LOCAL);
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_CALL);
    write_u32(code, func_print_char);
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
    code.push(OP_BR);
    write_u32(code, 1); // continue outer loop
    code.push(OP_END); // end if != '%'

    emit_format_percent_escape(code, func_print_char);
}

/// Consumes `%` and handles the `%%` escape before parsing flags.
fn emit_format_percent_escape(code: &mut Vec<u8>, func_print_char: u32) {
    // Consume '%' and handle the "%%" escape before parsing flags.
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);

    // Check for "%%"
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
    write_i32(code, 37); // '%'
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_I32_CONST);
    write_i32(code, 37);
    code.push(OP_CALL);
    write_u32(code, func_print_char);
    super::emit_increment_char_count(code, super::PRINTF_CHAR_COUNT_LOCAL);
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
    code.push(OP_BR);
    write_u32(code, 1); // continue outer loop
    code.push(OP_END);
}

/// Resets per-conversion state and parses the supported flag characters.
pub(super) fn emit_flag_parse(code: &mut Vec<u8>) {
    // Reset per-conversion state: flags=0, width=0, precision=unspecified
    // (-1), and is_long_long=0.
    for (local, value) in [(3, 0), (4, 0), (5, -1), (6, 0)] {
        code.push(OP_I32_CONST);
        write_i32(code, value);
        code.push(OP_LOCAL_SET);
        write_u32(code, local);
    }

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

    emit_flag(code, 48, 1); // '0': zero_pad
    emit_flag(code, 45, 2); // '-': left_align
    emit_flag(code, 43, 4); // '+': show_sign
    emit_flag(code, 32, 8); // ' ': space_sign
    emit_hash_flag(code);

    code.push(OP_BR);
    write_u32(code, 1); // break flags loop
    code.push(OP_END); // end loop
    code.push(OP_END); // end block
}

fn emit_flag(code: &mut Vec<u8>, character: i32, bit: i32) {
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, character);
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_I32_CONST);
    write_i32(code, bit);
    code.push(OP_I32_OR);
    code.push(OP_LOCAL_SET);
    write_u32(code, 3);
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
}

fn emit_hash_flag(code: &mut Vec<u8>) {
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 35);
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_I32_CONST);
    write_i32(code, 64);
    code.push(OP_I32_OR);
    code.push(OP_LOCAL_SET);
    write_u32(code, 3);
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
}
