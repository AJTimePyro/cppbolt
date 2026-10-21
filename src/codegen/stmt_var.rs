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
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected statement kind"
            ));
        };

        if *is_static {
            self.compile_static_local(name, ty, stmt, ctx);
            return Ok(());
        }
        let size = u32::try_from(ty.size_of(&self.sema.struct_sizes))
            .map_err(|_| crate::internal_error!(stmt.line, stmt.col, "local is too large"))?;
        let align = u32::try_from(ty.align_of(&self.sema.struct_aligns))
            .unwrap_or(4)
            .max(4);
        let offset = ctx.alloc_frame(size, align)?;
        ctx.bind_local(name.clone(), offset, ty.clone());

        if ty.is_reference() {
            if let Some(init_expr) = init {
                self.compile_reference_local(init_expr, offset, ctx, code)?;
            }
        } else if let Type::Struct(sname) = ty.strip_const() {
            self.compile_struct_local(
                LocalDecl {
                    struct_name: sname.as_str(),
                    ty,
                    init,
                    offset,
                    name,
                    stmt,
                },
                ctx,
                code,
            )?;
        } else if let Type::Array(elem_ty, count) = ty.strip_const() {
            self.compile_array_local(
                ArrayLocal {
                    elem_ty,
                    count: *count,
                    offset,
                    name,
                    init,
                    stmt,
                },
                ctx,
                code,
            )?;
        } else if let Some(init_expr) = init {
            self.compile_scalar_local(init_expr, ty, offset, ctx, code)?;
        }
        self.emit_clean_temporaries(ctx, code);

        Ok(())
    }

    fn compile_static_local(
        &mut self,
        name: &str,
        ty: &Type,
        stmt: &Stmt,
        ctx: &mut FunctionContext,
    ) {
        let gname =
            crate::name::static_local_name(&ctx.current_func_name, stmt.line, stmt.col, name);
        if let Some(&goff) = self.global_offsets.get(&gname) {
            ctx.bind_static(name.to_string(), goff, ty.clone());
        } else {
            ctx.bind_type(name.to_string(), ty.clone());
        }
    }

    fn compile_reference_local(
        &mut self,
        init_expr: &Expr,
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
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

        Ok(())
    }

    fn compile_struct_local(
        &mut self,
        decl: LocalDecl<'_>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let LocalDecl {
            struct_name,
            ty,
            init,
            offset,
            name,
            stmt,
        } = decl;
        let sname = &struct_name.to_string();
        let Some(scope) = ctx.scope_objects.last_mut() else {
            return Err(CompileError::new(
                "internal compiler error: missing object scope",
                stmt.line,
                stmt.col,
            )
            .with_kind(crate::diagnostics::ErrorKind::Internal));
        };
        scope.push((name.to_string(), ty.clone(), offset));

        // Install the vtable before construction. The constructor will
        // install it again for the most-derived class.
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

        self.compile_struct_init(
            StructInit {
                struct_name: sname,
                ty,
                offset,
            },
            init,
            ctx,
            code,
        )?;

        Ok(())
    }

    fn compile_struct_init(
        &mut self,
        decl: StructInit<'_>,
        init: &Option<Expr>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let StructInit {
            struct_name,
            ty,
            offset,
        } = decl;
        let sname = &struct_name.to_string();
        let Some(init_expr) = init else {
            self.compile_struct_default_constructor(sname, offset, ctx, code);
            return Ok(());
        };
        match init_expr {
            Expr::InitializerList(list) => {
                self.compile_struct_initializer_list(
                    list,
                    init_expr,
                    StructInit {
                        struct_name,
                        ty,
                        offset,
                    },
                    ctx,
                    code,
                )?;
            }
            Expr::Call { callee, args, .. } if matches!(&**callee, Expr::Identifier(cname) if cname == sname) =>
            {
                self.compile_struct_constructor_call(sname, args, offset, ctx, code)?;
            }
            _ => self.compile_struct_value_init(sname, init_expr, offset, ctx, code)?,
        }
        Ok(())
    }

    fn compile_struct_initializer_list(
        &mut self,
        list: &[Expr],
        init_expr: &Expr,
        decl: StructInit<'_>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let StructInit {
            struct_name,
            ty,
            offset,
        } = decl;
        let sname = &struct_name.to_string();
        let arg_types: Vec<Type> = list
            .iter()
            .map(|item| self.get_expr_type(item, ctx))
            .collect();
        if let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) {
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
        } else if self.sema.struct_layouts.contains_key(sname) {
            self.compile_aggregate_local(ty, init_expr, offset, ctx, code)?;
        }
        Ok(())
    }

    fn compile_struct_constructor_call(
        &mut self,
        sname: &str,
        args: &[Expr],
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let arg_types: Vec<Type> = args.iter().map(|a| self.get_expr_type(a, ctx)).collect();
        // A single argument of the same class is a copy initialization: the
        // copy constructor runs for an lvalue, and a prvalue initializes the
        // object directly.
        if let [only] = args
            && matches!(arg_types[0].strip_const(), Type::Struct(other) if other == sname)
        {
            return self.compile_struct_value_init(sname, only, offset, ctx, code);
        }
        if let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_CONST);
            write_i32(code, offset as i32);
            code.push(OP_I32_ADD); // this

            let full_args = self.constructor_arguments(&sig, args);
            for (i, a) in full_args.iter().enumerate() {
                let param_ty = sig.param_types.get(i + 1);
                self.compile_call_arg(a, param_ty, ctx, code)?;
            }
            code.push(OP_CALL);
            write_u32(code, ctor_idx);
        }
        Ok(())
    }

    fn compile_struct_value_init(
        &mut self,
        sname: &str,
        init_expr: &Expr,
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let init_ty = self.get_expr_type(init_expr, ctx);
        if *init_ty.strip_const().strip_reference() == Type::Struct(sname.to_string()) {
            if self.is_struct_lvalue(init_expr, ctx) {
                // Copy construction from lvalue
                let copy_ctor_arg = vec![Type::Reference(Box::new(Type::Const(Box::new(
                    Type::Struct(sname.to_string()),
                ))))];
                if let Some(ctor_idx) = self.find_constructor(sname, &copy_ctor_arg) {
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
                // The prvalue's heap storage is copied into the
                // local, so it can be freed immediately.
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_CALL);
                write_u32(code, self.func_indices["free"]);
                // Drop the temporary from the cleanup list so
                // statement-end RAII does not double-free it.
                if ctx.temp_objects.len() > temp_count_before {
                    ctx.temp_objects.truncate(temp_count_before);
                }
            }
        } else {
            // Converting constructor: sname(init_ty)
            let arg_types = vec![init_ty.clone()];
            if let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_CONST);
                write_i32(code, offset as i32);
                code.push(OP_I32_ADD); // this

                self.compile_call_arg(init_expr, sig.param_types.get(1), ctx, code)?;
                code.push(OP_CALL);
                write_u32(code, ctor_idx);
            } else {
                return Err(CompileError::new(
                    format!(
                        "Cannot initialize struct \x27{}\x27 with value of type {:?}",
                        sname, init_ty
                    ),
                    self.current_line,
                    self.current_col,
                ));
            }
        }
        Ok(())
    }

    fn compile_struct_default_constructor(
        &mut self,
        sname: &str,
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        // No initializer: run the default constructor when one exists.
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

    fn compile_array_local(
        &mut self,
        decl: ArrayLocal<'_>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let ArrayLocal {
            elem_ty,
            count,
            offset,
            name,
            init,
            stmt,
        } = decl;
        let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
            .map_err(|_| crate::internal_error!(stmt.line, stmt.col, "array is too large"))?;
        let init_list = match init {
            Some(Expr::InitializerList(list)) => Some(list.as_slice()),
            _ => None,
        };
        let mut scope_entries = Vec::new();
        for i in 0..count {
            let elem_offset = offset
                .checked_add((i as u32).checked_mul(elem_size).ok_or_else(|| {
                    crate::internal_error!(stmt.line, stmt.col, "array is too large")
                })?)
                .ok_or_else(|| crate::internal_error!(stmt.line, stmt.col, "array is too large"))?;
            if let Type::Struct(sname) = elem_ty.strip_const() {
                scope_entries.push((format!("{}[{}]", name, i), elem_ty.clone(), elem_offset));
                if let Some(item) = init_list.and_then(|items| items.get(i)) {
                    self.compile_aggregate_local(elem_ty, item, elem_offset, ctx, code)?;
                } else if let Some(ci) = self.find_constructor(sname, &[]) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_I32_CONST);
                    write_i32(code, elem_offset as i32);
                    code.push(OP_I32_ADD);
                    code.push(OP_CALL);
                    write_u32(code, ci);
                }
            } else if let Some(item) = init_list.and_then(|items| items.get(i)) {
                self.compile_aggregate_local(elem_ty, item, elem_offset, ctx, code)?;
            }
        }
        if let Some(scope) = ctx.scope_objects.last_mut() {
            scope.extend(scope_entries);
        }

        Ok(())
    }

    fn compile_scalar_local(
        &mut self,
        init_expr: &Expr,
        ty: &Type,
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let init_ty = self.get_expr_type(init_expr, ctx);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        self.compile_expr(init_expr, ctx, code)?;

        self.emit_conversion(&init_ty, ty, code);
        self.emit_store(ty, offset, code);
        Ok(())
    }
}

impl<'a> Codegen<'a> {
    pub(super) fn emit_local_address(
        &self,
        offset: u32,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        if offset != 0 {
            code.push(OP_I32_CONST);
            write_i32(code, offset as i32);
            code.push(OP_I32_ADD);
        }
    }
}
