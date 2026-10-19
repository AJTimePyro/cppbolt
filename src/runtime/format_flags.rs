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
    code.push(OP_I32_CONST);
    write_i32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 3);
    code.push(OP_I32_CONST);
    write_i32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 4);
    code.push(OP_I32_CONST);
    write_i32(code, -1);
    code.push(OP_LOCAL_SET);
    write_u32(code, 5);
    code.push(OP_I32_CONST);
    write_i32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 6);

    // Parse the supported flags: '0', '-', '+', and ' '.
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

    // '0': zero_pad (bit 0 = 1)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 48); // '0'
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
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
    write_u32(code, 1); // loop
    code.push(OP_END);

    // '-': left_align (bit 1 = 2)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 45); // '-'
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_I32_CONST);
    write_i32(code, 2);
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

    // '+': show_sign (bit 2 = 4)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 43); // '+'
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_I32_CONST);
    write_i32(code, 4);
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

    // ' ': space_sign (bit 3 = 8)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 32); // ' '
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_LOCAL_GET);
    write_u32(code, 3);
    code.push(OP_I32_CONST);
    write_i32(code, 8);
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

    // '#': alternate form (bit 6 = 64)
    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 35); // '#'
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

    // No more flags
    code.push(OP_BR);
    write_u32(code, 1); // break flags loop

    code.push(OP_END); // end loop
    code.push(OP_END); // end block
}
