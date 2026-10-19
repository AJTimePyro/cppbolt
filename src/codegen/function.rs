use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_function(
        &mut self,
        f: &FunctionDecl,
        body: &Stmt,
        class_name: Option<String>,
    ) -> Result<FuncBody, CompileError> {
        let mut code = Vec::new();

        let num_params = f.params.len() as u32;
        let fp_local = num_params;
        let scratch1 = num_params + 1;
        let scratch2 = num_params + 2;
        let ret_temp_i32 = num_params + 3;
        let scratch_i64 = num_params + 4;
        let ret_temp_i64 = num_params + 5;
        let scratch_f64 = num_params + 6;
        let ret_temp_f64 = num_params + 7;
        let scratch_f32 = num_params + 8;

        let locals = vec![
            (4, TYPE_I32), // fp_local, scratch1, scratch2, ret_temp_i32
            (2, TYPE_I64), // scratch_i64, ret_temp_i64
            (2, TYPE_F64), // scratch_f64, ret_temp_f64
            (1, TYPE_F32), // scratch_f32
        ];

        let is_dtor = if let Some(ref cls) = class_name {
            f.name == format!("~{}", cls)
        } else {
            false
        };

        let mut ctx = FunctionContext {
            fp_local,
            scratch1,
            scratch2,
            scratch_i64,
            scratch_f64,
            scratch_f32,
            ret_temp_i32,
            ret_temp_i64,
            ret_temp_f64,
            var_offsets: HashMap::new(),
            var_types: HashMap::new(),
            static_var_offsets: HashMap::new(),
            next_local_offset: 16,
            frame_size: 0,
            frame_size_patches: Vec::new(),
            var_scopes: vec![Vec::new()],
            scope_objects: vec![Vec::new()],
            temp_objects: Vec::new(),
            current_class: class_name.clone(),
            current_func_name: f.name.clone(),
            return_type: f.return_type.clone(),
            is_dtor,
            is_reference_return: f.return_type.is_reference(),
            is_main: f.name == "main",
        };

        // Function Prologue: allocate frame from stack pointer. The frame size is
        // patched once the body has determined how much scratch space it needs.
        code.push(OP_GLOBAL_GET);
        write_u32(&mut code, 0); // stack pointer
        ctx.emit_frame_size(&mut code);
        code.push(OP_I32_SUB);
        code.push(OP_LOCAL_TEE);
        write_u32(&mut code, fp_local);
        code.push(OP_GLOBAL_SET);
        write_u32(&mut code, 0);

        // Stack overflow guard: if fp_local < 65536 -> unreachable trap
        code.push(OP_LOCAL_GET);
        write_u32(&mut code, fp_local);
        code.push(OP_I32_CONST);
        write_i32(&mut code, memory_layout::LOCAL_FRAME_GUARD);
        code.push(OP_I32_LT_S);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_UNREACHABLE);
        code.push(OP_END);

        if ctx.is_main {
            // Global constructors before main
            let inits = self.global_inits.clone();
            for (gname, gty, init_opt) in inits {
                if let Some(&goffset) = self.global_offsets.get(&gname) {
                    if let Type::Struct(sname) = gty.strip_const() {
                        self.global_dtors.push((sname.clone(), goffset));
                        let arg_types = Vec::new();
                        if let Some(ctor_idx) = self.find_constructor(sname, &arg_types) {
                            code.push(OP_I32_CONST);
                            write_i32(&mut code, goffset as i32);
                            code.push(OP_CALL);
                            write_u32(&mut code, ctor_idx);
                        } else if let Some(Expr::InitializerList(list)) = init_opt
                            && let Some(layout) = self.sema.struct_layouts.get(sname)
                        {
                            for (i, item) in list.iter().enumerate() {
                                if let Some(fld) = layout.fields.get(i) {
                                    let item_ty = self.get_expr_type(item, &ctx);
                                    code.push(OP_I32_CONST);
                                    write_i32(&mut code, (goffset + fld.offset as u32) as i32);
                                    self.compile_expr(item, &mut ctx, &mut code)?;
                                    self.emit_conversion(&item_ty, &fld.ty, &mut code);
                                    self.emit_store(&fld.ty, 0, &mut code);
                                }
                            }
                        }
                    } else if let Type::Array(elem_ty, _) = gty.strip_const() {
                        if let Some(Expr::InitializerList(list)) = init_opt {
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
                                                write_i32(
                                                    &mut code,
                                                    item_offset + fld.offset as i32,
                                                );
                                                self.compile_expr(sub_item, &mut ctx, &mut code)?;
                                                self.emit_store(&fld.ty, 0, &mut code);
                                            }
                                        }
                                    }
                                } else {
                                    let item_ty = self.get_expr_type(item, &ctx);
                                    code.push(OP_I32_CONST);
                                    write_i32(&mut code, item_offset);
                                    self.compile_expr(item, &mut ctx, &mut code)?;
                                    self.emit_conversion(&item_ty, elem_ty, &mut code);
                                    self.emit_store(elem_ty, 0, &mut code);
                                }
                            }
                        }
                    } else if let Some(init_expr) = init_opt {
                        code.push(OP_I32_CONST);
                        write_i32(&mut code, goffset as i32);
                        let init_ty = self.get_expr_type(&init_expr, &ctx);
                        self.compile_expr(&init_expr, &mut ctx, &mut code)?;
                        self.emit_conversion(&init_ty, &gty, &mut code);
                        self.emit_store(&gty, 0, &mut code);
                    }
                }
            }
        }

        // Copy incoming parameters into stack slots
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
                write_u32(&mut code, i as u32);
                code.push(OP_LOCAL_SET);
                write_u32(&mut code, scratch1);
                self.emit_copy_struct(sname, offset, &ctx, &mut code);
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
                write_u32(&mut code, fp_local);
                code.push(OP_LOCAL_GET);
                write_u32(&mut code, i as u32);
                self.emit_store(&p.ty, offset, &mut code);
            }
        }

        // If constructor or destructor, set up vtable pointer and chain bases
        if let Some(ref cls) = class_name {
            let short_cls = cls.split("::").last().unwrap_or(cls);
            if f.name == *cls || f.name == short_cls {
                // Constructor:
                // 1. If base class exists, call base ctor
                if let Some(layout) = self.sema.struct_layouts.get(cls) {
                    for (base_name, base_offset) in &layout.base_offsets {
                        let base_args = f
                            .ctor_initializers
                            .iter()
                            .find(|(name, _)| name == base_name)
                            .map(|(_, args)| args.as_slice())
                            .unwrap_or(&[]);
                        let arg_types: Vec<Type> = base_args
                            .iter()
                            .map(|a| self.get_expr_type(a, &ctx))
                            .collect();
                        if let Some((b_idx, sig)) = self.find_constructor_sig(base_name, &arg_types)
                        {
                            code.push(OP_LOCAL_GET);
                            write_u32(&mut code, 0); // this
                            if *base_offset != 0 {
                                code.push(OP_I32_CONST);
                                write_i32(&mut code, *base_offset as i32);
                                code.push(OP_I32_ADD);
                            }
                            for (i, a) in base_args.iter().enumerate() {
                                let param_ty = sig.param_types.get(i + 1);
                                self.compile_call_arg(a, param_ty, &mut ctx, &mut code)?;
                            }
                            code.push(OP_CALL);
                            write_u32(&mut code, b_idx);
                        }
                    }
                    // 2. Set this class vtable
                    if let Some(&v_off) = self.vtable_offsets.get(cls) {
                        code.push(OP_LOCAL_GET);
                        write_u32(&mut code, 0); // this
                        code.push(OP_I32_CONST);
                        write_i32(&mut code, v_off as i32);
                        code.push(OP_I32_STORE);
                        write_u32(&mut code, 2);
                        write_u32(&mut code, 0);
                    }
                    // 3. Member initializers in declaration order. Base
                    // subobjects are initialized by their own constructors.
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
                                .map(|args| {
                                    args.iter().map(|a| self.get_expr_type(a, &ctx)).collect()
                                })
                                .unwrap_or_default();
                            if let Some((ctor_idx, sig)) =
                                self.find_constructor_sig(sname, &arg_types)
                            {
                                code.push(OP_LOCAL_GET);
                                write_u32(&mut code, 0); // this
                                code.push(OP_I32_CONST);
                                write_i32(&mut code, fld.offset as i32);
                                code.push(OP_I32_ADD); // addr of member
                                if let Some(args) = init_args {
                                    for (i, a) in args.iter().enumerate() {
                                        let param_ty = sig.param_types.get(i + 1);
                                        self.compile_call_arg(a, param_ty, &mut ctx, &mut code)?;
                                    }
                                }
                                code.push(OP_CALL);
                                write_u32(&mut code, ctor_idx);
                            }
                        } else if let Type::Array(elem_ty, count) = fld.ty.strip_const()
                            && let Type::Struct(sname) = elem_ty.strip_const()
                            && let Some(ctor_idx) = self.find_constructor(sname, &[])
                        {
                            let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
                                .map_err(|_| {
                                    crate::internal_error!(f.line, f.col, "member is too large")
                                })?;
                            for i in 0..*count {
                                let member_offset = (fld.offset as u32)
                                    .checked_add((i as u32).checked_mul(elem_size).ok_or_else(
                                        || {
                                            crate::internal_error!(
                                                f.line,
                                                f.col,
                                                "member is too large"
                                            )
                                        },
                                    )?)
                                    .ok_or_else(|| {
                                        crate::internal_error!(f.line, f.col, "member is too large")
                                    })?;
                                code.push(OP_LOCAL_GET);
                                write_u32(&mut code, 0); // this
                                code.push(OP_I32_CONST);
                                write_i32(&mut code, member_offset as i32);
                                code.push(OP_I32_ADD);
                                code.push(OP_CALL);
                                write_u32(&mut code, ctor_idx);
                            }
                        } else if let Some(args) = init_args
                            && !args.is_empty()
                        {
                            code.push(OP_LOCAL_GET);
                            write_u32(&mut code, 0); // this
                            let item_ty = self.get_expr_type(&args[0], &ctx);
                            self.compile_expr(&args[0], &mut ctx, &mut code)?;
                            self.emit_conversion(&item_ty, &fld.ty, &mut code);
                            self.emit_store(&fld.ty, fld.offset as u32, &mut code);
                        }
                    }
                }
            } else if f.name == format!("~{}", cls) || f.name == format!("~{}", short_cls) {
                // Destructor: reinstall this class's vtable in case derived overwrote it
                if let Some(&v_off) = self.vtable_offsets.get(cls) {
                    code.push(OP_LOCAL_GET);
                    write_u32(&mut code, 0); // this
                    code.push(OP_I32_CONST);
                    write_i32(&mut code, v_off as i32);
                    code.push(OP_I32_STORE);
                    write_u32(&mut code, 2);
                    write_u32(&mut code, 0);
                }
            }
        }

        self.compile_stmt(body, &mut ctx, &mut code)?;

        // Epilogue root RAII destructors
        let root_objs = ctx.scope_objects.pop().unwrap_or_default();
        self.emit_dtor_for_scope(&root_objs, &ctx, &mut code);

        // Member destructors in reverse declaration order
        if let Some(ref cls) = class_name
            && f.name == format!("~{}", cls)
            && let Some(layout) = self.sema.struct_layouts.get(cls)
        {
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
                        write_u32(&mut code, 0); // this
                        code.push(OP_I32_CONST);
                        write_i32(&mut code, fld.offset as i32);
                        code.push(OP_I32_ADD); // member addr
                        code.push(OP_CALL);
                        write_u32(&mut code, d_idx);
                    }
                } else if let Type::Array(elem_ty, count) = fld.ty.strip_const()
                    && let Type::Struct(sname) = elem_ty.strip_const()
                {
                    let member_dtor = format!("{}_~{}", sname, sname);
                    let Some(&d_idx) = self.func_indices.get(&member_dtor) else {
                        continue;
                    };
                    let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
                        .map_err(|_| {
                            crate::internal_error!(f.line, f.col, "member is too large")
                        })?;
                    for i in (0..*count).rev() {
                        let member_offset = (fld.offset as u32)
                            .checked_add((i as u32).checked_mul(elem_size).ok_or_else(|| {
                                crate::internal_error!(f.line, f.col, "member is too large")
                            })?)
                            .ok_or_else(|| {
                                crate::internal_error!(f.line, f.col, "member is too large")
                            })?;
                        code.push(OP_LOCAL_GET);
                        write_u32(&mut code, 0); // this
                        code.push(OP_I32_CONST);
                        write_i32(&mut code, member_offset as i32);
                        code.push(OP_I32_ADD);
                        code.push(OP_CALL);
                        write_u32(&mut code, d_idx);
                    }
                }
            }
        }

        // If destructor of derived class, call base destructors in reverse
        // construction order.
        if let Some(ref cls) = class_name
            && f.name == format!("~{}", cls)
            && let Some(layout) = self.sema.struct_layouts.get(cls)
        {
            for (base_name, base_offset) in layout.base_offsets.iter().rev() {
                let base_dtor = format!("{}_~{}", base_name, base_name);
                if let Some(&b_idx) = self.func_indices.get(&base_dtor) {
                    code.push(OP_LOCAL_GET);
                    write_u32(&mut code, 0); // this
                    if *base_offset != 0 {
                        code.push(OP_I32_CONST);
                        write_i32(&mut code, *base_offset as i32);
                        code.push(OP_I32_ADD);
                    }
                    code.push(OP_CALL);
                    write_u32(&mut code, b_idx);
                }
            }
        }

        if ctx.is_main {
            for (sname, goffset) in self.global_dtors.iter().rev() {
                let dtor_name = format!("{}_~{}", sname, sname);
                if let Some(&dtor_idx) = self.func_indices.get(&dtor_name) {
                    code.push(OP_I32_CONST);
                    write_i32(&mut code, *goffset as i32);
                    code.push(OP_CALL);
                    write_u32(&mut code, dtor_idx);
                }
            }
        }

        let frame_size = ctx
            .next_local_offset
            .checked_add(15)
            .map(|size| size & !15)
            .ok_or_else(frame_too_large)?;
        ctx.frame_size = i32::try_from(frame_size)
            .map_err(|_| crate::internal_error!(0, 0, "frame too large"))?;

        // Epilogue: restore stack pointer
        code.push(OP_LOCAL_GET);
        write_u32(&mut code, fp_local);
        ctx.emit_frame_size(&mut code);
        code.push(OP_I32_ADD);
        code.push(OP_GLOBAL_SET);
        write_u32(&mut code, 0);

        if f.return_type == Type::Void {
            // No return value
        } else {
            let ty = &f.return_type;
            if *ty == Type::LongLong || *ty == Type::UnsignedLongLong {
                code.push(OP_LOCAL_GET);
                write_u32(&mut code, ret_temp_i64);
            } else if *ty == Type::Double {
                code.push(OP_LOCAL_GET);
                write_u32(&mut code, ret_temp_f64);
            } else if *ty == Type::Float {
                code.push(OP_LOCAL_GET);
                write_u32(&mut code, scratch_f32);
            } else {
                code.push(OP_LOCAL_GET);
                write_u32(&mut code, ret_temp_i32);
            }
        }

        code.push(OP_END);
        ctx.patch_frame_sizes(&mut code)?;
        Ok((locals, code))
    }
}
