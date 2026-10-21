//! AST walks used by dead-code elimination, static-local discovery, and
//! address-taken function collection.

use super::*;

fn check_expr_has_new_delete(expr: &Expr) -> bool {
    let mut stack = vec![expr];
    while let Some(expr) = stack.pop() {
        match expr {
            Expr::New { .. } | Expr::Delete { .. } => return true,
            Expr::Binary { left, right, .. } => {
                stack.push(left);
                stack.push(right);
            }
            Expr::Unary { expr, .. } | Expr::Cast { expr, .. } => stack.push(expr),
            Expr::Call { callee, args } => {
                stack.push(callee);
                stack.extend(args.iter());
            }
            Expr::MemberAccess { object, .. } => stack.push(object),
            Expr::ArraySubscript { array, index } => {
                stack.push(array);
                stack.push(index);
            }
            Expr::InitializerList(list) => stack.extend(list.iter()),
            Expr::Assign { target, value } => {
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
            _ => {}
        }
    }
    false
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
            branches,
            else_stmt,
        } => {
            branches.iter().any(|(cond, then_stmt)| {
                check_expr_has_new_delete(cond) || check_stmt_has_new_delete(then_stmt)
            }) || else_stmt
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
        StmtKind::Compound(stmts) => {
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
        _ => {}
    }
}

/// Conservatively records names that may escape as function pointers.
///
/// A bare identifier callee is a named call or a function-pointer variable; it
/// does not by itself take a function's address, so only nested callee
/// expressions are visited.
pub(super) fn collect_address_taken_expr(expr: &Expr, address_taken: &mut HashSet<String>) {
    let mut stack = vec![expr];
    while let Some(expr) = stack.pop() {
        match expr {
            Expr::Call { callee, args } => {
                if !matches!(&**callee, Expr::Identifier(_)) {
                    stack.push(callee);
                }
                stack.extend(args.iter());
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
                    stack.push(expr);
                }
            }
            Expr::Unary { expr, .. } | Expr::Cast { expr, .. } => stack.push(expr),
            Expr::Binary { left, right, .. } => {
                stack.push(left);
                stack.push(right);
            }
            Expr::Assign { target, value } | Expr::CompoundAssign { target, value, .. } => {
                stack.push(target);
                stack.push(value);
            }
            Expr::ArraySubscript { array, index } => {
                stack.push(array);
                stack.push(index);
            }
            Expr::MemberAccess { object, .. } => stack.push(object),
            Expr::Ternary {
                cond,
                then_expr,
                else_expr,
            } => {
                stack.push(cond);
                stack.push(then_expr);
                stack.push(else_expr);
            }
            Expr::New { args, .. } => stack.extend(args.iter()),
            Expr::Delete { expr, .. } => stack.push(expr),
            Expr::InitializerList(list) => stack.extend(list.iter()),
            _ => {}
        }
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
            branches,
            else_stmt,
        } => {
            for (cond, then_stmt) in branches {
                collect_address_taken_expr(cond, address_taken);
                collect_address_taken_stmt(then_stmt, address_taken);
            }
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
    let mut stack = vec![expr];
    while let Some(expr) = stack.pop() {
        collect_call_node(expr, calls, &mut stack);
    }
}

fn collect_call_node<'a>(expr: &'a Expr, calls: &mut HashSet<String>, stack: &mut Vec<&'a Expr>) {
    match expr {
        Expr::Call { callee, args } => collect_call_expr(callee, args, calls, stack),
        Expr::Identifier(name) => {
            calls.insert(name.clone());
        }
        Expr::Unary { op, expr } => {
            stack.push(expr);
            if matches!(op, UnaryOp::Not) {
                calls.insert("operator!".to_string());
                calls.insert("operator bool".to_string());
            } else if matches!(op, UnaryOp::Neg) {
                calls.insert("operator-".to_string());
            } else if matches!(op, UnaryOp::BitNot) {
                calls.insert("operator~".to_string());
            } else if matches!(op, UnaryOp::Plus) {
                calls.insert("operator+".to_string());
            } else if matches!(op, UnaryOp::AddrOf) {
                calls.insert("operator&".to_string());
            } else if matches!(op, UnaryOp::PreInc) {
                calls.insert("operator++".to_string());
            } else if matches!(op, UnaryOp::PreDec) {
                calls.insert("operator--".to_string());
            }
        }
        Expr::Binary { op, left, right } => {
            stack.push(left);
            stack.push(right);
            if matches!(op, BinaryOp::LogicalAnd | BinaryOp::LogicalOr) {
                calls.insert("operator bool".to_string());
            }
            if let Some(op_name) = op.op_name() {
                calls.insert(op_name.to_string());
                calls.insert(format!("std::{}", op_name));
            }
        }
        Expr::Assign { target, value } => {
            stack.push(target);
            stack.push(value);
            calls.insert("operator=".to_string());
        }
        Expr::CompoundAssign { op, target, value } => {
            stack.push(target);
            stack.push(value);
            calls.insert(compound_assign_call_name(op));
        }
        Expr::ArraySubscript { array, index } => {
            calls.insert("operator[]".to_string());
            stack.push(array);
            stack.push(index);
        }
        Expr::MemberAccess { object, member, .. } => {
            calls.insert(member.clone());
            stack.push(object);
        }
        Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => {
            calls.insert("operator bool".to_string());
            stack.push(cond);
            stack.push(then_expr);
            stack.push(else_expr);
        }
        Expr::Cast { expr, .. } => stack.push(expr),
        Expr::New { ty, args, .. } => {
            if let Type::Struct(sname) = ty.strip_const() {
                calls.insert(format!("{}_{}", sname, sname));
                calls.insert(format!("{}::{}", sname, sname));
                calls.insert(format!("{}_~{}", sname, sname));
                calls.insert(format!("{}::~{}", sname, sname));
            }
            stack.extend(args.iter());
        }
        Expr::Delete { expr, .. } => stack.push(expr),
        _ => {}
    }
}

