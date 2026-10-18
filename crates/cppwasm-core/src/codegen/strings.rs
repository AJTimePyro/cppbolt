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
                cond,
                then_stmt,
                else_stmt,
            } => {
                self.collect_strings_expr(cond);
                self.collect_strings_stmt(then_stmt);
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
                self.collect_strings_expr(left);
                self.collect_strings_expr(right);
            }
            Expr::Unary { expr, .. } => self.collect_strings_expr(expr),
            Expr::Assign { target, value } | Expr::CompoundAssign { target, value, .. } => {
                self.collect_strings_expr(target);
                self.collect_strings_expr(value);
            }
            Expr::Call { callee, args } => {
                self.collect_strings_expr(callee);
                for a in args {
                    self.collect_strings_expr(a);
                }
            }
            Expr::ArraySubscript { array, index } => {
                self.collect_strings_expr(array);
                self.collect_strings_expr(index);
            }
            Expr::MemberAccess { object, .. } => self.collect_strings_expr(object),
            Expr::Ternary {
                cond,
                then_expr,
                else_expr,
            } => {
                self.collect_strings_expr(cond);
                self.collect_strings_expr(then_expr);
                self.collect_strings_expr(else_expr);
            }
            Expr::Cast { expr, .. } => self.collect_strings_expr(expr),
            Expr::InitializerList(list) => {
                for item in list {
                    self.collect_strings_expr(item);
                }
            }
            _ => {}
        }
    }
}
