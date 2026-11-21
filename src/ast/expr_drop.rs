//! Iterative drop of expression trees.

use super::Expr;

impl Drop for Expr {
    /// Drops deeply nested expression trees iteratively so a long
    /// left-associative chain cannot overflow the Rust stack while the AST is
    /// being released.
    fn drop(&mut self) {
        let mut stack = Vec::new();
        take_expr_children(self, &mut stack);
        while let Some(mut expr) = stack.pop() {
            take_expr_children(&mut expr, &mut stack);
        }
    }
}

/// Moves one child expression out of `child`, leaving a cheap leaf placeholder.
#[inline]
fn take_box(child: &mut Box<Expr>, out: &mut Vec<Expr>) {
    out.push(std::mem::replace(&mut **child, Expr::BoolLiteral(false)));
}

/// Moves every child expression out of `expr`, leaving cheap leaf
/// placeholders behind so the recursive `Box` drop cannot descend further.
fn take_expr_children(expr: &mut Expr, out: &mut Vec<Expr>) {
    match expr {
        Expr::Binary { left, right, .. } => {
            take_box(left, out);
            take_box(right, out);
        }
        Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::SizeofExpr(expr) => {
            take_box(expr, out);
        }
        Expr::Assign { target, value, .. } | Expr::CompoundAssign { target, value, .. } => {
            take_box(target, out);
            take_box(value, out);
        }
        Expr::Call { callee, args, .. } => {
            take_box(callee, out);
            out.append(&mut std::mem::take(args));
        }
        Expr::ArraySubscript { array, index, .. } => {
            take_box(array, out);
            take_box(index, out);
        }
        Expr::MemberAccess { object, .. } => take_box(object, out),
        Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => {
            take_box(cond, out);
            take_box(then_expr, out);
            take_box(else_expr, out);
        }
        Expr::InitializerList { elements, .. } | Expr::TypedInitList { elements, .. } => {
            out.append(elements);
        }
        Expr::New {
            args,
            array_size,
            placement,
            ..
        } => {
            out.append(&mut std::mem::take(args));
            if let Some(size) = array_size {
                take_box(size, out);
            }
            if let Some(place) = placement {
                take_box(place, out);
            }
        }
        Expr::Delete { expr, .. } => take_box(expr, out),
        Expr::Lambda { params, .. } => {
            for p in params {
                if let Some(def) = p.default_val.take() {
                    out.push(def);
                }
            }
        }
        Expr::PackExpansion { pattern, .. } => take_box(pattern, out),
        Expr::FoldExpr { left, right, .. } => {
            if let Some(l) = left {
                take_box(l, out);
            }
            if let Some(r) = right {
                take_box(r, out);
            }
        }
        Expr::SizeofPack { .. }
        | Expr::IntLiteral(..)
        | Expr::FloatLiteral(..)
        | Expr::BoolLiteral(_)
        | Expr::CharLiteral(_)
        | Expr::StringLiteral(_)
        | Expr::Identifier(_)
        | Expr::This
        | Expr::SizeofType(_)
        | Expr::AlignofType(_) => {}
    }
}
