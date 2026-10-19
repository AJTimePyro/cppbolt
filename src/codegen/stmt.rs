use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_stmt(
        &mut self,
        stmt: &Stmt,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        match &stmt.kind {
            StmtKind::Compound(stmts) => {
                ctx.enter_scope();
                ctx.scope_objects.push(Vec::new());
                for s in stmts {
                    self.compile_stmt(s, ctx, code)?;
                }
                let objs = ctx.scope_objects.pop().unwrap_or_default();
                self.emit_dtor_for_scope(&objs, ctx, code);
                ctx.exit_scope();
            }
            StmtKind::VarDecl { .. } => self.compile_var_decl(stmt, ctx, code)?,
            StmtKind::Expr(expr) => {
                let ty = self.get_expr_type(expr, ctx);
                self.compile_expr(expr, ctx, code)?;
                if ty != Type::Void {
                    code.push(OP_DROP);
                }
                self.emit_clean_temporaries(ctx, code);
            }
            StmtKind::If {
                cond,
                then_stmt,
                else_stmt,
            } => {
                self.compile_cond(cond, ctx, code)?;
                self.emit_clean_temporaries(ctx, code);
                code.push(OP_IF);
                code.push(0x40);
                self.current_block_depth += 1;
                self.compile_stmt(then_stmt, ctx, code)?;

                if let Some(else_s) = else_stmt {
                    code.push(OP_ELSE);
                    self.compile_stmt(else_s, ctx, code)?;
                }

                code.push(OP_END);
                self.current_block_depth -= 1;
            }
            StmtKind::While { cond, body } => {
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
            }
            StmtKind::DoWhile { body, cond } => {
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
            }
            StmtKind::For { .. } => self.compile_for(stmt, ctx, code)?,
            StmtKind::Switch { .. } => self.compile_switch(stmt, ctx, code)?,
            StmtKind::Break => {
                if let Some(&(break_target_abs, _, loop_scope_depth)) = self.loop_depths.last() {
                    for scope in ctx.scope_objects[loop_scope_depth..].iter().rev() {
                        self.emit_dtor_for_scope(scope, ctx, code);
                    }
                    code.push(OP_BR);
                    write_u32(code, self.current_block_depth - 1 - break_target_abs);
                }
            }
            StmtKind::Continue => {
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
            }
            StmtKind::Return { .. } => self.compile_return(stmt, ctx, code)?,
            _ => {}
        }
        Ok(())
    }
}
