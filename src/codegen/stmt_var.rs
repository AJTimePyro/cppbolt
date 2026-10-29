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
            return Err(self.internal_error("unexpected statement kind"));
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
            let Some(scope) = ctx.scope_objects.last_mut() else {
                return Err(self.internal_error("missing object scope"));
            };
            scope.push((name.to_string(), ty.clone(), offset));

            // Install the vtable before construction. The constructor will
            // install it again for the most-derived class.
            self.emit_vtable_install(sname, offset, ctx, code);

            if let Some(init_expr) = init {
                self.compile_aggregate_local(ty, init_expr, offset, ctx, code)?;
            } else {
                self.compile_struct_default_constructor(sname, offset, ctx, code);
            }
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
            Some(Expr::InitializerList { elements, .. })
            | Some(Expr::TypedInitList { elements, .. }) => Some(elements.as_slice()),
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

    /// Stores the vtable pointer at offset zero of a local object.
    fn emit_vtable_install(
        &self,
        sname: &str,
        offset: u32,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        if let Some(&v_off) = self.vtable_offsets.get(sname) {
            self.emit_local_address(offset, ctx, code);
            code.push(OP_I32_CONST);
            write_i32(code, v_off as i32);
            code.push(OP_I32_STORE);
            write_u32(code, 2);
            write_u32(code, 0);
        }
    }
}
