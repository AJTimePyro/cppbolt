use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_function(
        &mut self,
        f: &FunctionDecl,
        body: &Stmt,
        class_name: Option<String>,
    ) -> Result<FuncBody, CompileError> {
        let mut code = Vec::new();
        let max_catch = function_context::max_catch_depth(body, 0);
        let (locals, mut ctx) = Self::function_context(f, class_name.as_deref(), max_catch);
        self.current_scratch1.set(ctx.scratch1);
        self.current_scratch_i64.set(ctx.scratch_i64);

        Self::emit_prologue(&mut ctx, &mut code);

        if ctx.is_main {
            self.compile_global_initializers(&mut ctx, &mut code)?;
        }

        let needs_unwind = class_name.is_some()
            || ctx.is_main
            || f.params.iter().any(|p| matches!(p.ty.strip_const(), Type::Struct(_)))
            || unwind::stmt_can_throw_or_call(body);

        if needs_unwind {
            Self::emit_unwind_enter(&mut code);
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
            self.emit_member_destructors(&f.name, cls, f.line, f.col, &mut code)?;
            self.emit_base_destructors(&ctx, &mut code);
        }

        if ctx.is_main {
            self.emit_global_destructors(&mut code);
        }

        if let Some(ref cls) = class_name {
            let short_cls = QualifiedName::new(cls).class_short_name();
            if f.name == *cls || f.name == short_cls {
                code.push(OP_I64_CONST);
                crate::wasm::write_i64(&mut code, 0);
                code.push(OP_LOCAL_SET);
                write_u32(&mut code, ctx.cleanup_mask_local);
            }
        }

        if needs_unwind {
            self.emit_unwind_exit(&mut ctx, &mut code);
        }

        Self::emit_epilogue(f, &mut ctx, &mut code)?;
        Ok((locals, code))
    }

    fn emit_unwind_enter(code: &mut Vec<u8>) {
        code.push(OP_BLOCK);
        code.push(0x40);
        code.push(OP_BLOCK);
        code.push(TYPE_EXNREF);
        code.push(OP_TRY_TABLE);
        code.push(0x40);
        code.push(0x01);
        code.push(0x03); // catch_all_ref
        write_u32(code, 0);
    }

    fn emit_unwind_exit(&self, ctx: &mut FunctionContext, code: &mut Vec<u8>) {
        code.push(OP_BR);
        write_u32(code, 2);
        code.push(OP_END);
        code.push(OP_UNREACHABLE);
        code.push(OP_END);
        self.emit_function_unwind_handler(ctx, code);
        code.push(OP_END);
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
                .map_err(|_| self.internal_error("parameter is too large"))?;
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
                self.emit_copy_struct(sname, offset, ctx, code)?;
                let clean_idx = if let Some(dtor_idx) = self.lookup_struct_dtor(sname) {
                    let idx = ctx.register_cleanup(sname.to_string(), dtor_idx, offset, false)?;
                    self.emit_mark_cleanup_active(idx, ctx, code);
                    Some(idx)
                } else {
                    None
                };
                let Some(scope) = ctx.scope_objects.last_mut() else {
                    return Err(self.internal_error("missing object scope"));
                };
                scope.push((p.name.clone(), p.ty.clone(), offset, clean_idx));
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
            .ok_or_else(|| frame_too_large(ctx.line, ctx.col))?;
        ctx.frame_size = i32::try_from(frame_size)
            .map_err(|_| crate::internal_error!(ctx.line, ctx.col, "frame too large"))?;

        // Epilogue: restore stack pointer
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        ctx.emit_frame_size(code);
        code.push(OP_I32_ADD);
        code.push(OP_GLOBAL_SET);
        write_u32(code, 0);

        if let Some(local) = super::stmt_return::return_local(&f.return_type, ctx) {
            code.push(OP_LOCAL_GET);
            write_u32(code, local);
        }

        code.push(OP_END);
        ctx.patch_frame_sizes(code)?;
        Ok(())
    }
}
