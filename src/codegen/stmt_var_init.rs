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
        if let Expr::InitializerList(list) = init
            && list.len() == 1
            && !matches!(ty.strip_const(), Type::Struct(_) | Type::Array(_, _))
        {
            return self.compile_aggregate_local(ty, &list[0], offset, ctx, code);
        }

        if let Type::Struct(sname) = ty.strip_const() {
            return self.compile_aggregate_struct(sname, init, offset, ctx, code);
        }

        if let (Type::Array(elem_ty, _), Expr::InitializerList(list)) = (ty.strip_const(), init) {
            return self.compile_aggregate_array(elem_ty, list, offset, ctx, code);
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
        if let Expr::InitializerList(list) = init {
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
        let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) else {
            return Err(CompileError::new(
                format!("No matching constructor for '{}'", sname),
                self.current_line,
                self.current_col,
            ));
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
            let full_args = self.constructor_arguments(&sig, list);
            for (i, arg) in full_args.iter().enumerate() {
                self.compile_call_arg(arg, sig.param_types.get(i + 1), ctx, code)?;
            }
            code.push(OP_CALL);
            write_u32(code, ctor_idx);
            return Ok(());
        }
        if let Some(layout) = self.sema.struct_layouts.get(sname).cloned() {
            for (i, item) in list.iter().enumerate() {
                if let Some(fld) = layout.fields.get(i) {
                    let field_offset = offset
                        .checked_add(fld.offset as u32)
                        .ok_or_else(|| self.internal_error("object is too large"))?;
                    self.compile_aggregate_local(&fld.ty, item, field_offset, ctx, code)?;
                }
            }
        }
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
            let copy_ctor_arg = vec![Type::Reference(Box::new(Type::Const(Box::new(
                Type::Struct(sname.to_string()),
            ))))];
            if self.is_struct_lvalue(init, ctx) {
                if let Some(ctor_idx) = self.find_constructor(sname, &copy_ctor_arg) {
                    self.emit_local_address(offset, ctx, code);
                    self.compile_lvalue_addr(init, ctx, code)?;
                    code.push(OP_CALL);
                    write_u32(code, ctor_idx);
                } else {
                    self.compile_lvalue_addr(init, ctx, code)?;
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch1);
                    self.emit_copy_struct(sname, offset, ctx, code);
                }
            } else {
                self.compile_expr(init, ctx, code)?;
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1);
                if let Some(ctor_idx) = self.find_constructor(sname, &copy_ctor_arg) {
                    self.emit_local_address(offset, ctx, code);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_CALL);
                    write_u32(code, ctor_idx);
                } else {
                    self.emit_copy_struct(sname, offset, ctx, code);
                }
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
        elem_ty: &Type,
        list: &[Expr],
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
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
        Ok(())
    }
}
