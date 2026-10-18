use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_var_decl(
        &mut self,
        stmt: &Stmt,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let StmtKind::VarDecl {
            name,
            ty,
            init,
            is_static,
        } = &stmt.kind
        else {
            return Err(crate::internal_error!(0, 0, "unexpected statement kind"));
        };

        if *is_static {
            let gname = format!("__static_{}", name);
            if let Some(&goff) = self.global_offsets.get(&gname) {
                ctx.static_var_offsets
                    .insert(name.clone(), (goff, ty.clone()));
            }
            ctx.var_types.insert(name.clone(), ty.clone());
            return Ok(());
        }
        let size = ty.size_of(&self.sema.struct_sizes) as u32;
        let offset = ctx.next_local_offset;
        ctx.next_local_offset += (size + 3) & !3;
        ctx.var_offsets.insert(name.clone(), offset);
        ctx.var_types.insert(name.clone(), ty.clone());

        if ty.is_reference() {
            if let Some(init_expr) = init {
                self.compile_lvalue_addr(init_expr, ctx, code)?;
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1);

                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, offset);
            }
        } else if let Type::Struct(sname) = ty.strip_const() {
            let Some(scope) = ctx.scope_objects.last_mut() else {
                return Err(CompileError::new(
                    "internal compiler error: missing object scope",
                    stmt.line,
                    stmt.col,
                )
                .with_kind(crate::diagnostics::ErrorKind::Internal));
            };
            scope.push((name.clone(), ty.clone(), offset));

            // Install vtable if has vtable
            if let Some(&v_off) = self.vtable_offsets.get(sname) {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_CONST);
                write_i32(code, offset as i32);
                code.push(OP_I32_ADD);
                code.push(OP_I32_CONST);
                write_i32(code, v_off as i32);
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, 0);
            }

            if let Some(init_expr) = init {
                match init_expr {
                    Expr::InitializerList(list) => {
                        let arg_types: Vec<Type> = list
                            .iter()
                            .map(|item| self.get_expr_type(item, ctx))
                            .collect();
                        if let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types)
                        {
                            code.push(OP_LOCAL_GET);
                            write_u32(code, ctx.fp_local);
                            code.push(OP_I32_CONST);
                            write_i32(code, offset as i32);
                            code.push(OP_I32_ADD); // this

                            for (i, item) in list.iter().enumerate() {
                                let param_ty = sig.param_types.get(i + 1);
                                self.compile_call_arg(item, param_ty, ctx, code)?;
                            }
                            code.push(OP_CALL);
                            write_u32(code, ctor_idx);
                        } else if let Some(layout) = self.sema.struct_layouts.get(sname) {
                            for (i, item) in list.iter().enumerate() {
                                if let Some(fld) = layout.fields.get(i) {
                                    code.push(OP_LOCAL_GET);
                                    write_u32(code, ctx.fp_local);
                                    self.compile_expr(item, ctx, code)?;
                                    self.emit_store(&fld.ty, offset + fld.offset as u32, code);
                                }
                            }
                        }
                    }
                    Expr::Call { callee, args } if matches!(&**callee, Expr::Identifier(cname) if cname == sname) =>
                    {
                        // Direct constructor call
                        let arg_types: Vec<Type> =
                            args.iter().map(|a| self.get_expr_type(a, ctx)).collect();
                        if let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types)
                        {
                            code.push(OP_LOCAL_GET);
                            write_u32(code, ctx.fp_local);
                            code.push(OP_I32_CONST);
                            write_i32(code, offset as i32);
                            code.push(OP_I32_ADD); // this

                            for (i, a) in args.iter().enumerate() {
                                let param_ty = sig.param_types.get(i + 1);
                                self.compile_call_arg(a, param_ty, ctx, code)?;
                            }
                            code.push(OP_CALL);
                            write_u32(code, ctor_idx);
                        }
                    }
                    _ => {
                        let init_ty = self.get_expr_type(init_expr, ctx);
                        if *init_ty.strip_const().strip_reference() == Type::Struct(sname.clone()) {
                            if self.is_struct_lvalue(init_expr, ctx) {
                                // Copy construction from lvalue
                                let copy_ctor_arg = vec![Type::Reference(Box::new(Type::Const(
                                    Box::new(Type::Struct(sname.clone())),
                                )))];
                                if let Some(ctor_idx) = self.find_constructor(sname, &copy_ctor_arg)
                                {
                                    code.push(OP_LOCAL_GET);
                                    write_u32(code, ctx.fp_local);
                                    code.push(OP_I32_CONST);
                                    write_i32(code, offset as i32);
                                    code.push(OP_I32_ADD); // this

                                    self.compile_lvalue_addr(init_expr, ctx, code)?;
                                    code.push(OP_CALL);
                                    write_u32(code, ctor_idx);
                                } else {
                                    self.compile_lvalue_addr(init_expr, ctx, code)?;
                                    code.push(OP_LOCAL_SET);
                                    write_u32(code, ctx.scratch1);
                                    self.emit_copy_struct(sname, offset, ctx, code);
                                }
                            } else {
                                // Prvalue: copy elision
                                let temp_count_before = ctx.temp_objects.len();
                                self.compile_expr(init_expr, ctx, code)?;
                                code.push(OP_LOCAL_SET);
                                write_u32(code, ctx.scratch1);
                                self.emit_copy_struct(sname, offset, ctx, code);
                                // Free the temporary heap block returned
                                code.push(OP_LOCAL_GET);
                                write_u32(code, ctx.scratch1);
                                code.push(OP_CALL);
                                write_u32(code, self.func_indices["free"]);
                                // Truncate temp_objects so emit_clean_temporaries does not double-free
                                if ctx.temp_objects.len() > temp_count_before {
                                    ctx.temp_objects.truncate(temp_count_before);
                                }
                            }
                        } else {
                            // Converting constructor: sname(init_ty)
                            let arg_types = vec![init_ty.clone()];
                            if let Some(ctor_idx) = self.find_constructor(sname, &arg_types) {
                                code.push(OP_LOCAL_GET);
                                write_u32(code, ctx.fp_local);
                                code.push(OP_I32_CONST);
                                write_i32(code, offset as i32);
                                code.push(OP_I32_ADD); // this

                                self.compile_expr(init_expr, ctx, code)?;
                                code.push(OP_CALL);
                                write_u32(code, ctor_idx);
                            } else {
                                return Err(CompileError::new(
                                    format!(
                                        "Cannot initialize struct \x27{}\x27 with value of type {:?}",
                                        sname, init_ty
                                    ),
                                    1,
                                    1,
                                ));
                            }
                        }
                    }
                }
            } else {
                // Default constructor if available
                if let Some(ctor_idx) = self.find_constructor(sname, &[]) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_I32_CONST);
                    write_i32(code, offset as i32);
                    code.push(OP_I32_ADD); // this
                    code.push(OP_CALL);
                    write_u32(code, ctor_idx);
                }
            }
        } else if let Type::Array(elem_ty, _) = ty.strip_const() {
            if let Some(Expr::InitializerList(list)) = init {
                let elem_size = elem_ty.size_of(&self.sema.struct_sizes) as u32;
                for (i, item) in list.iter().enumerate() {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    self.compile_expr(item, ctx, code)?;
                    self.emit_store(elem_ty, offset + (i as u32) * elem_size, code);
                }
            }
        } else {
            if let Some(init_expr) = init {
                let init_ty = self.get_expr_type(init_expr, ctx);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                self.compile_expr(init_expr, ctx, code)?;

                // Implicit conversions
                if *ty == Type::Double && init_ty.is_integer() {
                    code.push(OP_F64_CONVERT_I32_S);
                } else if *ty == Type::Float && init_ty.is_integer() {
                    code.push(OP_F32_CONVERT_I32_S);
                } else if *ty == Type::Float && init_ty == Type::Double {
                    code.push(OP_F32_DEMOTE_F64);
                } else if *ty == Type::Double && init_ty == Type::Float {
                    code.push(OP_F64_PROMOTE_F32);
                } else if (*ty == Type::LongLong || *ty == Type::UnsignedLongLong)
                    && (init_ty.is_integer()
                        && init_ty != Type::LongLong
                        && init_ty != Type::UnsignedLongLong)
                {
                    code.push(OP_I64_EXTEND_I32_S);
                }
                self.emit_store(ty, offset, code);
            }
        }
        self.emit_clean_temporaries(ctx, code);

        Ok(())
    }
}
