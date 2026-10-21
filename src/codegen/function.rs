use super::*;
use crate::sema::StructLayout;

impl<'a> Codegen<'a> {
    pub(super) fn compile_function(
        &mut self,
        f: &FunctionDecl,
        body: &Stmt,
        class_name: Option<String>,
    ) -> Result<FuncBody, CompileError> {
        let mut code = Vec::new();
        let (locals, mut ctx) = Self::function_context(f, class_name.as_deref());

        Self::emit_prologue(&mut ctx, &mut code);

        if ctx.is_main {
            self.compile_global_initializers(&mut ctx, &mut code)?;
        }

        self.bind_parameters(f, &mut ctx, &mut code)?;

        if let Some(ref cls) = class_name {
            self.compile_constructor_prologue(f, cls, &mut ctx, &mut code)?;
        }

        self.compile_stmt(body, &mut ctx, &mut code)?;

        // Destroy locals still live in the body's root scope.
        let root_objs = ctx.scope_objects.pop().unwrap_or_default();
        self.emit_dtor_for_scope(&root_objs, &ctx, &mut code);

        if let Some(ref cls) = class_name {
            self.emit_member_destructors(f, cls, &mut code)?;
            self.emit_base_destructors(f, cls, &mut code);
        }

        if ctx.is_main {
            self.emit_global_destructors(&mut code);
        }

        Self::emit_epilogue(f, &mut ctx, &mut code)?;
        Ok((locals, code))
    }

