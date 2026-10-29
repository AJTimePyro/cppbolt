use super::*;

impl<'a> Codegen<'a> {
    /// Lowers `return`, including out-of-band struct results and every
    /// destructor that must run before the epilogue.
    pub(super) fn compile_return(
        &mut self,
        stmt: &Stmt,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let StmtKind::Return(expr_opt) = &stmt.kind else {
            return Err(self.internal_error("unexpected statement kind"));
        };

        if let Some(expr) = expr_opt {
            self.compile_return_value(expr, ctx, code)?;
        }

        // Epilogue: Run destructors across all active scopes in reverse order
        for scope in ctx.scope_objects.iter().rev() {
            self.emit_dtor_for_scope(scope, ctx, code);
        }

        if ctx.is_main {
            // A return from main still destroys namespace-scope objects.
            self.emit_global_destructors(code);
        }

        // If destructor of derived class, call base class destructors in
        // reverse construction order.
        self.emit_base_destructors(ctx, code);

        // Epilogue: restore stack pointer
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        ctx.emit_frame_size(code);
        code.push(OP_I32_ADD);
        code.push(OP_GLOBAL_SET);
        write_u32(code, 0);

        if expr_opt.is_some() {
            self.emit_return_value_load(ctx, code);
        }

        code.push(OP_RETURN);

        Ok(())
    }

    fn compile_return_value(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let synthetic;
        let expr = match expr {
            Expr::InitializerList { elements, pos } => {
                synthetic = Expr::TypedInitList {
                    ty: ctx.return_type.strip_reference().strip_const().clone(),
                    elements: elements.clone(),
                    pos: *pos,
                };
                &synthetic
            }
            _ => expr,
        };
        let ty = self.get_expr_type(expr, ctx);
        if ctx.is_reference_return {
            self.compile_lvalue_addr(expr, ctx, code)?;
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.ret_temp_i32);
            self.emit_clean_temporaries(ctx, code);
            return Ok(());
        }
        if let Type::Struct(sname) = ty.strip_const() {
            return self.compile_return_struct(expr, sname, ctx, code);
        }
        self.compile_expr(expr, ctx, code)?;
        self.emit_conversion(&ty, &ctx.return_type, code);
        if let Some(local) = return_local(&ctx.return_type, ctx) {
            code.push(OP_LOCAL_SET);
            write_u32(code, local);
        }
        self.emit_clean_temporaries(ctx, code);
        Ok(())
    }

    /// Structs are returned out of band: the result buffer is heap-allocated
    /// and its address is the Wasm return value.
    fn compile_return_struct(
        &mut self,
        expr: &Expr,
        sname: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let size = self
            .sema
            .struct_sizes
            .get(sname)
            .copied()
            .ok_or_else(|| self.internal_error("missing struct layout"))?;
        if self.is_struct_lvalue(expr, ctx) {
            code.push(OP_I32_CONST);
            write_i32(code, size as i32);
            code.push(OP_CALL);
            write_u32(code, self.func_indices["malloc"]);
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.ret_temp_i32);

            self.compile_lvalue_addr(expr, ctx, code)?;
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1);

            let expr_ty = self.get_expr_type(expr, ctx);
            let is_const = expr_ty.strip_reference().is_const();
            if let Some(ctor_idx) = self.find_copy_constructor(sname, is_const) {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.ret_temp_i32);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_CALL);
                write_u32(code, ctor_idx);
            } else {
                super::raii::emit_struct_byte_copy(
                    size,
                    0,
                    |code| {
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.ret_temp_i32);
                    },
                    |code| {
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.scratch1);
                    },
                    code,
                );
            }
            self.emit_clean_temporaries(ctx, code);
            return Ok(());
        }
        self.compile_expr(expr, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.ret_temp_i32);
        if let Some(pos) = ctx.temp_objects.iter().rposition(|(s, _, _)| s == sname) {
            ctx.temp_objects.remove(pos);
        }
        self.emit_clean_temporaries(ctx, code);
        Ok(())
    }

    fn emit_return_value_load(&mut self, ctx: &mut FunctionContext, code: &mut Vec<u8>) {
        if ctx.is_reference_return {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.ret_temp_i32);
        } else if let Some(local) = return_local(&ctx.return_type, ctx) {
            code.push(OP_LOCAL_GET);
            write_u32(code, local);
        }
    }
}

/// The frame local that holds a return value of `ty`.
///
/// Reference returns use `ret_temp_i32` directly and are handled by the caller.
pub(super) fn return_local(ty: &Type, ctx: &FunctionContext) -> Option<u32> {
    match ty.strip_const() {
        Type::Void => None,
        Type::LongLong | Type::UnsignedLongLong => Some(ctx.ret_temp_i64),
        Type::Double => Some(ctx.ret_temp_f64),
        Type::Float => Some(ctx.scratch_f32),
        _ => Some(ctx.ret_temp_i32),
    }
}
