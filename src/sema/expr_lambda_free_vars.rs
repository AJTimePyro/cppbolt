//! Free variable collector for implicit lambda capture analysis.

use crate::ast::decl::Param;
use crate::ast::expr::{Expr, LambdaCapture};
use crate::ast::stmt::{Stmt, StmtKind};
use std::collections::HashSet;

pub struct FreeVarCollector<'a> {
    pub params: &'a [Param],
    pub explicit: &'a HashSet<String>,
    pub discovered: &'a mut Vec<String>,
}

impl<'a> FreeVarCollector<'a> {
    pub fn visit_stmt(&mut self, stmt: &Stmt, locals: &mut HashSet<String>) {
        match &stmt.kind {
            StmtKind::VarDecl { name, init, .. } => {
                if let Some(e) = init {
                    self.visit_expr(e, locals);
                }
                locals.insert(name.clone());
            }
            StmtKind::VarDeclGroup(stmts) => {
                for s in stmts {
                    self.visit_stmt(s, locals);
                }
            }
            StmtKind::Compound(stmts) => {
                let prev = locals.clone();
                for s in stmts {
                    self.visit_stmt(s, locals);
                }
                *locals = prev;
            }
            StmtKind::Expr(e) | StmtKind::Return(Some(e)) => self.visit_expr(e, locals),
            StmtKind::If {
                branches,
                else_stmt,
            } => {
                for (cond, branch) in branches {
                    self.visit_expr(cond, locals);
                    self.visit_stmt(branch, locals);
                }
                if let Some(eb) = else_stmt {
                    self.visit_stmt(eb, locals);
                }
            }
            StmtKind::While { cond, body } | StmtKind::DoWhile { cond, body } => {
                self.visit_expr(cond, locals);
                self.visit_stmt(body, locals);
            }
            StmtKind::For {
                init,
                cond,
                step,
                body,
            } => {
                let prev = locals.clone();
                if let Some(s) = init {
                    self.visit_stmt(s, locals);
                }
                if let Some(e) = cond {
                    self.visit_expr(e, locals);
                }
                if let Some(e) = step {
                    self.visit_expr(e, locals);
                }
                self.visit_stmt(body, locals);
                *locals = prev;
            }
            StmtKind::RangeFor { decl, range, body } => {
                self.visit_expr(range, locals);
                let prev = locals.clone();
                self.visit_stmt(decl, locals);
                self.visit_stmt(body, locals);
                *locals = prev;
            }
            StmtKind::StructuredBinding { bindings, init, .. } => {
                self.visit_expr(init, locals);
                for b in bindings {
                    locals.insert(b.clone());
                }
            }
            StmtKind::Switch { expr, body } => {
                self.visit_expr(expr, locals);
                self.visit_stmt(body, locals);
            }
            StmtKind::Case(e, _) => self.visit_expr(e, locals),
            _ => {}
        }
    }

    pub fn visit_expr(&mut self, expr: &Expr, locals: &HashSet<String>) {
        match expr {
            Expr::Identifier(name) => {
                if !self.params.iter().any(|p| &p.name == name)
                    && !self.explicit.contains(name)
                    && !locals.contains(name)
                    && !self.discovered.contains(name)
                {
                    self.discovered.push(name.clone());
                }
            }
            Expr::Binary { left, right, .. } => {
                self.visit_expr(left, locals);
                self.visit_expr(right, locals);
            }
            Expr::Assign { target, value, .. } | Expr::CompoundAssign { target, value, .. } => {
                self.visit_expr(target, locals);
                self.visit_expr(value, locals);
            }
            Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::Delete { expr, .. } => {
                self.visit_expr(expr, locals);
            }
            Expr::MemberAccess { object, .. } => self.visit_expr(object, locals),
            Expr::ArraySubscript { array, index, .. } => {
                self.visit_expr(array, locals);
                self.visit_expr(index, locals);
            }
            Expr::Call { callee, args, .. } => {
                self.visit_expr(callee, locals);
                for a in args {
                    self.visit_expr(a, locals);
                }
            }
            Expr::Ternary {
                cond,
                then_expr,
                else_expr,
                ..
            } => {
                self.visit_expr(cond, locals);
                self.visit_expr(then_expr, locals);
                self.visit_expr(else_expr, locals);
            }
            Expr::InitializerList { elements: list, .. }
            | Expr::TypedInitList { elements: list, .. } => {
                for item in list {
                    self.visit_expr(item, locals);
                }
            }
            Expr::New {
                args,
                array_size,
                placement,
                ..
            } => {
                for a in args {
                    self.visit_expr(a, locals);
                }
                if let Some(size) = array_size {
                    self.visit_expr(size, locals);
                }
                if let Some(place) = placement {
                    self.visit_expr(place, locals);
                }
            }
            Expr::Lambda {
                captures,
                params,
                body,
                ..
            } => self.visit_lambda_expr(captures, params, body, locals),
            _ => {}
        }
    }

    fn visit_lambda_expr(
        &mut self,
        captures: &[LambdaCapture],
        params: &[Param],
        body: &Stmt,
        locals: &HashSet<String>,
    ) {
        for c in captures {
            if !locals.contains(&c.name)
                && !self.params.iter().any(|p| p.name == c.name)
                && !self.explicit.contains(&c.name)
                && !self.discovered.contains(&c.name)
            {
                self.discovered.push(c.name.clone());
            }
        }
        let mut inner_explicit = self.explicit.clone();
        for c in captures {
            inner_explicit.insert(c.name.clone());
        }
        let mut inner_collector = FreeVarCollector {
            params,
            explicit: &inner_explicit,
            discovered: self.discovered,
        };
        let mut inner_locals = locals.clone();
        inner_collector.visit_stmt(body, &mut inner_locals);
    }
}
