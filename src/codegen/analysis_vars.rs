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
        StmtKind::Try { try_body, catches } => {
            collect_static_vars(try_body, function, out);
            for c in catches {
                collect_static_vars(&c.body, function, out);
            }
        }
        _ => {}
    }
}

fn expr_contains_throw(expr: &Expr) -> bool {
    let mut stack = vec![expr];
    while let Some(e) = stack.pop() {
        if matches!(e, Expr::Throw(..)) {
            return true;
        }
        match e {
            Expr::Unary { expr: inner, .. }
            | Expr::Cast { expr: inner, .. }
            | Expr::PackExpansion { pattern: inner, .. }
            | Expr::Delete { expr: inner, .. } => stack.push(inner),
            Expr::Binary { left, right, .. }
            | Expr::Assign {
                target: left,
                value: right,
                ..
            }
            | Expr::CompoundAssign {
                target: left,
                value: right,
                ..
            }
            | Expr::ArraySubscript {
                array: left,
                index: right,
                ..
            } => {
                stack.push(left);
                stack.push(right);
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
            Expr::Call { callee, args, .. } => {
                stack.push(callee);
                stack.extend(args.iter());
            }
            Expr::InitializerList { elements, .. } | Expr::TypedInitList { elements, .. } => {
                stack.extend(elements.iter());
            }
            _ => {}
        }
    }
    false
}

pub(super) fn check_stmt_has_try_throw(stmt: &Stmt) -> bool {
    let mut stack = vec![stmt];
    while let Some(s) = stack.pop() {
        match &s.kind {
            StmtKind::Throw(_) | StmtKind::Try { .. } => return true,
            StmtKind::Expr(e)
            | StmtKind::Return(Some(e))
            | StmtKind::VarDecl { init: Some(e), .. } => {
                if expr_contains_throw(e) {
                    return true;
                }
            }
            StmtKind::Compound(stmts) | StmtKind::VarDeclGroup(stmts) => {
                stack.extend(stmts.iter());
            }
            StmtKind::If {
                branches,
                else_stmt,
                ..
            } => {
                for (cond, b) in branches {
                    if expr_contains_throw(cond) {
                        return true;
                    }
                    stack.push(b);
                }
                if let Some(eb) = else_stmt {
                    stack.push(eb);
                }
            }
            StmtKind::While { cond, body } | StmtKind::DoWhile { cond, body } => {
                if expr_contains_throw(cond) {
                    return true;
                }
                stack.push(body);
            }
            StmtKind::For {
                init,
                cond,
                step,
                body,
            } => {
                if let Some(init_s) = init {
                    stack.push(init_s);
                }
                if let Some(cond_e) = cond
                    && expr_contains_throw(cond_e)
                {
                    return true;
                }
                if let Some(step_e) = step
                    && expr_contains_throw(step_e)
                {
                    return true;
                }
                stack.push(body);
            }
            StmtKind::Switch { expr, body } => {
                if expr_contains_throw(expr) {
                    return true;
                }
                stack.push(body);
            }
            _ => {}
        }
    }
    false
}
