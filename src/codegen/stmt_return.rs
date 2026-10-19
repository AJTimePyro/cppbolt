use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_return(
        &mut self,
        stmt: &Stmt,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let StmtKind::Return(expr_opt) = &stmt.kind else {
            return Err(crate::internal_error!(0, 0, "unexpected statement kind"));
        };

        if let Some(expr) = expr_opt {
            let ty = self.get_expr_type(expr, ctx);
            if ctx.is_reference_return {
                self.compile_lvalue_addr(expr, ctx, code)?;
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.ret_temp_i32);
                self.emit_clean_temporaries(ctx, code);
            } else if let Type::Struct(sname) = ty.strip_const() {
                let size = self.sema.struct_sizes.get(sname).copied().unwrap_or(4);
                if self.is_struct_lvalue(expr, ctx) {
                    code.push(OP_I32_CONST);
                    write_i32(code, size as i32);
                    code.push(OP_CALL);
                    write_u32(code, self.func_indices["malloc"]);
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.ret_temp_i32);

                    self.compile_lvalue_addr(expr, ctx, code)?;
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch1);

                    let copy_ctor_arg = vec![Type::Reference(Box::new(Type::Const(Box::new(
                        Type::Struct(sname.clone()),
                    ))))];
                    if let Some(ctor_idx) = self.find_constructor(sname, &copy_ctor_arg) {
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.ret_temp_i32);
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.scratch1);
                        code.push(OP_CALL);
                        write_u32(code, ctor_idx);
                    } else {
                        for i in 0..size {
                            code.push(OP_LOCAL_GET);
                            write_u32(code, ctx.ret_temp_i32);
                            code.push(OP_LOCAL_GET);
                            write_u32(code, ctx.scratch1);
                            code.push(OP_I32_LOAD8_U);
                            write_u32(code, 0);
                            write_u32(code, i as u32);
                            code.push(OP_I32_STORE8);
                            write_u32(code, 0);
                            write_u32(code, i as u32);
                        }
                    }
                    self.emit_clean_temporaries(ctx, code);
                } else {
                    self.compile_expr(expr, ctx, code)?;
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.ret_temp_i32);
                    if let Some(pos) = ctx.temp_objects.iter().rposition(|(s, _, _)| s == sname) {
                        ctx.temp_objects.remove(pos);
                    }
                    self.emit_clean_temporaries(ctx, code);
                }
            } else {
                self.compile_expr(expr, ctx, code)?;
                self.emit_conversion(&ty, &ctx.return_type, code);
                let return_ty = ctx.return_type.strip_const();
                if matches!(return_ty, Type::LongLong | Type::UnsignedLongLong) {
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.ret_temp_i64);
                } else if return_ty == &Type::Double {
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.ret_temp_f64);
                } else if return_ty == &Type::Float {
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch_f32);
                } else if return_ty != &Type::Void {
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.ret_temp_i32);
                }
                self.emit_clean_temporaries(ctx, code);
            }
        }

        // Epilogue: Run destructors across all active scopes in reverse order
        for scope in ctx.scope_objects.iter().rev() {
            self.emit_dtor_for_scope(scope, ctx, code);
        }

        if ctx.is_main {
            for (sname, goffset) in self.global_dtors.iter().rev() {
                let dtor_name = format!("{}_~{}", sname, sname);
                if let Some(&dtor_idx) = self.func_indices.get(&dtor_name) {
                    code.push(OP_I32_CONST);
                    write_i32(code, *goffset as i32);
                    code.push(OP_CALL);
                    write_u32(code, dtor_idx);
                }
            }
        }

        // If destructor of derived class, call base class destructors in
        // reverse construction order.
        if ctx.is_dtor
            && let Some(ref cls) = ctx.current_class
            && let Some(layout) = self.sema.struct_layouts.get(cls)
        {
            for (base_name, base_offset) in layout.base_offsets.iter().rev() {
                let base_dtor = format!("{}_~{}", base_name, base_name);
                if let Some(&b_idx) = self.func_indices.get(&base_dtor) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, 0); // this
                    if *base_offset != 0 {
                        code.push(OP_I32_CONST);
                        write_i32(code, *base_offset as i32);
                        code.push(OP_I32_ADD);
                    }
                    code.push(OP_CALL);
                    write_u32(code, b_idx);
                }
            }
        }

        // Epilogue: restore stack pointer
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        ctx.emit_frame_size(code);
        code.push(OP_I32_ADD);
        code.push(OP_GLOBAL_SET);
        write_u32(code, 0);

        if expr_opt.is_some() {
            let ty = ctx.return_type.strip_const();
            if ctx.is_reference_return {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.ret_temp_i32);
            } else if matches!(ty, Type::LongLong | Type::UnsignedLongLong) {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.ret_temp_i64);
            } else if ty == &Type::Double {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.ret_temp_f64);
            } else if ty == &Type::Float {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch_f32);
            } else if ty != &Type::Void {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.ret_temp_i32);
            }
        }

        code.push(OP_RETURN);

        Ok(())
    }
}
