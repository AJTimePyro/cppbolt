use super::opcodes::*;
use crate::wasm::*;

/// Emits the shared variadic printf implementation.
///
/// The generated function reads its arguments from a caller-built argument
/// block; only conversion selection and formatting run here.
pub fn emit_printf(func_print_char: u32, func_print_int: u32, func_print_double: u32) -> FuncBody {
    let mut code = Vec::new();
    // Local 10 counts characters so printf returns the C standard result.
    code.push(OP_I32_CONST);
    write_i32(&mut code, 0);
    code.push(OP_LOCAL_SET);
    write_u32(&mut code, super::PRINTF_CHAR_COUNT_LOCAL);
    super::format_flags::emit_format_loop_start(&mut code, func_print_char);
    super::format_flags::emit_flag_parse(&mut code);
    super::format_number_parse::emit_width_parse(&mut code);
    super::format_number_parse::emit_precision_parse(&mut code);
    super::format_number_parse::emit_length_parse(&mut code);
    super::format_integer::emit_integer_conversion(&mut code, func_print_int);
    super::format_float::emit_float_conversion(&mut code, func_print_double);
    super::format_char_string::emit_char_string_conversion(&mut code, func_print_char);
    emit_printf_epilogue(&mut code);
    (
        vec![(6, TYPE_I32), (1, TYPE_I64), (1, TYPE_F64), (1, TYPE_I32)],
        code,
    )
}

fn emit_printf_epilogue(code: &mut Vec<u8>) {
    // If unrecognized specifier, just continue
    code.push(OP_BR);
    write_u32(code, 0);

    code.push(OP_END); // end loop
    code.push(OP_END); // end block

    code.push(OP_LOCAL_GET);
    write_u32(code, super::PRINTF_CHAR_COUNT_LOCAL);
    code.push(OP_RETURN);
    code.push(OP_END);
}
