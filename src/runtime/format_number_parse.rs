use super::opcodes::*;
use crate::wasm::*;

pub(super) fn emit_width_parse(code: &mut Vec<u8>) {
    // Loop to parse width
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
    // fmt++
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

pub(super) fn emit_precision_parse(code: &mut Vec<u8>) {
    // Check for precision: '.'
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
    // fmt++
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
    // precision = 0
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
    // fmt++
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
    code.push(OP_ELSE);

    // Loop to parse precision digits
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
    // fmt++
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
    code.push(OP_END); // end else for '*'

    code.push(OP_END); // end if '.'
}

pub(super) fn emit_length_parse(code: &mut Vec<u8>) {
    // Check for length modifier: 'z', 'l', or "ll"
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_LOAD8_U);
    write_u32(code, 0);
    write_u32(code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(code, 2);

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

    code.push(OP_LOCAL_GET);
    write_u32(code, 2);
    code.push(OP_I32_CONST);
    write_i32(code, 108); // 'l'
    code.push(OP_I32_EQ);
    code.push(OP_IF);
    code.push(0x40);
    // fmt++
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);

    // check if second 'l'
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
    // fmt++
    code.push(OP_LOCAL_GET);
    write_u32(code, 0);
    code.push(OP_I32_CONST);
    write_i32(code, 1);
    code.push(OP_I32_ADD);
    code.push(OP_LOCAL_SET);
    write_u32(code, 0);
    code.push(OP_END);

    code.push(OP_END); // end if 'l'

    // Specifier: ch = mem[fmt]; fmt++
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
