use super::*;

fn check_expr_has_new_delete(expr: &Expr) -> bool {
    match expr {
        Expr::New { .. } | Expr::Delete { .. } => true,
        Expr::Binary { left, right, .. } => {
            check_expr_has_new_delete(left) || check_expr_has_new_delete(right)
        }
        Expr::Unary { expr, .. } => check_expr_has_new_delete(expr),
        Expr::Call { callee, args } => {
            check_expr_has_new_delete(callee) || args.iter().any(check_expr_has_new_delete)
        }
        Expr::MemberAccess { object, .. } => check_expr_has_new_delete(object),
        Expr::ArraySubscript { array, index } => {
            check_expr_has_new_delete(array) || check_expr_has_new_delete(index)
        }
        Expr::InitializerList(list) => list.iter().any(check_expr_has_new_delete),
        Expr::Assign { target, value } => {
            check_expr_has_new_delete(target) || check_expr_has_new_delete(value)
        }
        Expr::CompoundAssign { target, value, .. } => {
            check_expr_has_new_delete(target) || check_expr_has_new_delete(value)
        }
        Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => {
            check_expr_has_new_delete(cond)
                || check_expr_has_new_delete(then_expr)
                || check_expr_has_new_delete(else_expr)
        }
        _ => false,
    }
}

pub(super) fn check_stmt_has_new_delete(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::Expr(e) => check_expr_has_new_delete(e),
        StmtKind::VarDecl { init, .. } => init
            .as_ref()
            .map(check_expr_has_new_delete)
            .unwrap_or(false),
        StmtKind::Compound(stmts) => stmts.iter().any(check_stmt_has_new_delete),
        StmtKind::If {
            cond,
            then_stmt,
            else_stmt,
        } => {
            check_expr_has_new_delete(cond)
                || check_stmt_has_new_delete(then_stmt)
                || else_stmt
                    .as_ref()
                    .map(|s| check_stmt_has_new_delete(s))
                    .unwrap_or(false)
        }
        StmtKind::While { cond, body } | StmtKind::DoWhile { cond, body } => {
            check_expr_has_new_delete(cond) || check_stmt_has_new_delete(body)
        }
        StmtKind::For {
            init,
            cond,
            step,
            body,
        } => {
            init.as_ref()
                .map(|s| check_stmt_has_new_delete(s))
                .unwrap_or(false)
                || cond
                    .as_ref()
                    .map(check_expr_has_new_delete)
                    .unwrap_or(false)
                || step
                    .as_ref()
                    .map(check_expr_has_new_delete)
                    .unwrap_or(false)
                || check_stmt_has_new_delete(body)
        }
        StmtKind::Switch { expr, body } => {
            check_expr_has_new_delete(expr) || check_stmt_has_new_delete(body)
        }
        StmtKind::Return(e) => e.as_ref().map(check_expr_has_new_delete).unwrap_or(false),
        _ => false,
    }
}

