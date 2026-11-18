//! Member initializer lowering.

use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_struct_field_init(
        &mut self,
        sname: &str,
        fld_offset: usize,
        init_args: Option<&[Expr]>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let arg_types: Vec<Type> = init_args
            .map(|args| args.iter().map(|a| self.get_expr_type(a, ctx)).collect())
            .unwrap_or_default();
        if let Some((ctor_idx, sig)) =
            self.find_constructor_sig_with_args(sname, &arg_types, init_args)
        {
            code.push(OP_LOCAL_GET);
            write_u32(code, 0); // this
            code.push(OP_I32_CONST);
            write_i32(code, fld_offset as i32);
            code.push(OP_I32_ADD);
            let empty = [];
            let args = init_args.unwrap_or(&empty);
            let full_args = self.constructor_arguments(&sig, args);
            for (i, a) in full_args.iter().enumerate() {
                let param_ty = sig.param_types.get(i + 1);
                self.compile_call_arg(a, param_ty, ctx, code)?;
            }
            code.push(OP_CALL);
            write_u32(code, ctor_idx);
        } else if let Some(args) = init_args
            && args.len() == 1
            && matches!(
            arg_types[0].clean_type(),
            Type::Struct(s) if QualifiedName::new(s).matches(QualifiedName::new(sname))
            )
        {
            self.compile_lvalue_addr(&args[0], ctx, code)?;
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1);
            let size = self
                .sema
                .struct_layouts
                .get(sname)
                .map(|l| l.size)
                .ok_or_else(|| self.internal_error("missing struct layout"))?;
            crate::codegen::raii::emit_struct_byte_copy(
                size,
                fld_offset as u32,
                |code| {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, 0); // this
                },
                |code| {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                },
                code,
            );
        } else if let Some(layout) = self.sema.struct_layouts.get(sname).cloned()
            && let Some(args) = init_args
        {
            self.emit_field_initializers(&layout, args, fld_offset as u32, ctx, code)?;
        }
        Ok(())
    }

    pub(super) fn compile_array_member_init(
        &mut self,
        (fld_offset, elem_ty, count, list): (usize, &Type, usize, &[Expr]),
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
            .map_err(|_| self.internal_error("member is too large"))?;
        for i in 0..count {
            let elem_offset = (fld_offset as u32)
                .checked_add(
                    (i as u32)
                        .checked_mul(elem_size)
                        .ok_or_else(|| self.internal_error("member is too large"))?,
                )
                .ok_or_else(|| self.internal_error("member is too large"))?;
            if let Type::Struct(sname) = elem_ty.strip_const() {
                let item_args = list.get(i).map(|item| match item {
                    Expr::InitializerList { elements, .. }
                    | Expr::TypedInitList { elements, .. } => elements.as_slice(),
                    _ => std::slice::from_ref(item),
                });
                self.compile_struct_field_init(sname, elem_offset as usize, item_args, ctx, code)?;
            } else {
                code.push(OP_LOCAL_GET);
                write_u32(code, 0); // this
                if let Some(item) = list.get(i) {
                    let item_ty = self.get_expr_type(item, ctx);
                    self.compile_expr(item, ctx, code)?;
                    self.emit_conversion(&item_ty, elem_ty, code);
                } else {
                    self.emit_zero_value(elem_ty, code);
                }
                self.emit_store(elem_ty, elem_offset, code);
            }
        }
        Ok(())
    }

    pub(super) fn compile_member_init_expr(
        &mut self,
        fld: &crate::sema::StructField,
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if fld.ty.is_reference() {
            code.push(OP_LOCAL_GET);
            write_u32(code, 0); // this
            self.compile_lvalue_addr(&args[0], ctx, code)?;
            self.emit_store(&fld.ty, fld.offset as u32, code);
        } else if let Type::Array(elem_ty, count) = fld.ty.strip_const() {
            let init_list = match &args[0] {
                Expr::InitializerList { elements, .. } | Expr::TypedInitList { elements, .. } => {
                    Some(elements.as_slice())
                }
                _ => None,
            };
            if let Some(list) = init_list {
                self.compile_array_member_init((fld.offset, elem_ty, *count, list), ctx, code)?;
            } else {
                self.compile_expr(&args[0], ctx, code)?;
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1);
                let size = fld.ty.size_of(&self.sema.struct_sizes);
                crate::codegen::raii::emit_struct_byte_copy(
                    size,
                    fld.offset as u32,
                    |code| {
                        code.push(OP_LOCAL_GET);
                        write_u32(code, 0);
                    },
                    |code| {
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.scratch1);
                    },
                    code,
                );
            }
        } else {
            code.push(OP_LOCAL_GET);
            write_u32(code, 0); // this
            let expr = construct::unpack_single_element_init(&args[0]);
            let item_ty = self.get_expr_type(expr, ctx);
            self.compile_expr(expr, ctx, code)?;
            self.emit_conversion(&item_ty, &fld.ty, code);
            self.emit_store(&fld.ty, fld.offset as u32, code);
        }
        Ok(())
    }

}
