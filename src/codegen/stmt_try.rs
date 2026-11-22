//! Statement code generation for exception constructs (`throw` and `try`/`catch`).

use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_throw_stmt(
        &mut self,
        expr_opt: &Option<Expr>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Some(expr) = expr_opt {
            self.compile_expr(expr, ctx, code)?;
            code.push(OP_THROW);
            write_u32(code, 0);
        } else if ctx.catch_depth == 0 {
            code.push(OP_UNREACHABLE);
        } else if ctx.in_catch_all {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.exnref_local);
            code.push(OP_THROW_REF);
        } else {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.last_exn_val_local);
            code.push(OP_THROW);
            write_u32(code, 0);
        }
        Ok(())
    }

    pub(super) fn compile_try_stmt(
        &mut self,
        try_body: &Stmt,
        catch_param: &Option<(Type, String)>,
        catch_body: &Stmt,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let start_clean_idx = ctx.cleanup_entries.len();
        code.push(OP_BLOCK);
        code.push(0x40);

        let is_catch_all = catch_param.is_none();
        if is_catch_all {
            code.push(OP_BLOCK);
            code.push(TYPE_EXNREF);
            code.push(OP_TRY_TABLE);
            code.push(0x40);
            code.push(0x01);
            code.push(0x03); // catch_all_ref
            write_u32(code, 0);
        } else {
            code.push(OP_BLOCK);
            code.push(TYPE_I32);
            code.push(OP_TRY_TABLE);
            code.push(0x40);
            code.push(0x01);
            code.push(0x00); // catch tag 0
            write_u32(code, 0);
            write_u32(code, 0);
        }

        self.compile_stmt(try_body, ctx, code)?;

        code.push(OP_BR);
        write_u32(code, 2);

        code.push(OP_END);
        code.push(OP_UNREACHABLE);
        code.push(OP_END);

        let end_clean_idx = ctx.cleanup_entries.len();
        self.emit_catch_handler(
            catch_param,
            catch_body,
            start_clean_idx..end_clean_idx,
            ctx,
            code,
        )?;

        code.push(OP_END);
        Ok(())
    }

    fn emit_catch_handler(
        &mut self,
        catch_param: &Option<(Type, String)>,
        catch_body: &Stmt,
        clean_range: std::ops::Range<usize>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let is_catch_all = catch_param.is_none();
        if is_catch_all {
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.exnref_local);
        } else {
            code.push(OP_LOCAL_TEE);
            write_u32(code, ctx.last_exn_val_local);
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1);
        }

        // Clean up only objects that were constructed inside the try body
        self.emit_cleanup_range(clean_range.start, clean_range.end, ctx, code);

        ctx.enter_scope();
        let old_in_catch_all = ctx.in_catch_all;
        ctx.in_catch_all = is_catch_all;
        ctx.catch_depth += 1;

        if let Some((ty, name)) = catch_param
            && !name.is_empty()
        {
            let size = u32::try_from(ty.size_of(&self.sema.struct_sizes))
                .map_err(|_| crate::internal_error!(catch_body.line, catch_body.col, "local too large"))?;
            let align = u32::try_from(ty.align_of(&self.sema.struct_aligns))
                .unwrap_or(4)
                .max(4);
            let offset = ctx.alloc_frame(size, align)?;
            ctx.bind_local(name.clone(), offset, ty.clone());

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_I32_STORE);
            write_u32(code, 2);
            write_u32(code, offset);
        }

        self.compile_stmt(catch_body, ctx, code)?;
        ctx.catch_depth -= 1;
        ctx.in_catch_all = old_in_catch_all;
        ctx.exit_scope();
        Ok(())
    }
}
