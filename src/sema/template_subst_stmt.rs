//! Statement-level template AST substitution.

use super::template_subst::{substitute_expr, substitute_type};
use crate::ast::stmt::{Stmt, StmtKind};
use crate::ast::types::Type;
use std::collections::HashMap;

pub fn substitute_stmt(
    stmt: &mut Stmt,
    mapping: &HashMap<String, Type>,
    injected_class: Option<(&str, &Type)>,
) {
    match &mut stmt.kind {
        StmtKind::Expr(expr) | StmtKind::Case(expr, _) => {
            substitute_expr(expr, mapping, injected_class);
        }
        StmtKind::Compound(stmts) | StmtKind::VarDeclGroup(stmts) => {
            for s in stmts {
                substitute_stmt(s, mapping, injected_class);
            }
        }
        StmtKind::If {
            branches,
            else_stmt,
        } => {
            for (cond, then_stmt) in branches {
                substitute_expr(cond, mapping, injected_class);
                substitute_stmt(then_stmt, mapping, injected_class);
            }
            if let Some(else_s) = else_stmt {
                substitute_stmt(else_s, mapping, injected_class);
            }
        }
        StmtKind::While { .. }
        | StmtKind::DoWhile { .. }
        | StmtKind::RangeFor { .. }
        | StmtKind::For { .. } => {
            substitute_loop_stmt(&mut stmt.kind, mapping, injected_class);
        }
        StmtKind::StructuredBinding { ty, init, .. } => {
            *ty = substitute_type(ty, mapping, injected_class);
            substitute_expr(init, mapping, injected_class);
        }
        StmtKind::Switch { expr, body } => {
            substitute_expr(expr, mapping, injected_class);
            substitute_stmt(body, mapping, injected_class);
        }
        StmtKind::Return(Some(expr)) => substitute_expr(expr, mapping, injected_class),
        StmtKind::VarDecl { ty, init, .. } => {
            *ty = substitute_type(ty, mapping, injected_class);
            if let Some(init_e) = init {
                substitute_expr(init_e, mapping, injected_class);
            }
        }
        StmtKind::Return(None)
        | StmtKind::Break
        | StmtKind::Continue
        | StmtKind::Default
        | StmtKind::Empty
        | StmtKind::UsingNamespace(_)
        | StmtKind::UsingDecl(_)
        | StmtKind::NamespaceAlias { .. } => {}
    }
}

fn substitute_loop_stmt(
    kind: &mut StmtKind,
    mapping: &HashMap<String, Type>,
    injected_class: Option<(&str, &Type)>,
) {
    match kind {
        StmtKind::While { cond, body } => {
            substitute_expr(cond, mapping, injected_class);
            substitute_stmt(body, mapping, injected_class);
        }
        StmtKind::DoWhile { body, cond } => {
            substitute_stmt(body, mapping, injected_class);
            substitute_expr(cond, mapping, injected_class);
        }
        StmtKind::RangeFor { decl, range, body } => {
            substitute_stmt(decl, mapping, injected_class);
            substitute_expr(range, mapping, injected_class);
            substitute_stmt(body, mapping, injected_class);
        }
        StmtKind::For {
            init,
            cond,
            step,
            body,
        } => {
            if let Some(init_s) = init {
                substitute_stmt(init_s, mapping, injected_class);
            }
            if let Some(cond_e) = cond {
                substitute_expr(cond_e, mapping, injected_class);
            }
            if let Some(step_e) = step {
                substitute_expr(step_e, mapping, injected_class);
            }
            substitute_stmt(body, mapping, injected_class);
        }
        _ => {}
    }
}
