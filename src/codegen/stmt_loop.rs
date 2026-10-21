use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_for(
        &mut self,
        stmt: &Stmt,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let StmtKind::For {
            init,
            cond,
            step,
            body,
        } = &stmt.kind
        else {
            return Err(self.internal_error("unexpected statement kind"));
        };

        ctx.enter_scope();
        if let Some(i) = init {
            self.compile_stmt(i, ctx, code)?;
        }

        code.push(OP_BLOCK);
        code.push(0x40); // break target
        let break_target_abs = self.current_block_depth;
        self.current_block_depth += 1;

        code.push(OP_LOOP);
        code.push(0x40); // loop body
        let loop_header_abs = self.current_block_depth;
        self.current_block_depth += 1;

        if let Some(c) = cond {
            self.compile_cond(c, ctx, code)?;
            self.emit_clean_temporaries(ctx, code);
            code.push(OP_I32_EQZ);
            code.push(OP_BR_IF);
            write_u32(code, self.current_block_depth - 1 - break_target_abs);
        }

        code.push(OP_BLOCK);
        code.push(0x40); // continue target
        let continue_target_abs = self.current_block_depth;
        self.current_block_depth += 1;

        self.loop_depths.push((
            break_target_abs,
            continue_target_abs,
            ctx.scope_objects.len(),
        ));

        self.compile_stmt(body, ctx, code)?;

        self.loop_depths.pop();

        code.push(OP_END); // end continue target
        self.current_block_depth -= 1;

        if let Some(s) = step {
            let ty = self.get_expr_type(s, ctx);
            self.compile_expr(s, ctx, code)?;
            if ty != Type::Void {
                code.push(OP_DROP);
            }
            self.emit_clean_temporaries(ctx, code);
        }

        code.push(OP_BR);
        write_u32(code, self.current_block_depth - 1 - loop_header_abs);

        code.push(OP_END); // end loop
        self.current_block_depth -= 1;

        code.push(OP_END); // end block
        self.current_block_depth -= 1;

        ctx.exit_scope();
        Ok(())
    }

    pub(super) fn compile_switch(
        &mut self,
        stmt: &Stmt,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let StmtKind::Switch { expr, body } = &stmt.kind else {
            return Err(self.internal_error("unexpected statement kind"));
        };

        let switch_ty = self.get_expr_type(expr, ctx);
        let switch_is_wide = matches!(
            switch_ty.strip_const(),
            Type::LongLong | Type::UnsignedLongLong
        );
        self.compile_expr(expr, ctx, code)?;
        self.emit_clean_temporaries(ctx, code);

        let (target_slot, matched_slot, any_match_slot) =
            self.emit_switch_slots(switch_is_wide, ctx, code)?;

        self.emit_switch_default_scan(
            body,
            SwitchSlots {
                target_slot,
                matched_slot: any_match_slot,
                switch_is_wide,
            },
            ctx,
            code,
        );

        code.push(OP_BLOCK);
        code.push(0x40); // break block
        let break_target_abs = self.current_block_depth;
        self.current_block_depth += 1;
        // `0` means this construct has no continue target.
        self.loop_depths
            .push((break_target_abs, 0, ctx.scope_objects.len()));

        ctx.enter_scope();
        ctx.scope_objects.push(Vec::new());
        if let StmtKind::Compound(stmts) = &body.kind {
            for s in stmts {
                match &s.kind {
                    StmtKind::Case(val) => {
                        self.emit_switch_case(
                            s,
                            *val,
                            SwitchSlots {
                                target_slot,
                                matched_slot,
                                switch_is_wide,
                            },
                            ctx,
                            code,
                        )?;
                    }
                    StmtKind::Default => {
                        self.emit_switch_default(s, matched_slot, any_match_slot, ctx, code)?;
                    }
                    _ => {
                        self.emit_switch_fallthrough(s, matched_slot, ctx, code)?;
                    }
                }
            }
        }

        self.loop_depths.pop();
        if let Some(scope) = ctx.scope_objects.last() {
            self.emit_dtor_for_scope(scope, ctx, code);
        }
        ctx.scope_objects.pop();
        code.push(OP_END);
        self.current_block_depth -= 1;
        ctx.exit_scope();

        Ok(())
    }

    /// Stores the switch value and the match state in frame slots.
    fn emit_switch_slots(
        &mut self,
        switch_is_wide: bool,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(u32, u32, u32), CompileError> {
        // Case expressions and bodies may call functions, which can clobber
        // scratch locals. Keep the switch value and match state in frame slots.
        let target_slot = if switch_is_wide {
            ctx.alloc_frame(8, 8)?
        } else {
            ctx.alloc_frame(4, 4)?
        };
        let matched_slot = ctx.alloc_frame(4, 4)?;
        let any_match_slot = ctx.alloc_frame(4, 4)?;

        if switch_is_wide {
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch_i64);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_i64);
            code.push(OP_I64_STORE);
            write_u32(code, 3);
            write_u32(code, target_slot);
        } else {
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_I32_STORE);
            write_u32(code, 2);
            write_u32(code, target_slot);
        }
        for slot in [matched_slot, any_match_slot] {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_CONST);
            write_i32(code, 0);
            code.push(OP_I32_STORE);
            write_u32(code, 2);
            write_u32(code, slot);
        }

        Ok((target_slot, matched_slot, any_match_slot))
    }

    /// Default starts only when no case label matches. Compute that once
    /// because case bodies may clobber scratch locals.
    fn emit_switch_default_scan(
        &mut self,
        body: &Stmt,
        slots: SwitchSlots,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        let SwitchSlots {
            target_slot,
            matched_slot: any_match_slot,
            switch_is_wide,
        } = slots;
        // Compute that once because case bodies may clobber scratch locals.
        if let StmtKind::Compound(stmts) = &body.kind {
            for s in stmts {
                if let StmtKind::Case(val) = &s.kind {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_I32_LOAD);
                    write_u32(code, 2);
                    write_u32(code, any_match_slot);
                    self.emit_switch_case_eq(target_slot, *val, switch_is_wide, ctx, code);
                    code.push(OP_I32_OR);
                    code.push(OP_I32_STORE);
                    write_u32(code, 2);
                    write_u32(code, any_match_slot);
                }
            }
        }
    }

    fn emit_switch_case(
        &mut self,
        stmt: &Stmt,
        val: i64,
        slots: SwitchSlots,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let SwitchSlots {
            target_slot,
            matched_slot,
            switch_is_wide,
        } = slots;
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, matched_slot);
        self.emit_switch_case_eq(target_slot, val, switch_is_wide, ctx, code);
        code.push(OP_I32_OR);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, matched_slot);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_IF);
        code.push(0x40);
        self.current_block_depth += 1;
        self.compile_stmt(stmt, ctx, code)?;
        self.current_block_depth -= 1;
        code.push(OP_END);
        Ok(())
    }

    fn emit_switch_default(
        &mut self,
        stmt: &Stmt,
        matched_slot: u32,
        any_match_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, any_match_slot);
        code.push(OP_I32_EQZ);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, matched_slot);
        code.push(OP_I32_EQZ);
        code.push(OP_I32_AND);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);

        // Once default starts, following case bodies fall through.
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, matched_slot);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_OR);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, matched_slot);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_IF);
        code.push(0x40);
        self.current_block_depth += 1;
        self.compile_stmt(stmt, ctx, code)?;
        self.current_block_depth -= 1;
        code.push(OP_END);
        Ok(())
    }

    fn emit_switch_fallthrough(
        &mut self,
        stmt: &Stmt,
        matched_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, matched_slot);
        code.push(OP_IF);
        code.push(0x40);
        self.current_block_depth += 1;
        self.compile_stmt(stmt, ctx, code)?;
        self.current_block_depth -= 1;
        code.push(OP_END);
        Ok(())
    }

    /// Pushes `1` when the saved switch value equals `value`, without
    /// re-evaluating the switch expression.
    fn emit_switch_case_eq(
        &self,
        target_slot: u32,
        value: i64,
        wide: bool,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        if wide {
            code.push(OP_I64_LOAD);
            write_u32(code, 3);
            write_u32(code, target_slot);
            code.push(OP_I64_CONST);
            write_i64(code, value);
            code.push(OP_I64_EQ);
        } else {
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, target_slot);
            code.push(OP_I32_CONST);
            write_i32(code, value as i32);
            code.push(OP_I32_EQ);
        }
    }
}
