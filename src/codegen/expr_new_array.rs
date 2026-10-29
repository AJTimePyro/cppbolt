use super::expr_new::emit_new_null_check;
use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_array_new(
        &mut self,
        ty: &Type,
        array_size: Option<&Expr>,
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let elem_size = i32::try_from(ty.size_of(&self.sema.struct_sizes))
            .map_err(|_| self.internal_error("array element is too large"))?;
        if elem_size <= 0 {
            return Err(self.internal_error("array element has no size"));
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
        let slots = ArraySlots {
            elem_size,
            count_slot,
            base_slot,
        };
        match has_struct {
            Some(sname) => {
                self.compile_array_new_struct(&sname, slots, args, ctx, code)?;
            }
            None => {
                self.compile_array_new_plain(ty, slots, args, ctx, code)?;
            }
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
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let ArraySlots {
            elem_size,
            count_slot,
            base_slot,
        } = slots;
        self.emit_array_struct_alloc(count_slot, base_slot, ctx, code);

        let user_slot = ctx.alloc_frame(4, 4)?;
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, user_slot);

        for (i, arg) in args.iter().enumerate() {
            let elem_slot = ctx.alloc_frame(4, 4)?;
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, user_slot);
            code.push(OP_I32_CONST);
            write_i32(code, (i as i32) * elem_size);
            code.push(OP_I32_ADD);
            code.push(OP_I32_STORE);
            write_u32(code, 2);
            write_u32(code, elem_slot);

            if let Some(&v_off) = self.vtable_offsets.get(sname) {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, elem_slot);
                code.push(OP_I32_CONST);
                write_i32(code, v_off as i32);
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, 0);
            }

            self.compile_struct_value_ctor(sname, elem_slot, std::slice::from_ref(arg), ctx, code)?;
        }

        let ctor_idx = self.find_constructor(sname, &[]);
        let v_off = self.vtable_offsets.get(sname).copied();

        if ctor_idx.is_some() || v_off.is_some() || args.is_empty() {
            self.emit_array_element_loop(
                ArrayLoop {
                    ctor_idx,
                    v_off,
                    slots: ArraySlots {
                        elem_size,
                        count_slot,
                        base_slot,
                    },
                    start_idx: args.len() as i32,
                    user_slot,
                },
                ctx,
                code,
            );
        }
        self.emit_array_reload_user_ptr(base_slot, ctx, code);
        Ok(())
    }

    fn emit_array_struct_alloc(
        &mut self,
        count_slot: u32,
        base_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch_i64);
        code.push(OP_I32_WRAP_I64);
        code.push(OP_I32_CONST);
        write_i32(code, 4);
        code.push(OP_I32_ADD);
        code.push(OP_CALL);
        write_u32(code, self.func_indices["malloc"]);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);
        emit_new_null_check(code, ctx.scratch1);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, base_slot);

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

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_CONST);
        write_i32(code, 4);
        code.push(OP_I32_ADD);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);
    }

    fn emit_array_elem_ptr(
        &self,
        user_slot: u32,
        elem_size: i32,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, user_slot);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.ret_temp_i32);
        code.push(OP_I32_CONST);
        write_i32(code, elem_size);
        code.push(OP_I32_MUL);
        code.push(OP_I32_ADD);
    }

    fn emit_array_element_loop(
        &mut self,
        loop_info: ArrayLoop,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        let ArrayLoop {
            ctor_idx,
            v_off,
            slots,
            start_idx,
            user_slot,
        } = loop_info;
        let elem_size = slots.elem_size;
        code.push(OP_I32_CONST);
        write_i32(code, start_idx);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.ret_temp_i32);

        code.push(OP_BLOCK);
        code.push(0x40);
        code.push(OP_LOOP);
        code.push(0x40);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.ret_temp_i32);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, slots.count_slot);
        code.push(OP_I32_GE_S);
        code.push(OP_BR_IF);
        write_u32(code, 1);

        if let Some(vo) = v_off {
            self.emit_array_elem_ptr(user_slot, elem_size, ctx, code);
            code.push(OP_I32_CONST);
            write_i32(code, vo as i32);
            code.push(OP_I32_STORE);
            write_u32(code, 2);
            write_u32(code, 0);
        }

        if let Some(ci) = ctor_idx {
            self.emit_array_elem_ptr(user_slot, elem_size, ctx, code);
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

    fn emit_array_reload_user_ptr(
        &self,
        base_slot: u32,
        ctx: &FunctionContext,
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

    fn compile_array_new_plain(
        &mut self,
        ty: &Type,
        slots: ArraySlots,
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let ArraySlots {
            elem_size,
            base_slot,
            ..
        } = slots;
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch_i64);
        code.push(OP_I32_WRAP_I64);
        code.push(OP_CALL);
        write_u32(code, self.func_indices["malloc"]);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);
        emit_new_null_check(code, ctx.scratch1);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, base_slot);

        for (i, arg) in args.iter().enumerate() {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, base_slot);
            code.push(OP_I32_CONST);
            write_i32(code, (i as i32) * elem_size);
            code.push(OP_I32_ADD);

            let val_ty = self.get_expr_type(arg, ctx);
            self.compile_expr(arg, ctx, code)?;
            self.emit_conversion(&val_ty, ty, code);
            self.emit_store(ty, 0, code);
        }

        if !args.is_empty() {
            self.emit_array_plain_zero_loop(ty, &slots, args.len(), ctx, code);
        }

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, base_slot);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);
        Ok(())
    }

    fn emit_array_plain_zero_loop(
        &mut self,
        ty: &Type,
        slots: &ArraySlots,
        start_idx: usize,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        let ArraySlots {
            elem_size,
            count_slot,
            base_slot,
        } = *slots;
        code.push(OP_I32_CONST);
        write_i32(code, start_idx as i32);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.ret_temp_i32);

        code.push(OP_BLOCK);
        code.push(0x40);
        code.push(OP_LOOP);
        code.push(0x40);

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

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, base_slot);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.ret_temp_i32);
        code.push(OP_I32_CONST);
        write_i32(code, elem_size);
        code.push(OP_I32_MUL);
        code.push(OP_I32_ADD);

        self.emit_zero_value(ty, code);
        self.emit_store(ty, 0, code);

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
}
