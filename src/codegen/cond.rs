use super::*;

impl<'a> Codegen<'a> {
    /// Leaves a truthy `i32` on the stack (not necessarily exactly `1`).
    ///
    /// Uses a user-defined `operator bool` when present and compares floats to
    /// zero so NaN remains truthy, matching C++ semantics.
    pub(super) fn compile_cond(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let ty = self.get_expr_type(expr, ctx);
        if let Type::Struct(sname) = ty.strip_const()
            && let Some((m_idx, _)) = self.find_method(sname, "operator bool", &[])
        {
            self.compile_lvalue_addr(expr, ctx, code)?;
            code.push(OP_CALL);
            write_u32(code, m_idx);
            return Ok(());
        }
        self.compile_expr(expr, ctx, code)?;
        match ty.strip_const() {
            Type::LongLong | Type::UnsignedLongLong => {
                code.push(OP_I64_EQZ);
                code.push(OP_I32_EQZ);
            }
            Type::Float => {
                code.push(OP_F32_CONST);
                code.extend_from_slice(&0.0f32.to_le_bytes());
                code.push(OP_F32_NE);
            }
            Type::Double => {
                code.push(OP_F64_CONST);
                code.extend_from_slice(&0.0f64.to_le_bytes());
                code.push(OP_F64_NE);
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn check_expr_has_indirect_call(&self, expr: &Expr) -> bool {
        let mut stack = vec![expr];
        while let Some(expr) = stack.pop() {
            match expr {
                Expr::Call { callee, args, .. } => {
                    if let Expr::Identifier(name) = &**callee {
                        if !self.sema.functions.contains_key(name)
                            && name != "printf"
                            && name != "std::printf"
                            && !name.ends_with("::printf")
                            && name != "malloc"
                            && name != "free"
                        {
                            return true;
                        }
                    } else {
                        return true;
                    }
                    stack.extend(args.iter());
                }
                Expr::Binary { left, right, .. } => {
                    stack.push(left);
                    stack.push(right);
                }
                Expr::Unary { expr, .. } | Expr::Cast { expr, .. } => stack.push(expr),
                Expr::MemberAccess { object, .. } => stack.push(object),
                Expr::ArraySubscript { array, index, .. } => {
                    stack.push(array);
                    stack.push(index);
                }
                Expr::InitializerList { elements, .. }
                | Expr::TypedInitList { elements, .. } => stack.extend(elements.iter()),
                Expr::Assign { target, value, .. } => {
                    stack.push(target);
                    stack.push(value);
                }
                Expr::CompoundAssign { target, value, .. } => {
                    stack.push(target);
                    stack.push(value);
                }
                Expr::Ternary {
                    cond,
                    then_expr,
                    else_expr,
                } => {
                    stack.push(cond);
                    stack.push(then_expr);
                    stack.push(else_expr);
                }
                Expr::New {
                    args,
                    array_size,
                    placement,
                    ..
                } => {
                    stack.extend(args.iter());
                    if let Some(size) = array_size {
                        stack.push(size);
                    }
                    if let Some(place) = placement {
                        stack.push(place);
                    }
                }
                Expr::Delete { expr, .. } => stack.push(expr),
                _ => {}
            }
        }
        false
    }

    pub(super) fn check_stmt_has_indirect_call(&self, stmt: &Stmt) -> bool {
        match &stmt.kind {
            StmtKind::Expr(e) => self.check_expr_has_indirect_call(e),
            StmtKind::VarDecl { init, .. } => init
                .as_ref()
                .map(|e| self.check_expr_has_indirect_call(e))
                .unwrap_or(false),
            StmtKind::Compound(stmts) | StmtKind::VarDeclGroup(stmts) => {
                stmts.iter().any(|s| self.check_stmt_has_indirect_call(s))
            }
            StmtKind::If {
                branches,
                else_stmt,
            } => {
                branches.iter().any(|(cond, then_stmt)| {
                    self.check_expr_has_indirect_call(cond)
                        || self.check_stmt_has_indirect_call(then_stmt)
                }) || else_stmt
                    .as_ref()
                    .map(|s| self.check_stmt_has_indirect_call(s))
                    .unwrap_or(false)
            }
            StmtKind::While { cond, body } | StmtKind::DoWhile { cond, body } => {
                self.check_expr_has_indirect_call(cond) || self.check_stmt_has_indirect_call(body)
            }
            StmtKind::For {
                init,
                cond,
                step,
                body,
            } => {
                init.as_ref()
                    .map(|s| self.check_stmt_has_indirect_call(s))
                    .unwrap_or(false)
                    || cond
                        .as_ref()
                        .map(|e| self.check_expr_has_indirect_call(e))
                        .unwrap_or(false)
                    || step
                        .as_ref()
                        .map(|e| self.check_expr_has_indirect_call(e))
                        .unwrap_or(false)
                    || self.check_stmt_has_indirect_call(body)
            }
            StmtKind::Switch { expr, body } => {
                self.check_expr_has_indirect_call(expr) || self.check_stmt_has_indirect_call(body)
            }
            StmtKind::Return(e) => e
                .as_ref()
                .map(|e| self.check_expr_has_indirect_call(e))
                .unwrap_or(false),
            _ => false,
        }
    }
}
