use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn collect_strings_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Function(f) => {
                if let Some(body) = &f.body {
                    self.collect_strings_stmt(body);
                }
            }
            Decl::StructDecl { methods, .. } => {
                for m in methods {
                    if let Some(body) = &m.body {
                        self.collect_strings_stmt(body);
                    }
                }
            }
            Decl::GlobalVar { init: Some(e), .. } => {
                self.collect_strings_expr(e);
            }
            _ => {}
        }
    }

    pub(super) fn collect_strings_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Compound(stmts) => {
                for s in stmts {
                    self.collect_strings_stmt(s);
                }
            }
            StmtKind::VarDecl { init: Some(e), .. } => self.collect_strings_expr(e),
            StmtKind::Expr(e) => self.collect_strings_expr(e),
            StmtKind::If {
                branches,
                else_stmt,
            } => {
                for (cond, then_stmt) in branches {
                    self.collect_strings_expr(cond);
                    self.collect_strings_stmt(then_stmt);
                }
                if let Some(el) = else_stmt {
                    self.collect_strings_stmt(el);
                }
            }
            StmtKind::While { cond, body } => {
                self.collect_strings_expr(cond);
                self.collect_strings_stmt(body);
            }
            StmtKind::DoWhile { body, cond } => {
                self.collect_strings_stmt(body);
                self.collect_strings_expr(cond);
            }
            StmtKind::For {
                init,
                cond,
                step,
                body,
            } => {
                if let Some(i) = init {
                    self.collect_strings_stmt(i);
                }
                if let Some(c) = cond {
                    self.collect_strings_expr(c);
                }
                if let Some(s) = step {
                    self.collect_strings_expr(s);
                }
                self.collect_strings_stmt(body);
            }
            StmtKind::Switch { expr, body } => {
                self.collect_strings_expr(expr);
                self.collect_strings_stmt(body);
            }
            StmtKind::Return(Some(e)) => self.collect_strings_expr(e),
            _ => {}
        }
    }

    pub(super) fn collect_strings_expr(&mut self, expr: &Expr) {
        let mut stack = vec![expr];
        while let Some(expr) = stack.pop() {
            let mut children: Vec<&Expr> = Vec::new();
            match expr {
                Expr::StringLiteral(s) => {
                    if !self.string_table.contains_key(s) {
                        let offset = self.next_data_offset;
                        let mut bytes = s.as_bytes().to_vec();
                        bytes.push(0);
                        self.next_data_offset += (bytes.len() as u32 + 3) & !3;
                        self.string_table.insert(s.clone(), offset);
                    }
                }
                Expr::Binary { left, right, .. } => {
                    children.push(left);
                    children.push(right);
                }
                Expr::Unary { expr, .. } => children.push(expr),
                Expr::Assign { target, value } | Expr::CompoundAssign { target, value, .. } => {
                    children.push(target);
                    children.push(value);
                }
                Expr::Call { callee, args } => {
                    children.push(callee);
                    children.extend(args.iter());
                }
                Expr::ArraySubscript { array, index } => {
                    children.push(array);
                    children.push(index);
                }
                Expr::MemberAccess { object, .. } => children.push(object),
                Expr::Ternary {
                    cond,
                    then_expr,
                    else_expr,
                } => {
                    children.push(cond);
                    children.push(then_expr);
                    children.push(else_expr);
                }
                Expr::Cast { expr, .. } => children.push(expr),
                Expr::InitializerList(list) => children.extend(list.iter()),
                _ => {}
            }
            // Preserve the original left-to-right visit order so string data
            // offsets stay stable.
            stack.extend(children.into_iter().rev());
        }
    }
}