fn compound_assign_call_name(op: &BinaryOp) -> String {
    format!(
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
    )
}

pub(super) fn collect_type_lifecycle(ty: &Type, calls: &mut HashSet<String>) {
    match ty.strip_const() {
        Type::Struct(sname) => {
            calls.insert(format!("{}_{}", sname, sname));
            calls.insert(format!("{}::{}", sname, sname));
            calls.insert(format!("{}_~{}", sname, sname));
            calls.insert(format!("{}::~{}", sname, sname));
            calls.insert(format!("{}_operator=", sname));
            calls.insert(format!("{}::operator=", sname));
        }
        Type::Array(element, _) => collect_type_lifecycle(element, calls),
        _ => {}
    }
}

pub(super) fn collect_stmt_calls(stmt: &Stmt, calls: &mut HashSet<String>) {
    match &stmt.kind {
        StmtKind::Expr(e) => collect_expr_calls(e, calls),
        StmtKind::VarDecl { ty, init, .. } => {
            collect_type_lifecycle(ty, calls);
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
            branches,
            else_stmt,
        } => {
            calls.insert("operator bool".to_string());
            for (cond, then_stmt) in branches {
                collect_expr_calls(cond, calls);
                collect_stmt_calls(then_stmt, calls);
            }
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

/// Records the callee and arguments of a call expression.
fn collect_call_expr<'a>(
    callee: &'a Expr,
    args: &'a [Expr],
    calls: &mut HashSet<String>,
    stack: &mut Vec<&'a Expr>,
) {
    match callee {
        Expr::Identifier(name) => {
            calls.insert(name.clone());
            calls.insert(QualifiedName::new(name).alternate().into_owned());
        }
        Expr::MemberAccess { object, member, .. } => {
            calls.insert(member.clone());
            stack.push(object);
        }
        _ => stack.push(callee),
    }
    stack.extend(args.iter());
}
