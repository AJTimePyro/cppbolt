use super::*;

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

    fn bind_parameters(
        &mut self,
        f: &FunctionDecl,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        // Copy parameter locals into the frame; the body addresses locals
        // relative to the frame pointer.
        for (i, p) in f.params.iter().enumerate() {
            let psize = u32::try_from(p.ty.size_of(&self.sema.struct_sizes)).map_err(|_| {
                crate::internal_error!(
                    self.current_line,
                    self.current_col,
                    "parameter is too large"
                )
            })?;
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
                        self.current_line,
                        self.current_col,
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
