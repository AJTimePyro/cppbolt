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
            let gname =
                crate::name::static_local_name(&ctx.current_func_name, stmt.line, stmt.col, name);
            if let Some(&goff) = self.global_offsets.get(&gname) {
                ctx.bind_static(name.clone(), goff, ty.clone());
            } else {
                ctx.bind_type(name.clone(), ty.clone());
            }
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
                        } else if self.sema.struct_layouts.contains_key(sname) {
                            self.compile_aggregate_local(ty, init_expr, offset, ctx, code)?;
                        }
                    }
                    Expr::Call { callee, args } if matches!(&**callee, Expr::Identifier(cname) if cname == sname) =>
                    {
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
                            if let Some((ctor_idx, sig)) =
                                self.find_constructor_sig(sname, &arg_types)
                            {
                                code.push(OP_LOCAL_GET);
                                write_u32(code, ctx.fp_local);
                                code.push(OP_I32_CONST);
                                write_i32(code, offset as i32);
                                code.push(OP_I32_ADD); // this

                                self.compile_call_arg(
                                    init_expr,
                                    sig.param_types.get(1),
                                    ctx,
                                    code,
                                )?;
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
        } else if let Type::Array(elem_ty, count) = ty.strip_const() {
            let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
                .map_err(|_| crate::internal_error!(stmt.line, stmt.col, "array is too large"))?;
            let init_list = match init {
                Some(Expr::InitializerList(list)) => Some(list.as_slice()),
                _ => None,
            };
            let mut scope_entries = Vec::new();
            for i in 0..*count {
                let elem_offset = offset
                    .checked_add((i as u32).checked_mul(elem_size).ok_or_else(|| {
                        crate::internal_error!(stmt.line, stmt.col, "array is too large")
                    })?)
                    .ok_or_else(|| {
                        crate::internal_error!(stmt.line, stmt.col, "array is too large")
                    })?;
                if let Type::Struct(sname) = elem_ty.strip_const() {
                    scope_entries.push((
                        format!("{}[{}]", name, i),
                        elem_ty.as_ref().clone(),
                        elem_offset,
                    ));
                    if let Some(item) = init_list.and_then(|items| items.get(i)) {
                        self.compile_aggregate_local(
                            elem_ty.as_ref(),
                            item,
                            elem_offset,
                            ctx,
                            code,
                        )?;
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
                    self.compile_aggregate_local(elem_ty.as_ref(), item, elem_offset, ctx, code)?;
                }
            }
            if let Some(scope) = ctx.scope_objects.last_mut() {
                scope.extend(scope_entries);
            }
        } else {
            if let Some(init_expr) = init {
                let init_ty = self.get_expr_type(init_expr, ctx);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                self.compile_expr(init_expr, ctx, code)?;

                self.emit_conversion(&init_ty, ty, code);
                self.emit_store(ty, offset, code);
            }
        }
        self.emit_clean_temporaries(ctx, code);

        Ok(())
    }
}

impl<'a> Codegen<'a> {
    fn compile_aggregate_local(
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
            if let Expr::Call { callee, args } = init
                && matches!(&**callee, Expr::Identifier(name) if name == sname)
            {
                let arg_types: Vec<Type> = args
                    .iter()
                    .map(|arg| self.get_expr_type(arg, ctx))
                    .collect();
                let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) else {
                    return Err(CompileError::new(
                        format!("No matching constructor for '{}'", sname),
                        1,
                        1,
                    ));
                };
                self.emit_local_address(offset, ctx, code);
                for (i, arg) in args.iter().enumerate() {
                    self.compile_call_arg(arg, sig.param_types.get(i + 1), ctx, code)?;
                }
                code.push(OP_CALL);
                write_u32(code, ctor_idx);
                return Ok(());
            }

            if let Expr::InitializerList(list) = init {
                let arg_types: Vec<Type> = list
                    .iter()
                    .map(|arg| self.get_expr_type(arg, ctx))
                    .collect();
                if let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) {
                    self.emit_local_address(offset, ctx, code);
                    for (i, arg) in list.iter().enumerate() {
                        self.compile_call_arg(arg, sig.param_types.get(i + 1), ctx, code)?;
                    }
                    code.push(OP_CALL);
                    write_u32(code, ctor_idx);
                    return Ok(());
                }
                if let Some(layout) = self.sema.struct_layouts.get(sname).cloned() {
                    for (i, item) in list.iter().enumerate() {
                        if let Some(fld) = layout.fields.get(i) {
                            let field_offset =
                                offset.checked_add(fld.offset as u32).ok_or_else(|| {
                                    crate::internal_error!(1, 1, "object is too large")
                                })?;
                            self.compile_aggregate_local(&fld.ty, item, field_offset, ctx, code)?;
                        }
                    }
                }
                return Ok(());
            }

            let init_ty = self.get_expr_type(init, ctx);
            if matches!(
                init_ty.strip_const().strip_reference(),
                Type::Struct(other) if other == sname
            ) {
                let copy_ctor_arg = vec![Type::Reference(Box::new(Type::Const(Box::new(
                    Type::Struct(sname.clone()),
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
            return Err(CompileError::new(
                format!(
                    "Cannot initialize struct '{}' with value of type {:?}",
                    sname, init_ty
                ),
                1,
                1,
            ));
        }

        if let (Type::Array(elem_ty, _), Expr::InitializerList(list)) = (ty.strip_const(), init) {
            let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
                .map_err(|_| crate::internal_error!(1, 1, "array is too large"))?;
            for (i, item) in list.iter().enumerate() {
                let item_offset = offset
                    .checked_add(
                        (i as u32)
                            .checked_mul(elem_size)
                            .ok_or_else(|| crate::internal_error!(1, 1, "array is too large"))?,
                    )
                    .ok_or_else(|| crate::internal_error!(1, 1, "array is too large"))?;
                self.compile_aggregate_local(elem_ty, item, item_offset, ctx, code)?;
            }
            return Ok(());
        }

        let item_ty = self.get_expr_type(init, ctx);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        self.compile_expr(init, ctx, code)?;
        self.emit_conversion(&item_ty, ty, code);
        self.emit_store(ty, offset, code);
        Ok(())
    }

    fn emit_local_address(&self, offset: u32, ctx: &FunctionContext, code: &mut Vec<u8>) {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        if offset != 0 {
            code.push(OP_I32_CONST);
            write_i32(code, offset as i32);
            code.push(OP_I32_ADD);
        }
    }
}