pub(super) fn collect_static_vars(stmt: &Stmt, out: &mut Vec<(String, Type, Option<Expr>)>) {
    match &stmt.kind {
        StmtKind::VarDecl {
            name,
            ty,
            init,
            is_static,
        } => {
            if *is_static {
                let gname = format!("__static_{}", name);
                out.push((gname, ty.clone(), init.clone()));
            }
        }
        StmtKind::Compound(stmts) => {
            for s in stmts {
                collect_static_vars(s, out);
            }
        }
        StmtKind::If {
            then_stmt,
            else_stmt,
            ..
        } => {
            collect_static_vars(then_stmt, out);
            if let Some(eb) = else_stmt {
                collect_static_vars(eb, out);
            }
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::Switch { body, .. } => {
            collect_static_vars(body, out);
        }
        _ => {}
    }
}

pub(super) fn collect_address_taken_expr(expr: &Expr, address_taken: &mut HashSet<String>) {
    match expr {
        Expr::Call { callee, args } => {
            if let Expr::Identifier(_) = &**callee {
                // direct call
            } else {
                collect_address_taken_expr(callee, address_taken);
            }
            for a in args {
                collect_address_taken_expr(a, address_taken);
            }
        }
        Expr::Identifier(name) => {
            address_taken.insert(name.clone());
            address_taken.insert(QualifiedName::new(name).alternate().into_owned());
        }
        Expr::Unary {
            op: UnaryOp::AddrOf,
            expr,
        } => {
            if let Expr::Identifier(name) = &**expr {
                address_taken.insert(name.clone());
            } else {
                collect_address_taken_expr(expr, address_taken);
            }
        }
        Expr::Unary { expr, .. } => collect_address_taken_expr(expr, address_taken),
        Expr::Binary { left, right, .. } => {
            collect_address_taken_expr(left, address_taken);
            collect_address_taken_expr(right, address_taken);
        }
        Expr::Assign { target, value } | Expr::CompoundAssign { target, value, .. } => {
            collect_address_taken_expr(target, address_taken);
            collect_address_taken_expr(value, address_taken);
        }
        Expr::ArraySubscript { array, index } => {
            collect_address_taken_expr(array, address_taken);
            collect_address_taken_expr(index, address_taken);
        }
        Expr::MemberAccess { object, .. } => {
            collect_address_taken_expr(object, address_taken);
        }
        Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => {
            collect_address_taken_expr(cond, address_taken);
            collect_address_taken_expr(then_expr, address_taken);
            collect_address_taken_expr(else_expr, address_taken);
        }
        Expr::Cast { expr, .. } => collect_address_taken_expr(expr, address_taken),
        Expr::New { args, .. } => {
            for a in args {
                collect_address_taken_expr(a, address_taken);
            }
        }
        Expr::Delete { expr, .. } => collect_address_taken_expr(expr, address_taken),
        Expr::InitializerList(list) => {
            for e in list {
                collect_address_taken_expr(e, address_taken);
            }
        }
        _ => {}
    }
}

pub(super) fn collect_address_taken_stmt(stmt: &Stmt, address_taken: &mut HashSet<String>) {
    match &stmt.kind {
        StmtKind::Expr(e) => collect_address_taken_expr(e, address_taken),
        StmtKind::VarDecl { init: Some(e), .. } => {
            collect_address_taken_expr(e, address_taken);
        }
        StmtKind::Compound(stmts) => {
            for s in stmts {
                collect_address_taken_stmt(s, address_taken);
            }
        }
        StmtKind::If {
            cond,
            then_stmt,
            else_stmt,
        } => {
            collect_address_taken_expr(cond, address_taken);
            collect_address_taken_stmt(then_stmt, address_taken);
            if let Some(eb) = else_stmt {
                collect_address_taken_stmt(eb, address_taken);
            }
        }
        StmtKind::While { cond, body } => {
            collect_address_taken_expr(cond, address_taken);
            collect_address_taken_stmt(body, address_taken);
        }
        StmtKind::DoWhile { body, cond } => {
            collect_address_taken_stmt(body, address_taken);
            collect_address_taken_expr(cond, address_taken);
        }
        StmtKind::For {
            init,
            cond,
            step,
            body,
        } => {
            if let Some(s) = init {
                collect_address_taken_stmt(s, address_taken);
            }
            if let Some(e) = cond {
                collect_address_taken_expr(e, address_taken);
            }
            if let Some(e) = step {
                collect_address_taken_expr(e, address_taken);
            }
            collect_address_taken_stmt(body, address_taken);
        }
        StmtKind::Return(Some(e)) => collect_address_taken_expr(e, address_taken),
        StmtKind::Switch { expr, body } => {
            collect_address_taken_expr(expr, address_taken);
            collect_address_taken_stmt(body, address_taken);
        }
        _ => {}
    }
}

fn collect_expr_calls(expr: &Expr, calls: &mut HashSet<String>) {
    match expr {
        Expr::Call { callee, args } => {
            match &**callee {
                Expr::Identifier(name) => {
                    calls.insert(name.clone());
                    calls.insert(QualifiedName::new(name).alternate().into_owned());
                }
                Expr::MemberAccess { object, member, .. } => {
                    calls.insert(member.clone());
                    collect_expr_calls(object, calls);
                }
                _ => collect_expr_calls(callee, calls),
            }
            for a in args {
                collect_expr_calls(a, calls);
            }
        }
        Expr::Identifier(name) => {
            calls.insert(name.clone());
        }
        Expr::Unary { op, expr } => {
            collect_expr_calls(expr, calls);
            if matches!(op, UnaryOp::Not) {
                calls.insert("operator!".to_string());
                calls.insert("operator bool".to_string());
            } else if matches!(op, UnaryOp::Neg) {
                calls.insert("operator-".to_string());
            } else if matches!(op, UnaryOp::PreInc) {
                calls.insert("operator++".to_string());
            } else if matches!(op, UnaryOp::PreDec) {
                calls.insert("operator--".to_string());
            }
        }
        Expr::Binary { op, left, right } => {
            collect_expr_calls(left, calls);
            collect_expr_calls(right, calls);
            if matches!(op, BinaryOp::LogicalAnd | BinaryOp::LogicalOr) {
                calls.insert("operator bool".to_string());
            }
            if let Some(op_name) = op.op_name() {
                calls.insert(op_name.to_string());
                calls.insert(format!("std::{}", op_name));
            }
        }
        Expr::Assign { target, value } => {
            collect_expr_calls(target, calls);
            collect_expr_calls(value, calls);
            calls.insert("operator=".to_string());
        }
        Expr::CompoundAssign { op, target, value } => {
            collect_expr_calls(target, calls);
            collect_expr_calls(value, calls);
            let op_name = format!(
                "operator{}=",
                match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Rem => "%",
                    BinaryOp::BitAnd => "&",
                    BinaryOp::BitOr => "|",
                    BinaryOp::BitXor => "^",
                    BinaryOp::Shl => "<<",
                    BinaryOp::Shr => ">>",
                    _ => "",
                }
            );
            calls.insert(op_name);
        }
        Expr::ArraySubscript { array, index } => {
            calls.insert("operator[]".to_string());
            collect_expr_calls(array, calls);
            collect_expr_calls(index, calls);
        }
        Expr::MemberAccess { object, member, .. } => {
            calls.insert(member.clone());
            collect_expr_calls(object, calls);
        }
        Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => {
            calls.insert("operator bool".to_string());
            collect_expr_calls(cond, calls);
            collect_expr_calls(then_expr, calls);
            collect_expr_calls(else_expr, calls);
        }
        Expr::Cast { expr, .. } => collect_expr_calls(expr, calls),
        Expr::New { ty, args, .. } => {
            if let Type::Struct(sname) = ty.strip_const() {
                calls.insert(format!("{}_{}", sname, sname));
                calls.insert(format!("{}::{}", sname, sname));
                calls.insert(format!("{}_~{}", sname, sname));
                calls.insert(format!("{}::~{}", sname, sname));
            }
            for a in args {
                collect_expr_calls(a, calls);
            }
        }
        Expr::Delete { expr, .. } => {
            collect_expr_calls(expr, calls);
        }
        _ => {}
    }
}

