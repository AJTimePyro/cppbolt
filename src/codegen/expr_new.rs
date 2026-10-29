use super::*;
use crate::sema::StructLayout;

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
            return Err(self.internal_error("unexpected expression kind"));
        };

        if let Some(place_expr) = placement {
            return self.compile_placement_new(ty, args, place_expr, ctx, code);
        }
        if *is_array {
            let array_size = array_size.as_deref();
            self.compile_array_new(ty, array_size, args, ctx, code)
        } else {
            self.compile_scalar_new(ty, args, ctx, code)
        }
    }

    /// Installs the vtable and runs the constructor (or scalar store) for an
    /// object whose address is stored in `obj_slot`.
    fn emit_construct_object_at(
        &mut self,
        ty: &Type,
        args: &[Expr],
        obj_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Type::Struct(sname) = ty.strip_const() {
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
                for (i, a) in full_args.iter().enumerate() {
                    let param_ty = sig.param_types.get(i + 1);
                    self.compile_call_arg(a, param_ty, ctx, code)?;
                }
                code.push(OP_CALL);
                write_u32(code, ctor_idx);
            } else if let Some(layout) = self.sema.struct_layouts.get(sname).cloned() {
                self.emit_construct_aggregate_at(&layout, args, obj_slot, ctx, code)?;
            }
        } else {
            self.emit_construct_scalar_at(ty, args, obj_slot, ctx, code)?;
        }
        Ok(())
    }

    fn emit_construct_aggregate_at(
        &mut self,
        layout: &StructLayout,
        args: &[Expr],
        obj_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let mut field_idx = 0;
        for a in args {
            if let Some(fld) = layout.fields.get(field_idx) {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, obj_slot); // this
                let a_ty = self.get_expr_type(a, ctx);
                self.compile_expr(a, ctx, code)?;
                self.emit_conversion(&a_ty, &fld.ty, code);
                self.emit_store(&fld.ty, fld.offset as u32, code);
                field_idx += 1;
            }
        }
        while field_idx < layout.fields.len() {
            let fld = &layout.fields[field_idx];
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, obj_slot); // this
            self.emit_zero_value(&fld.ty, code);
            self.emit_store(&fld.ty, fld.offset as u32, code);
            field_idx += 1;
        }
        Ok(())
    }

    fn emit_construct_scalar_at(
        &mut self,
        ty: &Type,
        args: &[Expr],
        obj_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, obj_slot);
        if args.is_empty() {
            self.emit_zero_value(ty, code);
        } else {
            let value_ty = self.get_expr_type(&args[0], ctx);
            self.compile_expr(&args[0], ctx, code)?;
            self.emit_conversion(&value_ty, ty, code);
        }
        self.emit_store(ty, 0, code);
        Ok(())
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

        self.emit_construct_object_at(ty, args, place_slot, ctx, code)?;
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, place_slot);
        Ok(())
    }

    fn compile_scalar_new(
        &mut self,
        ty: &Type,
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let size = i32::try_from(ty.size_of(&self.sema.struct_sizes))
            .map_err(|_| self.internal_error("object is too large"))?;
        if size <= 0 {
            return Err(self.internal_error("object has no size"));
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

        self.emit_construct_object_at(ty, args, obj_slot, ctx, code)?;
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, obj_slot);
        Ok(())
    }
}

/// `operator new` must trap when the allocator cannot satisfy the request.
pub(super) fn emit_new_null_check(code: &mut Vec<u8>, pointer_local: u32) {
    code.push(OP_LOCAL_GET);
    write_u32(code, pointer_local);
    code.push(OP_I32_EQZ);
    code.push(OP_IF);
    code.push(0x40);
    code.push(OP_UNREACHABLE);
    code.push(OP_END);
}
