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
            return Err(crate::internal_error!(0, 0, "unexpected statement kind"));
        };

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

        Ok(())
    }

    pub(super) fn compile_switch(
        &mut self,
        stmt: &Stmt,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let StmtKind::Switch { expr, body } = &stmt.kind else {
            return Err(crate::internal_error!(0, 0, "unexpected statement kind"));
        };

        self.compile_expr(expr, ctx, code)?;
        self.emit_clean_temporaries(ctx, code);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1); // target value

        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch2);

        code.push(OP_BLOCK);
        code.push(0x40); // break block
        let break_target_abs = self.current_block_depth;
        self.current_block_depth += 1;
        self.loop_depths
            .push((break_target_abs, 0, ctx.scope_objects.len()));

        if let StmtKind::Compound(stmts) = &body.kind {
            for s in stmts {
                match &s.kind {
                    StmtKind::Case(val) => {
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.scratch2);
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.scratch1);
                        code.push(OP_I32_CONST);
                        write_i32(code, *val as i32);
                        code.push(OP_I32_EQ);
                        code.push(OP_I32_OR);
                        code.push(OP_LOCAL_SET);
                        write_u32(code, ctx.scratch2);
                    }
                    StmtKind::Default => {
                        code.push(OP_I32_CONST);
                        write_i32(code, 1);
                        code.push(OP_LOCAL_SET);
                        write_u32(code, ctx.scratch2);
                    }
                    _ => {
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.scratch2);
                        code.push(OP_IF);
                        code.push(0x40);
                        self.current_block_depth += 1;
                        self.compile_stmt(s, ctx, code)?;
                        self.current_block_depth -= 1;
                        code.push(OP_END);
                    }
                }
            }
        }

        self.loop_depths.pop();
        code.push(OP_END);
        self.current_block_depth -= 1;

        Ok(())
    }
}
