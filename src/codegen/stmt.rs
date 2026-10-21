use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_stmt(
        &mut self,
        stmt: &Stmt,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let _guard = self.depth.enter(stmt.line, stmt.col)?;
        match &stmt.kind {
            StmtKind::Compound(stmts) => self.compile_compound_stmt(stmts, ctx, code)?,
            StmtKind::VarDecl { .. } => self.compile_var_decl(stmt, ctx, code)?,
            StmtKind::Expr(expr) => self.compile_expr_stmt(expr, ctx, code)?,
            StmtKind::If {
                branches,
                else_stmt,
            } => self.compile_if_stmt(branches, else_stmt, ctx, code)?,
            StmtKind::While { cond, body } => self.compile_while_stmt(cond, body, ctx, code)?,
            StmtKind::DoWhile { body, cond } => {
                self.compile_do_while_stmt(body, cond, ctx, code)?
            }
            StmtKind::For { .. } => self.compile_for(stmt, ctx, code)?,
            StmtKind::Switch { .. } => self.compile_switch(stmt, ctx, code)?,
            StmtKind::Break => self.compile_break_stmt(ctx, code)?,
            StmtKind::Continue => self.compile_continue_stmt(ctx, code)?,
            StmtKind::Return { .. } => self.compile_return(stmt, ctx, code)?,
            _ => {}
        }
        Ok(())
    }

    fn compile_compound_stmt(
        &mut self,
        stmts: &[Stmt],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        ctx.enter_scope();
        ctx.scope_objects.push(Vec::new());
        for s in stmts {
            self.compile_stmt(s, ctx, code)?;
        }
        let objs = ctx.scope_objects.pop().unwrap_or_default();
        self.emit_dtor_for_scope(&objs, ctx, code);
        ctx.exit_scope();
        Ok(())
    }

    fn compile_expr_stmt(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let ty = self.get_expr_type(expr, ctx);
        self.compile_expr(expr, ctx, code)?;
        if ty != Type::Void {
            code.push(OP_DROP);
        }
        self.emit_clean_temporaries(ctx, code);
        Ok(())
    }

    fn compile_if_stmt(
        &mut self,
        branches: &[(Expr, Box<Stmt>)],
        else_stmt: &Option<Box<Stmt>>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        // Emit the chain as nested `if` blocks without recursing once
        // per branch, matching the byte-for-byte output of the former
        // nested `if / else / if` representation.
        for (index, (cond, then_stmt)) in branches.iter().enumerate() {
            self.compile_cond(cond, ctx, code)?;
            self.emit_clean_temporaries(ctx, code);
            code.push(OP_IF);
            code.push(0x40);
            self.current_block_depth += 1;
            self.compile_stmt(then_stmt, ctx, code)?;

            if index + 1 < branches.len() || else_stmt.is_some() {
                code.push(OP_ELSE);
            }
        }

        if let Some(else_s) = else_stmt {
            self.compile_stmt(else_s, ctx, code)?;
        }

        for _ in branches {
            code.push(OP_END);
            self.current_block_depth -= 1;
        }
        Ok(())
    }

    fn compile_while_stmt(
        &mut self,
        cond: &Expr,
        body: &Stmt,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        code.push(OP_BLOCK);
        code.push(0x40); // break target
        let break_target_abs = self.current_block_depth;
        self.current_block_depth += 1;

        code.push(OP_LOOP);
        code.push(0x40); // continue target
        let continue_target_abs = self.current_block_depth;
        self.current_block_depth += 1;

        self.loop_depths.push((
            break_target_abs,
            continue_target_abs,
            ctx.scope_objects.len(),
        ));

        self.compile_cond(cond, ctx, code)?;
        self.emit_clean_temporaries(ctx, code);
        code.push(OP_I32_EQZ);
        code.push(OP_BR_IF);
        write_u32(code, self.current_block_depth - 1 - break_target_abs);

        self.compile_stmt(body, ctx, code)?;

        code.push(OP_BR);
        write_u32(code, self.current_block_depth - 1 - continue_target_abs);

        self.loop_depths.pop();

        code.push(OP_END); // loop end
        self.current_block_depth -= 1;

        code.push(OP_END); // block end
        self.current_block_depth -= 1;
        Ok(())
    }

    fn compile_do_while_stmt(
        &mut self,
        body: &Stmt,
        cond: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        code.push(OP_BLOCK);
        code.push(0x40);
        let break_target_abs = self.current_block_depth;
        self.current_block_depth += 1;

        code.push(OP_LOOP);
        code.push(0x40);
        let loop_target_abs = self.current_block_depth;
        self.current_block_depth += 1;

        code.push(OP_BLOCK);
        code.push(0x40);
        let continue_target_abs = self.current_block_depth;
        self.current_block_depth += 1;

        self.loop_depths.push((
            break_target_abs,
            continue_target_abs,
            ctx.scope_objects.len(),
        ));

        self.compile_stmt(body, ctx, code)?;

        self.loop_depths.pop();

        code.push(OP_END);
        self.current_block_depth -= 1;

        self.compile_cond(cond, ctx, code)?;
        self.emit_clean_temporaries(ctx, code);
        code.push(OP_BR_IF);
        write_u32(code, self.current_block_depth - 1 - loop_target_abs);

        code.push(OP_END);
        self.current_block_depth -= 1;

        code.push(OP_END);
        self.current_block_depth -= 1;
        Ok(())
    }

    fn compile_break_stmt(
        &mut self,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Some(&(break_target_abs, _, loop_scope_depth)) = self.loop_depths.last() {
            for scope in ctx.scope_objects[loop_scope_depth..].iter().rev() {
                self.emit_dtor_for_scope(scope, ctx, code);
            }
            code.push(OP_BR);
            write_u32(code, self.current_block_depth - 1 - break_target_abs);
        }
        Ok(())
    }

    fn compile_continue_stmt(
        &mut self,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Some((_, continue_target_abs, loop_scope_depth)) = self
            .loop_depths
            .iter()
            .rev()
            .copied()
            .find(|(_, target, _)| *target != 0)
        {
            for scope in ctx.scope_objects[loop_scope_depth..].iter().rev() {
                self.emit_dtor_for_scope(scope, ctx, code);
            }
            code.push(OP_BR);
            write_u32(code, self.current_block_depth - 1 - continue_target_abs);
        }
        Ok(())
    }
}
