use super::*;

impl<'a> Codegen<'a> {
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
        let ty = self.get_expr_type(expr, ctx);
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
        match expr {
            Expr::Call { callee, args } => {
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
                args.iter().any(|a| self.check_expr_has_indirect_call(a))
            }
            Expr::Binary { left, right, .. } => {
                self.check_expr_has_indirect_call(left) || self.check_expr_has_indirect_call(right)
            }
            Expr::Unary { expr, .. } => self.check_expr_has_indirect_call(expr),
            Expr::MemberAccess { object, .. } => self.check_expr_has_indirect_call(object),
            Expr::ArraySubscript { array, index } => {
                self.check_expr_has_indirect_call(array) || self.check_expr_has_indirect_call(index)
            }
            Expr::InitializerList(list) => {
                list.iter().any(|a| self.check_expr_has_indirect_call(a))
            }
            Expr::Assign { target, value } => {
                self.check_expr_has_indirect_call(target)
                    || self.check_expr_has_indirect_call(value)
            }
            Expr::CompoundAssign { target, value, .. } => {
                self.check_expr_has_indirect_call(target)
                    || self.check_expr_has_indirect_call(value)
            }
            Expr::Ternary {
                cond,
                then_expr,
                else_expr,
            } => {
                self.check_expr_has_indirect_call(cond)
                    || self.check_expr_has_indirect_call(then_expr)
                    || self.check_expr_has_indirect_call(else_expr)
            }
            Expr::Cast { expr, .. } => self.check_expr_has_indirect_call(expr),
            Expr::New {
                args,
                array_size,
                placement,
                ..
            } => {
                args.iter()
                    .any(|arg| self.check_expr_has_indirect_call(arg))
                    || array_size
                        .as_ref()
                        .map(|size| self.check_expr_has_indirect_call(size))
                        .unwrap_or(false)
                    || placement
                        .as_ref()
                        .map(|place| self.check_expr_has_indirect_call(place))
                        .unwrap_or(false)
            }
            Expr::Delete { expr, .. } => self.check_expr_has_indirect_call(expr),
            _ => false,
        }
    }

    pub(super) fn check_stmt_has_indirect_call(&self, stmt: &Stmt) -> bool {
        match &stmt.kind {
            StmtKind::Expr(e) => self.check_expr_has_indirect_call(e),
            StmtKind::VarDecl { init, .. } => init
                .as_ref()
                .map(|e| self.check_expr_has_indirect_call(e))
                .unwrap_or(false),
            StmtKind::Compound(stmts) => stmts.iter().any(|s| self.check_stmt_has_indirect_call(s)),
            StmtKind::If {
                cond,
                then_stmt,
                else_stmt,
            } => {
                self.check_expr_has_indirect_call(cond)
                    || self.check_stmt_has_indirect_call(then_stmt)
                    || else_stmt
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
