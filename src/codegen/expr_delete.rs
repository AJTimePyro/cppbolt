use super::*;

impl<'a> Codegen<'a> {
    /// Lowers scalar and array `delete`.
    ///
    /// Class arrays read their element count from a four-byte cookie before
    /// the user pointer. Virtual scalar deletes dispatch through the deleting
    /// destructor slot, which also releases the object.
    pub(super) fn compile_delete(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::Delete { expr, is_array } = expr else {
            return Err(self.internal_error("unexpected expression kind"));
        };

        let ty = self.get_expr_type(expr, ctx);
        self.compile_expr(expr, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1); // ptr

        if *is_array {
            self.compile_delete_array(&ty, ctx, code)?;
        } else {
            self.compile_delete_scalar(&ty, ctx, code)?;
        }

        Ok(())
    }

    fn compile_delete_array(
        &mut self,
        ty: &Type,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Some(inner) = ty.deref_type()
            && let Type::Struct(sname) = inner.strip_const()
        {
            self.compile_delete_class_array(sname, inner, ctx, code)?;
            return Ok(());
        }
        // POD array: free(ptr), which ignores null.
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_CALL);
        write_u32(code, self.func_indices["free"]);
        Ok(())
    }

    fn compile_delete_class_array(
        &mut self,
        sname: &str,
        inner: &Type,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let elem_size = i32::try_from(inner.size_of(&self.sema.struct_sizes))
            .map_err(|_| self.internal_error("array element is too large"))?;
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
            self.emit_class_array_dtor_loop(di, elem_size, ctx, code);
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

        Ok(())
    }

    fn emit_class_array_dtor_loop(
        &mut self,
        dtor_idx: u32,
        elem_size: i32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
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

        // Destroy elements from last to first. The loop counter starts at
        // `count` and is decremented before each call.
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
        write_u32(code, dtor_idx);

        code.push(OP_BR);
        write_u32(code, 0);

        code.push(OP_END);
        code.push(OP_END);
    }

    fn compile_delete_scalar(
        &mut self,
        ty: &Type,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Some(inner) = ty.deref_type()
            && let Type::Struct(sname) = inner.strip_const()
            && self.compile_delete_class_scalar(sname, ctx, code)?
        {
            return Ok(());
        }

        // Scalar delete and the non-class fallback release the allocation;
        // `free` ignores null.
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_CALL);
        write_u32(code, self.func_indices["free"]);

        Ok(())
    }
    /// Loads the deleting destructor from the subobject's vtable and calls it.
    /// `scratch1` holds the object pointer (possibly a base subobject pointer).
    fn emit_virtual_delete_dispatch(
        &mut self,
        slot_idx: usize,
        vptr_offset: u32,
        fn_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        // scratch2 holds the vptr of the subobject being deleted.
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        if vptr_offset != 0 {
            code.push(OP_I32_CONST);
            write_i32(code, vptr_offset as i32);
            code.push(OP_I32_ADD);
        }
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, 0);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch2);
        // entry[0]: function table index.
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch2);
        code.push(OP_I32_CONST);
        write_i32(code, (slot_idx * 8) as i32);
        code.push(OP_I32_ADD);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, 0);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, fn_slot);
        // entry[1]: adjustment to the final overrider's subobject.
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        if vptr_offset != 0 {
            code.push(OP_I32_CONST);
            write_i32(code, vptr_offset as i32);
            code.push(OP_I32_ADD);
        }
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch2);
        code.push(OP_I32_CONST);
        write_i32(code, (slot_idx * 8 + 4) as i32);
        code.push(OP_I32_ADD);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, 0);
        code.push(OP_I32_ADD);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, fn_slot);
        code.push(OP_CALL_INDIRECT);
        write_u32(code, self.t_i32_v);
        code.push(0);
    }

    /// Returns `true` when the object was released by a virtual destructor.
    fn compile_delete_class_scalar(
        &mut self,
        sname: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<bool, CompileError> {
        let layout = self.sema.struct_layouts.get(sname);
        let is_virtual_dtor = layout.and_then(|l| {
            l.vtables.iter().find_map(|t| {
                t.slots
                    .iter()
                    .position(|s| s.name == "__deleting_dtor")
                    .map(|idx| (idx, t.vptr_offset as u32))
            })
        });
        if let Some((slot_idx, vptr_offset)) = is_virtual_dtor {
            // Virtual deleting destructor dispatch via vtable slot. The
            // deleting destructor releases the object itself.
            code.push(OP_BLOCK);
            code.push(0x40);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_I32_EQZ);
            code.push(OP_BR_IF);
            write_u32(code, 0);

            let fn_slot = ctx.alloc_frame(4, 4)?;
            self.emit_virtual_delete_dispatch(slot_idx, vptr_offset, fn_slot, ctx, code);
            code.push(OP_END);

            return Ok(true);
        }

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

        Ok(false)
    }
}
