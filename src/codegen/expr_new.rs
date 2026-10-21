use super::*;

impl<'a> Codegen<'a> {
    /// Lowers scalar/array `new`, including placement new.
    ///
    /// Class arrays reserve a four-byte element-count cookie before the user
    /// pointer so `delete[]` can run the right number of destructors.
    pub(super) fn compile_new(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::New {
            ty,
            args,
            is_array,
            array_size,
            placement,
        } = expr
        else {
            return Err(crate::internal_error!(0, 0, "unexpected expression kind"));
        };

        if let Some(place_expr) = placement {
            return self.compile_placement_new(ty, args, place_expr, ctx, code);
        }
        if *is_array {
            let array_size = array_size.as_ref().map(|size| size.as_ref());
            self.compile_array_new(ty, array_size, ctx, code)
        } else {
            self.compile_scalar_new(ty, args, ctx, code)
        }
    }

    fn compile_placement_new(
        &mut self,
        ty: &Type,
        args: &[Expr],
        place_expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let place_slot = ctx.alloc_frame(4, 4)?;
        self.compile_expr(place_expr, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1); // placement ptr
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, place_slot);

        if let Type::Struct(sname) = ty.strip_const() {
            if let Some(&v_off) = self.vtable_offsets.get(sname) {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, place_slot);
                code.push(OP_I32_CONST);
                write_i32(code, v_off as i32);
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, 0);
            }
            let arg_types: Vec<Type> = args.iter().map(|a| self.get_expr_type(a, ctx)).collect();
            if let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, place_slot); // this
                let full_args = self.constructor_arguments(&sig, args);
                for a in &full_args {
                    self.compile_expr(a, ctx, code)?;
                }
                code.push(OP_CALL);
                write_u32(code, ctor_idx);
            }
        } else if !args.is_empty() {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, place_slot);
            let value_ty = self.get_expr_type(&args[0], ctx);
            self.compile_expr(&args[0], ctx, code)?;
            self.emit_conversion(&value_ty, ty, code);
            self.emit_store(ty, 0, code);
        }
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, place_slot);
        Ok(())
    }

    fn compile_array_new(
        &mut self,
        ty: &Type,
        array_size: Option<&Expr>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let elem_size = i32::try_from(ty.size_of(&self.sema.struct_sizes))
            .map_err(|_| crate::internal_error!(0, 0, "array element is too large"))?;
        if elem_size <= 0 {
            return Err(crate::internal_error!(0, 0, "array element has no size"));
        }
        let has_struct = if let Type::Struct(sname) = ty.strip_const() {
            Some(sname.clone())
        } else {
            None
        };

        let count_slot = ctx.alloc_frame(4, 4)?;
        let base_slot = ctx.alloc_frame(4, 4)?;

        self.compile_array_new_count(array_size, count_slot, ctx, code)?;
        self.compile_array_new_size_check(elem_size, ctx, code);
        match has_struct {
            Some(sname) => {
                self.compile_array_new_struct(
                    &sname,
                    ArraySlots {
                        elem_size,
                        count_slot,
                    },
                    base_slot,
                    ctx,
                    code,
                );
            }
            None => self.compile_array_new_plain(ctx, code),
        }
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        Ok(())
    }

    fn compile_array_new_count(
        &mut self,
        array_size: Option<&Expr>,
        count_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Some(sz_expr) = array_size {
            let size_ty = self.get_expr_type(sz_expr, ctx);
            self.compile_expr(sz_expr, ctx, code)?;
            self.emit_conversion(&size_ty, &Type::UnsignedInt, code);
        } else {
            code.push(OP_I32_CONST);
            write_i32(code, 1);
        }
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch2); // count
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch2);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, count_slot);

        // Reject negative element counts instead of letting the size wrap.
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch2);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_I32_LT_S);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_UNREACHABLE);
        code.push(OP_END);
        Ok(())
    }

    fn compile_array_new_size_check(
        &mut self,
        elem_size: i32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        // Compute the payload size in 64 bits and trap if it does not fit a
        // signed 32-bit allocation (with room for the array cookie).
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch2);
        code.push(OP_I64_EXTEND_I32_U);
        code.push(OP_I64_CONST);
        write_i64(code, elem_size as i64);
        code.push(OP_I64_MUL);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch_i64);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch_i64);
        code.push(OP_I64_CONST);
        write_i64(code, i32::MAX as i64 - 4);
        code.push(OP_I64_GT_U);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_UNREACHABLE);
        code.push(OP_END);
    }

    fn compile_array_new_struct(
        &mut self,
        sname: &str,
        slots: ArraySlots,
        base_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        let ArraySlots {
            elem_size,
            count_slot,
        } = slots;
        self.emit_array_struct_alloc(count_slot, base_slot, ctx, code);

        let ctor_idx = self.find_constructor(sname, &[]);
        let v_off = self.vtable_offsets.get(sname).copied();

        if ctor_idx.is_some() || v_off.is_some() {
            self.emit_array_element_loop(
                ctor_idx,
                v_off,
                ArraySlots {
                    elem_size,
                    count_slot,
                },
                ctx,
                code,
            );
            // Constructors may clobber scratch locals, so reload the
            // user pointer from its frame slot.
            self.emit_array_reload_user_ptr(base_slot, ctx, code);
        } else {
            self.emit_array_reload_user_ptr(base_slot, ctx, code);
        }
    }

    /// Allocates the array payload and stores the element-count cookie.
    fn emit_array_struct_alloc(
        &mut self,
        count_slot: u32,
        base_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        // Allocation size: count * elem_size + 4 (for cookie)
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch_i64);
        code.push(OP_I32_WRAP_I64);
        code.push(OP_I32_CONST);
        write_i32(code, 4);
        code.push(OP_I32_ADD);
        code.push(OP_CALL);
        write_u32(code, self.func_indices["malloc"]);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1); // base ptr
        emit_new_null_check(code, ctx.scratch1);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, base_slot);

        // Store count cookie at offset 0
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, count_slot);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, 0);

        // user ptr = base ptr + 4
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_CONST);
        write_i32(code, 4);
        code.push(OP_I32_ADD);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1); // user ptr
    }

    /// Installs each element's vtable and runs its constructor.
    fn emit_array_element_loop(
        &mut self,
        ctor_idx: Option<u32>,
        v_off: Option<u32>,
        slots: ArraySlots,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        let ArraySlots {
            elem_size,
            count_slot,
        } = slots;
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.ret_temp_i32); // loop index i

        code.push(OP_BLOCK);
        code.push(0x40);
        code.push(OP_LOOP);
        code.push(0x40);

        // if i >= count break;
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.ret_temp_i32);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, count_slot);
        code.push(OP_I32_GE_S);
        code.push(OP_BR_IF);
        write_u32(code, 1);

        // Install each element's vtable, then construct it.
        if let Some(vo) = v_off {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.ret_temp_i32);
            code.push(OP_I32_CONST);
            write_i32(code, elem_size);
            code.push(OP_I32_MUL);
            code.push(OP_I32_ADD);
            code.push(OP_I32_CONST);
            write_i32(code, vo as i32);
            code.push(OP_I32_STORE);
            write_u32(code, 2);
            write_u32(code, 0);
        }

        if let Some(ci) = ctor_idx {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.ret_temp_i32);
            code.push(OP_I32_CONST);
            write_i32(code, elem_size);
            code.push(OP_I32_MUL);
            code.push(OP_I32_ADD); // this
            code.push(OP_CALL);
            write_u32(code, ci);
        }

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.ret_temp_i32);
        code.push(OP_I32_CONST);
        write_i32(code, 1);
        code.push(OP_I32_ADD);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.ret_temp_i32);

        code.push(OP_BR);
        write_u32(code, 0);

        code.push(OP_END);
        code.push(OP_END);
    }

    /// Reloads the user pointer from its frame slot.
    fn emit_array_reload_user_ptr(
        &mut self,
        base_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, base_slot);
        code.push(OP_I32_CONST);
        write_i32(code, 4);
        code.push(OP_I32_ADD);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);
    }

    fn compile_array_new_plain(&mut self, ctx: &mut FunctionContext, code: &mut Vec<u8>) {
        // Plain arrays have no cookie and no element constructors.
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch_i64);
        code.push(OP_I32_WRAP_I64);
        code.push(OP_CALL);
        write_u32(code, self.func_indices["malloc"]);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);
        emit_new_null_check(code, ctx.scratch1);
    }

    fn compile_scalar_new(
        &mut self,
        ty: &Type,
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let size = i32::try_from(ty.size_of(&self.sema.struct_sizes))
            .map_err(|_| crate::internal_error!(0, 0, "object is too large"))?;
        if size <= 0 {
            return Err(crate::internal_error!(0, 0, "object has no size"));
        }
        let obj_slot = ctx.alloc_frame(4, 4)?;
        code.push(OP_I32_CONST);
        write_i32(code, size);
        code.push(OP_CALL);
        write_u32(code, self.func_indices["malloc"]);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1); // heap obj ptr
        emit_new_null_check(code, ctx.scratch1);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, obj_slot);

        if let Type::Struct(sname) = ty.strip_const() {
            // Install the vtable before the constructor runs, matching the
            // C++ object model.
            if let Some(&v_off) = self.vtable_offsets.get(sname) {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, obj_slot);
                code.push(OP_I32_CONST);
                write_i32(code, v_off as i32);
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, 0);
            }
            let arg_types: Vec<Type> = args.iter().map(|a| self.get_expr_type(a, ctx)).collect();
            if let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, obj_slot); // this
                let full_args = self.constructor_arguments(&sig, args);
                for a in &full_args {
                    self.compile_expr(a, ctx, code)?;
                }
                code.push(OP_CALL);
                write_u32(code, ctor_idx);
            }
        } else if !args.is_empty() {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, obj_slot);
            let value_ty = self.get_expr_type(&args[0], ctx);
            self.compile_expr(&args[0], ctx, code)?;
            self.emit_conversion(&value_ty, ty, code);
            self.emit_store(ty, 0, code);
        }
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, obj_slot);
        Ok(())
    }
}

/// `operator new` must trap when the allocator cannot satisfy the request.
fn emit_new_null_check(code: &mut Vec<u8>, pointer_local: u32) {
    code.push(OP_LOCAL_GET);
    write_u32(code, pointer_local);
    code.push(OP_I32_EQZ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_UNREACHABLE);
    code.push(OP_END);
}
