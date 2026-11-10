use crate::ast::expr::{BinaryOp, Expr, ExprPos, UnaryOp};
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use std::collections::HashMap;

/// Environment for constant expression evaluation.
#[derive(Clone, Copy, Default)]
pub struct ConstexprEnv<'a> {
    pub struct_sizes: Option<&'a HashMap<String, usize>>,
    pub constants: Option<&'a HashMap<String, i128>>,
}

/// Evaluates an expression as an integral constant expression.
pub fn eval_integral_constexpr(
    expr: &Expr,
    struct_sizes: Option<&HashMap<String, usize>>,
) -> Result<i128, CompileError> {
    let env = ConstexprEnv {
        struct_sizes,
        constants: None,
    };
    eval_integral_constexpr_with_env(expr, env)
}

/// Evaluates an expression as an integral constant expression with non-type template parameters or constants.
pub fn eval_integral_constexpr_with_env(
    expr: &Expr,
    env: ConstexprEnv<'_>,
) -> Result<i128, CompileError> {
    match expr {
        Expr::IntLiteral(val, _) => Ok(*val as i128),
        Expr::BoolLiteral(b) => Ok(if *b { 1 } else { 0 }),
        Expr::CharLiteral(c) => Ok(*c as i128),
        Expr::Identifier(name) => {
            if let Some(val) = env.constants.and_then(|c| c.get(name)) {
                Ok(*val)
            } else {
                let pos = expr.pos();
                Err(CompileError::new(
                    "expression is not an integer constant expression",
                    pos.line,
                    pos.col,
                ))
            }
        }
        Expr::Unary {
            op,
            expr: inner,
            pos,
        } => eval_unary_op(*op, inner, *pos, env),
        Expr::Binary {
            op,
            left,
            right,
            pos,
        } => eval_binary_op(*op, left, right, *pos, env),
        Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => {
            let c = eval_integral_constexpr_with_env(cond, env)?;
            if c != 0 {
                eval_integral_constexpr_with_env(then_expr, env)
            } else {
                eval_integral_constexpr_with_env(else_expr, env)
            }
        }
        Expr::Cast { expr: inner, .. } => eval_integral_constexpr_with_env(inner, env),
        Expr::SizeofType(ty) => {
            let empty = HashMap::new();
            let sizes = env.struct_sizes.unwrap_or(&empty);
            Ok(ty.size_of(sizes) as i128)
        }
        Expr::SizeofExpr(inner) => eval_sizeof_expr(inner, env.struct_sizes),
        _ => {
            let pos = expr.pos();
            Err(CompileError::new(
                "expression is not an integer constant expression",
                pos.line,
                pos.col,
            ))
        }
    }
}

fn eval_unary_op(
    op: UnaryOp,
    inner: &Expr,
    pos: ExprPos,
    env: ConstexprEnv<'_>,
) -> Result<i128, CompileError> {
    let val = eval_integral_constexpr_with_env(inner, env)?;
    match op {
        UnaryOp::Plus => Ok(val),
        UnaryOp::Neg => Ok(-val),
        UnaryOp::Not => Ok(if val == 0 { 1 } else { 0 }),
        UnaryOp::BitNot => Ok(!val),
        _ => Err(CompileError::new(
            "unsupported unary operator in constant expression",
            pos.line,
            pos.col,
        )),
    }
}

fn eval_binary_op(
    op: BinaryOp,
    left: &Expr,
    right: &Expr,
    pos: ExprPos,
    env: ConstexprEnv<'_>,
) -> Result<i128, CompileError> {
    let l = eval_integral_constexpr_with_env(left, env)?;
    let r = eval_integral_constexpr_with_env(right, env)?;
    eval_binary_values(op, l, r, pos)
}

fn check_div_rem(l: i128, r: i128, pos: ExprPos) -> Result<(), CompileError> {
    if r == 0 {
        return Err(CompileError::new(
            "division by zero in constant expression",
            pos.line,
            pos.col,
        ));
    }
    if l == i128::MIN && r == -1 {
        return Err(CompileError::new(
            "integer overflow in constant expression division",
            pos.line,
            pos.col,
        ));
    }
    Ok(())
}

fn check_shift_count(r: i128, pos: ExprPos) -> Result<u32, CompileError> {
    if !(0..128).contains(&r) {
        return Err(CompileError::new(
            "shift count out of range in constant expression",
            pos.line,
            pos.col,
        ));
    }
    Ok(r as u32)
}

fn eval_binary_values(op: BinaryOp, l: i128, r: i128, pos: ExprPos) -> Result<i128, CompileError> {
    match op {
        BinaryOp::Add => Ok(l.wrapping_add(r)),
        BinaryOp::Sub => Ok(l.wrapping_sub(r)),
        BinaryOp::Mul => Ok(l.wrapping_mul(r)),
        BinaryOp::Div => {
            check_div_rem(l, r, pos)?;
            Ok(l / r)
        }
        BinaryOp::Rem => {
            check_div_rem(l, r, pos)?;
            Ok(l % r)
        }
        BinaryOp::BitAnd => Ok(l & r),
        BinaryOp::BitOr => Ok(l | r),
        BinaryOp::BitXor => Ok(l ^ r),
        BinaryOp::Shl => Ok(l.wrapping_shl(check_shift_count(r, pos)?)),
        BinaryOp::Shr => Ok(l.wrapping_shr(check_shift_count(r, pos)?)),
        BinaryOp::LogicalAnd => Ok(if l != 0 && r != 0 { 1 } else { 0 }),
        BinaryOp::LogicalOr => Ok(if l != 0 || r != 0 { 1 } else { 0 }),
        BinaryOp::Eq => Ok(if l == r { 1 } else { 0 }),
        BinaryOp::Ne => Ok(if l != r { 1 } else { 0 }),
        BinaryOp::Lt => Ok(if l < r { 1 } else { 0 }),
        BinaryOp::Le => Ok(if l <= r { 1 } else { 0 }),
        BinaryOp::Gt => Ok(if l > r { 1 } else { 0 }),
        BinaryOp::Ge => Ok(if l >= r { 1 } else { 0 }),
        BinaryOp::Comma => Ok(r),
    }
}

fn eval_sizeof_expr(
    inner: &Expr,
    struct_sizes: Option<&HashMap<String, usize>>,
) -> Result<i128, CompileError> {
    let empty = HashMap::new();
    let sizes = struct_sizes.unwrap_or(&empty);
    match inner {
        Expr::IntLiteral(_, ty) => Ok(ty.size_of(sizes) as i128),
        Expr::FloatLiteral(_, ty) => Ok(ty.size_of(sizes) as i128),
        Expr::BoolLiteral(_) => Ok(Type::Bool.size_of(sizes) as i128),
        Expr::CharLiteral(_) => Ok(Type::Char.size_of(sizes) as i128),
        Expr::StringLiteral(s) => Ok((s.len() + 1) as i128),
        _ => Err(CompileError::new(
            "unsupported expression in sizeof constant evaluation",
            0,
            0,
        )),
    }
}
