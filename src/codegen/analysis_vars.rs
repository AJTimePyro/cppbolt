//! Static-local variable collection during AST analysis.

use crate::ast::expr::Expr;
use crate::ast::stmt::{Stmt, StmtKind};
use crate::ast::types::Type;

pub(super) fn collect_static_vars(
    stmt: &Stmt,
    function: &str,
    out: &mut Vec<(String, Type, Option<Expr>)>,
) {
    match &stmt.kind {
        StmtKind::VarDecl {
            name,
            ty,
            init,
            is_static,
        } => {
            if *is_static {
                let gname = crate::name::static_local_name(function, stmt.line, stmt.col, name);
                out.push((gname, ty.clone(), init.clone()));
            }
        }
        StmtKind::Compound(stmts) | StmtKind::VarDeclGroup(stmts) => {
            for s in stmts {
                collect_static_vars(s, function, out);
            }
        }
        StmtKind::If {
            branches,
            else_stmt,
            ..
        } => {
            for (_, then_stmt) in branches {
                collect_static_vars(then_stmt, function, out);
            }
            if let Some(eb) = else_stmt {
                collect_static_vars(eb, function, out);
            }
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::Switch { body, .. } => {
            collect_static_vars(body, function, out);
        }
        StmtKind::Try {
            try_body,
            catch_body,
            ..
        } => {
            collect_static_vars(try_body, function, out);
            collect_static_vars(catch_body, function, out);
        }
        _ => {}
    }
}
