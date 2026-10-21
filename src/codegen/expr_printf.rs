use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_printf_call(
        &mut self,
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        // Variadic arguments are laid out by value in a compact frame
        // block. Floats are promoted to double, matching the runtime's
        // 4/8-byte reader.
        let mut vararg_bytes = 0u32;
        for a in args.iter().skip(1) {
            let aty = self.get_expr_type(a, ctx);
            let size = match aty.strip_const().strip_reference() {
                Type::Float | Type::Double | Type::LongLong | Type::UnsignedLongLong => 8,
                _ => 4,
            };
            vararg_bytes = vararg_bytes
                .checked_add(size)
                .ok_or_else(|| self.internal_error("printf arguments are too large"))?;
        }
        let varargs_base = ctx.alloc_frame(vararg_bytes, 8)?;
        let mut arg_offset = varargs_base;

        for a in args.iter().skip(1) {
            let aty = self.get_expr_type(a, ctx);
            let clean_aty = aty.strip_const().strip_reference();
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            self.compile_expr(a, ctx, code)?;

            if matches!(clean_aty, Type::Float) {
                code.push(OP_F64_PROMOTE_F32);
                code.push(OP_F64_STORE);
                write_u32(code, 3);
                write_u32(code, arg_offset);
                arg_offset += 8;
            } else if matches!(clean_aty, Type::Double) {
                code.push(OP_F64_STORE);
                write_u32(code, 3);
                write_u32(code, arg_offset);
                arg_offset += 8;
            } else if matches!(clean_aty, Type::LongLong | Type::UnsignedLongLong) {
                code.push(OP_I64_STORE);
                write_u32(code, 3);
                write_u32(code, arg_offset);
                arg_offset += 8;
            } else {
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, arg_offset);
                arg_offset += 4;
            }
        }

        self.compile_expr(&args[0], ctx, code)?;
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_CONST);
        write_i32(code, varargs_base as i32);
        code.push(OP_I32_ADD);

        code.push(OP_CALL);
        write_u32(code, self.func_printf);
        Ok(())
    }
}
