use super::*;

impl<'a> Codegen<'a> {
    /// Lowers one call argument.
    ///
    /// Reference parameters take an lvalue address; by-value structs may need
    /// a copy-constructor temporary, whose cleanup is transferred to the
    /// callee's owned parameter copy.
    pub(super) fn compile_call_arg(
        &mut self,
        a: &Expr,
        param_ty: Option<&Type>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let is_ref = param_ty.map(|p| p.is_reference()).unwrap_or(false);
        let is_byval_struct = param_ty
            .map(|p| matches!(p.strip_const(), Type::Struct(_)))
            .unwrap_or(false);
        let temp_count_before = ctx.temp_objects.len();

        if is_ref {
            let aty = self.get_expr_type(a, ctx);
            let Some(param_ty) = param_ty else {
                return Err(CompileError::new(
                    "internal compiler error: missing reference parameter type",
                    1,
                    1,
                )
                .with_kind(crate::diagnostics::ErrorKind::Internal));
            };
            let Some(clean_p) = param_ty.deref_type() else {
                return Err(CompileError::new(
                    "internal compiler error: invalid reference parameter type",
                    1,
                    1,
                )
                .with_kind(crate::diagnostics::ErrorKind::Internal));
            };
            let clean_p = clean_p.strip_const();
            let clean_a = aty.strip_const();
            let is_same = if let (Type::Struct(s1), Type::Struct(s2)) = (&clean_p, &clean_a) {
                QualifiedName::new(s1).matches(QualifiedName::new(s2))
            } else {
                clean_p == clean_a
            };
            if !is_same
                && matches!(clean_p, Type::Struct(_))
                && let Type::Struct(psname) = clean_p
                && let Some(ctor_idx) = self.find_constructor(psname, std::slice::from_ref(clean_a))
            {
                let size = i32::try_from(clean_p.size_of(&self.sema.struct_sizes))
                    .map_err(|_| crate::internal_error!(0, 0, "object is too large"))?;
                let temp_ptr_slot = ctx.alloc_frame(4, 4)?;
                code.push(OP_I32_CONST);
                write_i32(code, size);
                code.push(OP_CALL);
                write_u32(code, self.func_indices["malloc"]);
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1); // temp_ptr
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, temp_ptr_slot);

                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, temp_ptr_slot);
                self.compile_expr(a, ctx, code)?;
                code.push(OP_CALL);
                write_u32(code, ctor_idx);

                let temp_slot = ctx.alloc_frame(4, 4)?;
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, temp_ptr_slot);
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, temp_slot);

                ctx.temp_objects.push((psname.clone(), temp_slot, true));

                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, temp_ptr_slot);
                return Ok(());
            }
            self.compile_lvalue_addr(a, ctx, code)?;
        } else {
            let aty = self.get_expr_type(a, ctx);
            if is_byval_struct
                && self.is_struct_lvalue(a, ctx)
                && let Type::Struct(sname) = aty.strip_const().strip_reference()
            {
                let copy_ctor_arg = vec![Type::Reference(Box::new(Type::Const(Box::new(
                    Type::Struct(sname.clone()),
                ))))];
                if let Some(ctor_idx) = self.find_constructor(sname, &copy_ctor_arg) {
                    let size =
                        i32::try_from(self.sema.struct_sizes.get(sname).copied().unwrap_or(4))
                            .map_err(|_| crate::internal_error!(0, 0, "object is too large"))?;
                    let temp_ptr_slot = ctx.alloc_frame(4, 4)?;
                    code.push(OP_I32_CONST);
                    write_i32(code, size);
                    code.push(OP_CALL);
                    write_u32(code, self.func_indices["malloc"]);
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_I32_STORE);
                    write_u32(code, 2);
                    write_u32(code, temp_ptr_slot);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_I32_LOAD);
                    write_u32(code, 2);
                    write_u32(code, temp_ptr_slot);
                    self.compile_lvalue_addr(a, ctx, code)?;
                    code.push(OP_CALL);
                    write_u32(code, ctor_idx);

                    let temp_slot = ctx.alloc_frame(4, 4)?;
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_I32_LOAD);
                    write_u32(code, 2);
                    write_u32(code, temp_ptr_slot);
                    code.push(OP_I32_STORE);
                    write_u32(code, 2);
                    write_u32(code, temp_slot);

                    ctx.temp_objects.push((sname.clone(), temp_slot, false));

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_I32_LOAD);
                    write_u32(code, 2);
                    write_u32(code, temp_ptr_slot);
                    return Ok(());
                }
            }
            self.compile_expr(a, ctx, code)?;
            if let Some(pty) = param_ty {
                self.emit_conversion(&aty, pty, code);
            }
        }

        if is_byval_struct {
            for item in &mut ctx.temp_objects[temp_count_before..] {
                item.2 = false;
            }
        }
        Ok(())
    }
}