    fn emit_prologue(ctx: &mut FunctionContext, code: &mut Vec<u8>) {
        // Prologue: reserve the frame below the stack pointer. The size is
        // patched after the body has allocated all of its locals and scratch
        // slots, because both the prologue and epilogue refer to it.
        code.push(OP_GLOBAL_GET);
        write_u32(code, 0); // stack pointer
        ctx.emit_frame_size(code);
        code.push(OP_I32_SUB);
        code.push(OP_LOCAL_TEE);
        write_u32(code, ctx.fp_local);
        code.push(OP_GLOBAL_SET);
        write_u32(code, 0);

        // Keep the reserved low page untouched: a frame that would cross it
        // traps instead of corrupting null-page storage.
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_CONST);
        write_i32(code, memory_layout::LOCAL_FRAME_GUARD);
        code.push(OP_I32_LT_S);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_UNREACHABLE);
        code.push(OP_END);
    }

    fn compile_global_initializers(
        &mut self,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        // Namespace-scope objects are dynamically initialized before
        // main's body. Their destructors are queued and run in reverse.
        let inits = self.global_inits.clone();
        for (gname, gty, init_opt) in inits {
            if let Some(&goffset) = self.global_offsets.get(&gname) {
                if let Type::Struct(sname) = gty.strip_const() {
                    self.global_dtors.push((sname.clone(), goffset));
                    let arg_types = Vec::new();
                    if let Some(ctor_idx) = self.find_constructor(sname, &arg_types) {
                        code.push(OP_I32_CONST);
                        write_i32(code, goffset as i32);
                        code.push(OP_CALL);
                        write_u32(code, ctor_idx);
                    } else if let Some(Expr::InitializerList(list)) = init_opt.as_ref()
                        && let Some(layout) = self.sema.struct_layouts.get(sname)
                    {
                        for (i, item) in list.iter().enumerate() {
                            if let Some(fld) = layout.fields.get(i) {
                                let item_ty = self.get_expr_type(item, ctx);
                                code.push(OP_I32_CONST);
                                write_i32(code, (goffset + fld.offset as u32) as i32);
                                self.compile_expr(item, ctx, code)?;
                                self.emit_conversion(&item_ty, &fld.ty, code);
                                self.emit_store(&fld.ty, 0, code);
                            }
                        }
                    }
                } else if let Type::Array(elem_ty, _) = gty.strip_const() {
                    if let Some(Expr::InitializerList(list)) = init_opt.as_ref() {
                        let elem_size = elem_ty.size_of(&self.sema.struct_sizes) as u32;
                        for (i, item) in list.iter().enumerate() {
                            let item_offset = (goffset + (i as u32) * elem_size) as i32;
                            if let (Type::Struct(sname), Expr::InitializerList(sublist)) =
                                (elem_ty.strip_const(), item)
                            {
                                if let Some(layout) = self.sema.struct_layouts.get(sname) {
                                    for (j, sub_item) in sublist.iter().enumerate() {
                                        if let Some(fld) = layout.fields.get(j) {
                                            code.push(OP_I32_CONST);
                                            write_i32(code, item_offset + fld.offset as i32);
                                            self.compile_expr(sub_item, ctx, code)?;
                                            self.emit_store(&fld.ty, 0, code);
                                        }
                                    }
                                }
                            } else {
                                let item_ty = self.get_expr_type(item, ctx);
                                code.push(OP_I32_CONST);
                                write_i32(code, item_offset);
                                self.compile_expr(item, ctx, code)?;
                                self.emit_conversion(&item_ty, elem_ty, code);
                                self.emit_store(elem_ty, 0, code);
                            }
                        }
                    }
                } else if let Some(init_expr) = init_opt {
                    code.push(OP_I32_CONST);
                    write_i32(code, goffset as i32);
                    let init_ty = self.get_expr_type(&init_expr, ctx);
                    self.compile_expr(&init_expr, ctx, code)?;
                    self.emit_conversion(&init_ty, &gty, code);
                    self.emit_store(&gty, 0, code);
                }
            }
        }
        Ok(())
    }

    fn bind_parameters(
        &mut self,
        f: &FunctionDecl,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        // Copy parameter locals into the frame; the body addresses locals
        // relative to the frame pointer.
        for (i, p) in f.params.iter().enumerate() {
            let psize = u32::try_from(p.ty.size_of(&self.sema.struct_sizes))
                .map_err(|_| crate::internal_error!(0, 0, "parameter is too large"))?;
            let palign = u32::try_from(p.ty.align_of(&self.sema.struct_aligns))
                .unwrap_or(4)
                .max(4);
            let offset = ctx.alloc_frame(psize, palign)?;
            ctx.bind_local(p.name.clone(), offset, p.ty.clone());

            if let Type::Struct(sname) = p.ty.strip_const() {
                code.push(OP_LOCAL_GET);
                write_u32(code, i as u32);
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1);
                self.emit_copy_struct(sname, offset, ctx, code);
                let Some(scope) = ctx.scope_objects.last_mut() else {
                    return Err(CompileError::new(
                        "internal compiler error: missing object scope",
                        1,
                        1,
                    )
                    .with_kind(crate::diagnostics::ErrorKind::Internal));
                };
                scope.push((p.name.clone(), p.ty.clone(), offset));
            } else {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_LOCAL_GET);
                write_u32(code, i as u32);
                self.emit_store(&p.ty, offset, code);
            }
        }
        Ok(())
    }

    fn compile_constructor_prologue(
        &mut self,
        f: &FunctionDecl,
        cls: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let short_cls = cls.split("::").last().unwrap_or(cls);
        if f.name == *cls || f.name == short_cls {
            self.compile_constructor_body(f, cls, ctx, code)?;
        } else if f.name == format!("~{}", cls) || f.name == format!("~{}", short_cls) {
            // Destructor: reinstall this class's vtable in case derived overwrote it
            self.install_class_vtable(cls, code);
        }
        Ok(())
    }

    /// Emits the body of a constructor: base constructor calls, the class
    /// vtable, and the member initializers.
    fn compile_constructor_body(
        &mut self,
        f: &FunctionDecl,
        cls: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        self.compile_base_constructor_calls(f, cls, ctx, code)?;
        self.install_class_vtable(cls, code);
        let Some(layout) = self.sema.struct_layouts.get(cls) else {
            return Ok(());
        };
        self.compile_member_initializers(f, layout, ctx, code)
    }

    /// Installs the class's vtable pointer in the object under construction.
    fn install_class_vtable(&mut self, cls: &str, code: &mut Vec<u8>) {
        if let Some(&v_off) = self.vtable_offsets.get(cls) {
            code.push(OP_LOCAL_GET);
            write_u32(code, 0); // this
            code.push(OP_I32_CONST);
            write_i32(code, v_off as i32);
            code.push(OP_I32_STORE);
            write_u32(code, 2);
            write_u32(code, 0);
        }
    }

    /// Calls the constructor of every base subobject.
    fn compile_base_constructor_calls(
        &mut self,
        f: &FunctionDecl,
        cls: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Some(layout) = self.sema.struct_layouts.get(cls) else {
            return Ok(());
        };
        for (base_name, base_offset) in &layout.base_offsets {
            let base_args = f
                .ctor_initializers
                .iter()
                .find(|(name, _)| name == base_name)
                .map(|(_, args)| args.as_slice())
                .unwrap_or(&[]);
            let arg_types: Vec<Type> = base_args
                .iter()
                .map(|a| self.get_expr_type(a, ctx))
                .collect();
            if let Some((b_idx, sig)) = self.find_constructor_sig(base_name, &arg_types) {
                code.push(OP_LOCAL_GET);
                write_u32(code, 0); // this
                if *base_offset != 0 {
                    code.push(OP_I32_CONST);
                    write_i32(code, *base_offset as i32);
                    code.push(OP_I32_ADD);
                }
                for (i, a) in base_args.iter().enumerate() {
                    let param_ty = sig.param_types.get(i + 1);
                    self.compile_call_arg(a, param_ty, ctx, code)?;
                }
                code.push(OP_CALL);
                write_u32(code, b_idx);
            }
        }
        Ok(())
    }

    /// Emits the member initializers in declaration order. Base subobjects
    /// are initialized by their own constructors.
    fn compile_member_initializers(
        &mut self,
        f: &FunctionDecl,
        layout: &StructLayout,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let base_field_count: usize = layout
            .base_classes
            .iter()
            .filter_map(|b| self.sema.struct_layouts.get(b))
            .map(|bl| bl.fields.len())
            .sum();
        for fld in layout.fields.iter().skip(base_field_count) {
            let init_args = f
                .ctor_initializers
                .iter()
                .find(|(name, _)| name == &fld.name)
                .map(|(_, args)| args.as_slice());
            if let Type::Struct(sname) = fld.ty.strip_const() {
                let arg_types: Vec<Type> = init_args
                    .map(|args| args.iter().map(|a| self.get_expr_type(a, ctx)).collect())
                    .unwrap_or_default();
                if let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, 0); // this
                    code.push(OP_I32_CONST);
                    write_i32(code, fld.offset as i32);
                    code.push(OP_I32_ADD);
                    if let Some(args) = init_args {
                        for (i, a) in args.iter().enumerate() {
                            let param_ty = sig.param_types.get(i + 1);
                            self.compile_call_arg(a, param_ty, ctx, code)?;
                        }
                    }
                    code.push(OP_CALL);
                    write_u32(code, ctor_idx);
                }
            } else if let Type::Array(elem_ty, count) = fld.ty.strip_const()
                && let Type::Struct(sname) = elem_ty.strip_const()
                && let Some(ctor_idx) = self.find_constructor(sname, &[])
            {
                let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
                    .map_err(|_| crate::internal_error!(f.line, f.col, "member is too large"))?;
                for i in 0..*count {
                    let member_offset = (fld.offset as u32)
                        .checked_add((i as u32).checked_mul(elem_size).ok_or_else(|| {
                            crate::internal_error!(f.line, f.col, "member is too large")
                        })?)
                        .ok_or_else(|| {
                            crate::internal_error!(f.line, f.col, "member is too large")
                        })?;
                    code.push(OP_LOCAL_GET);
                    write_u32(code, 0); // this
                    code.push(OP_I32_CONST);
                    write_i32(code, member_offset as i32);
                    code.push(OP_I32_ADD);
                    code.push(OP_CALL);
                    write_u32(code, ctor_idx);
                }
            } else if let Some(args) = init_args
                && !args.is_empty()
            {
                code.push(OP_LOCAL_GET);
                write_u32(code, 0); // this
                let item_ty = self.get_expr_type(&args[0], ctx);
                self.compile_expr(&args[0], ctx, code)?;
                self.emit_conversion(&item_ty, &fld.ty, code);
                self.emit_store(&fld.ty, fld.offset as u32, code);
            }
        }
        Ok(())
    }

    fn emit_member_destructors(
        &mut self,
        f: &FunctionDecl,
        cls: &str,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if f.name != format!("~{}", cls) {
            return Ok(());
        }
        let Some(layout) = self.sema.struct_layouts.get(cls) else {
            return Ok(());
        };
        let base_field_count: usize = layout
            .base_classes
            .iter()
            .filter_map(|b| self.sema.struct_layouts.get(b))
            .map(|bl| bl.fields.len())
            .sum();
        for fld in layout.fields.iter().skip(base_field_count).rev() {
            if let Type::Struct(sname) = fld.ty.strip_const() {
                let member_dtor = format!("{}_~{}", sname, sname);
                if let Some(&d_idx) = self.func_indices.get(&member_dtor) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, 0); // this
                    code.push(OP_I32_CONST);
                    write_i32(code, fld.offset as i32);
                    code.push(OP_I32_ADD);
                    code.push(OP_CALL);
                    write_u32(code, d_idx);
                }
            } else if let Type::Array(elem_ty, count) = fld.ty.strip_const()
                && let Type::Struct(sname) = elem_ty.strip_const()
            {
                let member_dtor = format!("{}_~{}", sname, sname);
                let Some(&d_idx) = self.func_indices.get(&member_dtor) else {
                    continue;
                };
                let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
                    .map_err(|_| crate::internal_error!(f.line, f.col, "member is too large"))?;
                for i in (0..*count).rev() {
                    let member_offset = (fld.offset as u32)
                        .checked_add((i as u32).checked_mul(elem_size).ok_or_else(|| {
                            crate::internal_error!(f.line, f.col, "member is too large")
                        })?)
                        .ok_or_else(|| {
                            crate::internal_error!(f.line, f.col, "member is too large")
                        })?;
                    code.push(OP_LOCAL_GET);
                    write_u32(code, 0); // this
                    code.push(OP_I32_CONST);
                    write_i32(code, member_offset as i32);
                    code.push(OP_I32_ADD);
                    code.push(OP_CALL);
                    write_u32(code, d_idx);
                }
            }
        }
        Ok(())
    }

    fn emit_base_destructors(&self, f: &FunctionDecl, cls: &str, code: &mut Vec<u8>) {
        if f.name != format!("~{}", cls) {
            return;
        }
        let Some(layout) = self.sema.struct_layouts.get(cls) else {
            return;
        };
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

    fn emit_global_destructors(&self, code: &mut Vec<u8>) {
        // Falling out of main destroys namespace-scope objects in reverse
        // construction order.
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

    fn emit_epilogue(
        f: &FunctionDecl,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        // Round the final frame to 16 bytes; all frame-size placeholders are
        // patched after this point.
        let frame_size = ctx
            .next_local_offset
            .checked_add(15)
            .map(|size| size & !15)
            .ok_or_else(frame_too_large)?;
        ctx.frame_size = i32::try_from(frame_size)
            .map_err(|_| crate::internal_error!(0, 0, "frame too large"))?;

        // Epilogue: restore stack pointer
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        ctx.emit_frame_size(code);
        code.push(OP_I32_ADD);
        code.push(OP_GLOBAL_SET);
        write_u32(code, 0);

        if f.return_type != Type::Void {
            let ty = &f.return_type;
            if *ty == Type::LongLong || *ty == Type::UnsignedLongLong {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.ret_temp_i64);
            } else if *ty == Type::Double {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.ret_temp_f64);
            } else if *ty == Type::Float {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch_f32);
            } else {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.ret_temp_i32);
            }
        }

        code.push(OP_END);
        ctx.patch_frame_sizes(code)?;
        Ok(())
    }
}
