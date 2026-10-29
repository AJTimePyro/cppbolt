//! Lowering of aggregate initializers for local variables.

use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_aggregate_local(
        &mut self,
        ty: &Type,
        init: &Expr,
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Expr::InitializerList { elements: list, .. }
        | Expr::TypedInitList { elements: list, .. } = init
        {
            if list.is_empty() && !matches!(ty.strip_const(), Type::Struct(_) | Type::Array(_, _)) {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                self.emit_zero_value(ty, code);
                self.emit_store(ty, offset, code);
                return Ok(());
            }
            if list.len() == 1 && !matches!(ty.strip_const(), Type::Struct(_) | Type::Array(_, _)) {
                return self.compile_aggregate_local(ty, &list[0], offset, ctx, code);
            }
        }

        if let Type::Struct(sname) = ty.strip_const() {
            return self.compile_aggregate_struct(sname, init, offset, ctx, code);
        }

        if let (
            Type::Array(_, _),
            Expr::InitializerList { elements: list, .. }
            | Expr::TypedInitList { elements: list, .. },
        ) = (ty.strip_const(), init)
        {
            return self.compile_aggregate_array(ty, list, offset, ctx, code);
        }

        let item_ty = self.get_expr_type(init, ctx);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        self.compile_expr(init, ctx, code)?;
        self.emit_conversion(&item_ty, ty, code);
        self.emit_store(ty, offset, code);
        Ok(())
    }

    pub(super) fn compile_aggregate_struct(
        &mut self,
        sname: &str,
        init: &Expr,
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Expr::Call { callee, args, .. } = init
            && matches!(&**callee, Expr::Identifier(name) if name == sname)
        {
            return self.compile_aggregate_struct_call(sname, args, offset, ctx, code);
        }
        if let Expr::InitializerList { elements: list, .. }
        | Expr::TypedInitList { elements: list, .. } = init
        {
            return self.compile_aggregate_struct_list(sname, list, offset, ctx, code);
        }
        self.compile_aggregate_struct_value(sname, init, offset, ctx, code)
    }

    pub(super) fn compile_aggregate_struct_call(
        &mut self,
        sname: &str,
        args: &[Expr],
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let arg_types: Vec<Type> = args
            .iter()
            .map(|arg| self.get_expr_type(arg, ctx))
            .collect();
        // A single argument of the same class is copy initialization: the
        // copy constructor runs for an lvalue, and a prvalue initializes the
        // object directly.
        if let [only] = args
            && matches!(arg_types[0].strip_const(), Type::Struct(other) if other == sname)
        {
            return self.compile_aggregate_struct_value(sname, only, offset, ctx, code);
        }
        let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) else {
            return Ok(());
        };
        self.emit_local_address(offset, ctx, code);
        let full_args = self.constructor_arguments(&sig, args);
        for (i, arg) in full_args.iter().enumerate() {
            self.compile_call_arg(arg, sig.param_types.get(i + 1), ctx, code)?;
        }
        code.push(OP_CALL);
        write_u32(code, ctor_idx);
        Ok(())
    }

    pub(super) fn compile_aggregate_struct_list(
        &mut self,
        sname: &str,
        list: &[Expr],
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let arg_types: Vec<Type> = list
            .iter()
            .map(|arg| self.get_expr_type(arg, ctx))
            .collect();
        if let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) {
            self.emit_local_address(offset, ctx, code);
            for (i, item) in list.iter().enumerate() {
                let param_ty = sig.param_types.get(i + 1);
                self.compile_call_arg(item, param_ty, ctx, code)?;
            }
            code.push(OP_CALL);
            write_u32(code, ctor_idx);
            return Ok(());
        }
        if let Some(layout) = self.sema.struct_layouts.get(sname).cloned() {
            let mut field_idx = 0;
            let mut elem_idx = 0;
            if let Some(base_name) = layout.base_classes.first()
                && let Some(base_layout) = self.sema.struct_layouts.get(base_name).cloned()
            {
                let base_field_count = base_layout.fields.len();
                if elem_idx < list.len()
                    && matches!(
                        &list[elem_idx],
                        Expr::InitializerList { .. } | Expr::TypedInitList { .. }
                    )
                {
                    let base_ty = Type::Struct(base_name.clone());
                    self.compile_aggregate_local(&base_ty, &list[elem_idx], offset, ctx, code)?;
                    elem_idx += 1;
                    field_idx += base_field_count;
                }
            }
            while elem_idx < list.len() {
                let item = &list[elem_idx];
                if let Some(fld) = layout.fields.get(field_idx) {
                    let field_offset = offset
                        .checked_add(fld.offset as u32)
                        .ok_or_else(|| self.internal_error("object is too large"))?;
                    self.compile_aggregate_local(&fld.ty, item, field_offset, ctx, code)?;
                    field_idx += 1;
                }
                elem_idx += 1;
            }
            while field_idx < layout.fields.len() {
                let fld = &layout.fields[field_idx];
                let field_offset = offset
                    .checked_add(fld.offset as u32)
                    .ok_or_else(|| self.internal_error("object is too large"))?;
                self.emit_zero_local(&fld.ty, field_offset, ctx, code)?;
                field_idx += 1;
            }
        }
        Ok(())
    }

    pub(super) fn emit_zero_value(&self, ty: &Type, code: &mut Vec<u8>) {
        match ty.strip_const() {
            Type::Float => {
                code.push(OP_F32_CONST);
                code.extend(&0.0f32.to_le_bytes());
            }
            Type::Double => {
                code.push(OP_F64_CONST);
                code.extend(&0.0f64.to_le_bytes());
            }
            Type::LongLong | Type::UnsignedLongLong => {
                code.push(OP_I64_CONST);
                write_i64(code, 0);
            }
            _ => {
                code.push(OP_I32_CONST);
                write_i32(code, 0);
            }
        }
    }

    pub(super) fn emit_zero_local(
        &mut self,
        ty: &Type,
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let clean = ty.strip_const();
        if let Type::Struct(sname) = clean {
            if let Some((ctor_idx, _)) = self.find_constructor_sig(sname, &[]) {
                self.emit_local_address(offset, ctx, code);
                code.push(OP_CALL);
                write_u32(code, ctor_idx);
                return Ok(());
            }
            if let Some(layout) = self.sema.struct_layouts.get(sname).cloned() {
                for fld in &layout.fields {
                    let field_offset = offset
                        .checked_add(fld.offset as u32)
                        .ok_or_else(|| self.internal_error("object is too large"))?;
                    self.emit_zero_local(&fld.ty, field_offset, ctx, code)?;
                }
                return Ok(());
            }
        } else if let Type::Array(elem_ty, count) = clean {
            let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
                .map_err(|_| self.internal_error("array element is too large"))?;
            for i in 0..*count {
                let elem_offset = offset
                    .checked_add((i as u32) * elem_size)
                    .ok_or_else(|| self.internal_error("array is too large"))?;
                self.emit_zero_local(elem_ty, elem_offset, ctx, code)?;
            }
            return Ok(());
        }

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        self.emit_zero_value(ty, code);
        self.emit_store(ty, offset, code);
        Ok(())
    }

    pub(super) fn compile_aggregate_struct_value(
        &mut self,
        sname: &str,
        init: &Expr,
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let init_ty = self.get_expr_type(init, ctx);
        if matches!(
            init_ty.strip_const().strip_reference(),
            Type::Struct(other) if other == sname
        ) {
            if self.is_struct_lvalue(init, ctx) {
                let is_const = init_ty.strip_reference().is_const();
                if let Some(ctor_idx) = self.find_copy_constructor(sname, is_const) {
                    self.emit_local_address(offset, ctx, code);
                    self.compile_lvalue_addr(init, ctx, code)?;
                    code.push(OP_CALL);
                    write_u32(code, ctor_idx);
                } else {
                    self.compile_lvalue_addr(init, ctx, code)?;
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch1);
                    self.emit_copy_struct(sname, offset, ctx, code)?;
                }
            } else {
                let temp_count_before = ctx.temp_objects.len();
                self.compile_expr(init, ctx, code)?;
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1);
                self.emit_copy_struct(sname, offset, ctx, code)?;
                // The prvalue's heap storage is copied into the local, so it
                // can be freed immediately.
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_CALL);
                write_u32(code, self.func_indices["free"]);
                // Drop the temporary from the cleanup list so statement-end
                // RAII does not double-free it.
                ctx.temp_objects.truncate(temp_count_before);
            }
            return Ok(());
        }

        let arg_types = vec![init_ty.clone()];
        if let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) {
            self.emit_local_address(offset, ctx, code);
            self.compile_call_arg(init, sig.param_types.get(1), ctx, code)?;
            code.push(OP_CALL);
            write_u32(code, ctor_idx);
            return Ok(());
        }
        Err(CompileError::new(
            format!(
                "Cannot initialize struct '{}' with value of type {:?}",
                sname, init_ty
            ),
            self.current_line,
            self.current_col,
        ))
    }

    pub(super) fn compile_aggregate_array(
        &mut self,
        array_ty: &Type,
        list: &[Expr],
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Type::Array(elem_ty, total_count) = array_ty.strip_const() else {
            return Ok(());
        };
        let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
            .map_err(|_| self.internal_error("array is too large"))?;
        for (i, item) in list.iter().enumerate() {
            let item_offset = offset
                .checked_add(
                    (i as u32)
                        .checked_mul(elem_size)
                        .ok_or_else(|| self.internal_error("array is too large"))?,
                )
                .ok_or_else(|| self.internal_error("array is too large"))?;
            self.compile_aggregate_local(elem_ty, item, item_offset, ctx, code)?;
        }
        for i in list.len()..*total_count {
            let item_offset = offset
                .checked_add(
                    (i as u32)
                        .checked_mul(elem_size)
                        .ok_or_else(|| self.internal_error("array is too large"))?,
                )
                .ok_or_else(|| self.internal_error("array is too large"))?;
            self.emit_zero_local(elem_ty, item_offset, ctx, code)?;
        }
        Ok(())
    }
}
