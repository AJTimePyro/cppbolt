use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_delete(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::Delete { expr, is_array } = expr else {
            return Err(crate::internal_error!(0, 0, "unexpected expression kind"));
        };

        let ty = self.get_expr_type(expr, ctx);
        self.compile_expr(expr, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1); // ptr

        if *is_array {
            if let Some(inner) = ty.deref_type()
                && let Type::Struct(sname) = inner.strip_const()
            {
                let elem_size = i32::try_from(inner.size_of(&self.sema.struct_sizes))
                    .map_err(|_| crate::internal_error!(0, 0, "array element is too large"))?;
                let dtor_name = format!("{}_~{}", sname, sname);
                let dtor_idx = self.func_indices.get(&dtor_name).copied();

                // Skip the whole operation for a null pointer, including the
                // cookie lookup and the destructor calls.
                code.push(OP_BLOCK);
                code.push(0x40);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_I32_EQZ);
                code.push(OP_BR_IF);
                write_u32(code, 0);

                if let Some(di) = dtor_idx {
                    // count = load i32 from (scratch1 - 4)
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_I32_CONST);
                    write_i32(code, 4);
                    code.push(OP_I32_SUB);
                    code.push(OP_I32_LOAD);
                    write_u32(code, 2);
                    write_u32(code, 0);
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch2); // count

                    // Loop backwards: i = count;
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch2);
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.ret_temp_i32); // i = count

                    code.push(OP_BLOCK);
                    code.push(0x40);
                    code.push(OP_LOOP);
                    code.push(0x40);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.ret_temp_i32);
                    code.push(OP_I32_EQZ);
                    code.push(OP_BR_IF);
                    write_u32(code, 1);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.ret_temp_i32);
                    code.push(OP_I32_CONST);
                    write_i32(code, 1);
                    code.push(OP_I32_SUB);
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.ret_temp_i32);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.ret_temp_i32);
                    code.push(OP_I32_CONST);
                    write_i32(code, elem_size);
                    code.push(OP_I32_MUL);
                    code.push(OP_I32_ADD); // this
                    code.push(OP_CALL);
                    write_u32(code, di);

                    code.push(OP_BR);
                    write_u32(code, 0);

                    code.push(OP_END);
                    code.push(OP_END);
                }

                // The user pointer follows a four-byte count cookie.
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_I32_CONST);
                write_i32(code, 4);
                code.push(OP_I32_SUB);
                code.push(OP_CALL);
                write_u32(code, self.func_indices["free"]);
                code.push(OP_END);

                return Ok(());
            }
            // POD array: free(ptr), which ignores null.
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_CALL);
            write_u32(code, self.func_indices["free"]);
            return Ok(());
        }

        if let Some(inner) = ty.deref_type()
            && let Type::Struct(sname) = inner.strip_const()
        {
            let layout = self.sema.struct_layouts.get(sname);
            let is_virtual_dtor =
                layout.and_then(|l| l.vtable_slots.iter().position(|s| s == "__deleting_dtor"));

            if let Some(slot_idx) = is_virtual_dtor {
                // Virtual deleting destructor dispatch via vtable slot. The
                // deleting destructor releases the object itself.
                code.push(OP_BLOCK);
                code.push(0x40);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_I32_EQZ);
                code.push(OP_BR_IF);
                write_u32(code, 0);

                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, 0); // vptr
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, (slot_idx * 4) as u32); // table_idx
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch2);

                // Push this
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch2);

                code.push(OP_CALL_INDIRECT);
                write_u32(code, self.t_i32_v);
                code.push(0);
                code.push(OP_END);

                return Ok(());
            } else {
                // Non-virtual destructor call. Guard the null case; free below
                // already accepts null.
                let dtor_name = format!("{}_~{}", sname, sname);
                if let Some(&dtor_idx) = self.func_indices.get(&dtor_name) {
                    code.push(OP_BLOCK);
                    code.push(0x40);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_I32_EQZ);
                    code.push(OP_BR_IF);
                    write_u32(code, 0);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1); // this
                    code.push(OP_CALL);
                    write_u32(code, dtor_idx);
                    code.push(OP_END);
                }
            }
        }

        // Call free
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_CALL);
        write_u32(code, self.func_indices["free"]);

        Ok(())
    }
}