pub(super) fn collect_stmt_calls(stmt: &Stmt, calls: &mut HashSet<String>) {
    match &stmt.kind {
        StmtKind::Expr(e) => collect_expr_calls(e, calls),
        StmtKind::VarDecl { ty, init, .. } => {
            if let Type::Struct(sname) = ty.strip_const() {
                calls.insert(format!("{}_{}", sname, sname));
                calls.insert(format!("{}::{}", sname, sname));
                calls.insert(format!("{}_~{}", sname, sname));
                calls.insert(format!("{}::~{}", sname, sname));
                calls.insert(format!("{}_operator=", sname));
                calls.insert(format!("{}::operator=", sname));
            }
            if let Some(e) = init {
                collect_expr_calls(e, calls);
            }
        }
        StmtKind::Compound(stmts) => {
            for s in stmts {
                collect_stmt_calls(s, calls);
            }
        }
        StmtKind::If {
            cond,
            then_stmt,
            else_stmt,
        } => {
            calls.insert("operator bool".to_string());
            collect_expr_calls(cond, calls);
            collect_stmt_calls(then_stmt, calls);
            if let Some(eb) = else_stmt {
                collect_stmt_calls(eb, calls);
            }
        }
        StmtKind::While { cond, body } => {
            calls.insert("operator bool".to_string());
            collect_expr_calls(cond, calls);
            collect_stmt_calls(body, calls);
        }
        StmtKind::DoWhile { body, cond } => {
            calls.insert("operator bool".to_string());
            collect_stmt_calls(body, calls);
            collect_expr_calls(cond, calls);
        }
        StmtKind::For {
            init,
            cond,
            step,
            body,
        } => {
            if let Some(s) = init {
                collect_stmt_calls(s, calls);
            }
            if let Some(e) = cond {
                calls.insert("operator bool".to_string());
                collect_expr_calls(e, calls);
            }
            if let Some(e) = step {
                collect_expr_calls(e, calls);
            }
            collect_stmt_calls(body, calls);
        }
        StmtKind::Return(Some(e)) => collect_expr_calls(e, calls),
        StmtKind::Switch { expr, body } => {
            collect_expr_calls(expr, calls);
            collect_stmt_calls(body, calls);
        }
        _ => {}
    }
}

pub(super) struct FuncCandidate<'p> {
    pub(super) is_root: bool,
    pub(super) aliases: Vec<String>,
    pub(super) decl: FuncDeclRef<'p>,
    pub(super) calls: HashSet<String>,
}

pub(super) enum FuncDeclRef<'p> {
    Function(&'p FunctionDecl),
    Method(&'p str, &'p FunctionDecl),
}
