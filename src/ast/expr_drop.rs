//! Iterative drop of expression trees.

use super::Expr;

impl Drop for Expr {
    /// Drops deeply nested expression trees iteratively so a long
    /// left-associative chain cannot overflow the Rust stack while the AST is
    /// being released.
    fn drop(&mut self) {
        if !has_children(self) {
            return;
        }
        let mut stack = Vec::new();
        take_expr_children(self, &mut stack);
        while let Some(mut expr) = stack.pop() {
            take_expr_children(&mut expr, &mut stack);
            let _ = std::mem::replace(&mut expr, Expr::BoolLiteral(false));
        }
    }
}

/// Returns true when `expr` can contain child expressions that need
/// iterative dropping.
#[inline]
fn has_children(expr: &Expr) -> bool {
    match expr {
        Expr::Binary { .. }
        | Expr::Unary { .. }
        | Expr::Cast { .. }
        | Expr::SizeofExpr(..)
        | Expr::Assign { .. }
        | Expr::CompoundAssign { .. }
        | Expr::ArraySubscript { .. }
        | Expr::MemberAccess { .. }
        | Expr::Ternary { .. }
        | Expr::Delete { .. }
        | Expr::PackExpansion { .. } => true,
        Expr::Call { .. } => true,
        Expr::InitializerList { elements, .. } | Expr::TypedInitList { elements, .. } => {
            !elements.is_empty()
        }
        Expr::New {
            args,
            array_size,
            placement,
            ..
        } => !args.is_empty() || array_size.is_some() || placement.is_some(),
        Expr::Lambda { params, .. } => params.iter().any(|p| p.default_val.is_some()),
        Expr::FoldExpr { left, right, .. } => left.is_some() || right.is_some(),
        Expr::SizeofPack { .. }
        | Expr::IntLiteral(..)
        | Expr::FloatLiteral(..)
        | Expr::BoolLiteral(_)
        | Expr::CharLiteral(_)
        | Expr::StringLiteral(_)
        | Expr::Identifier(_)
        | Expr::This
        | Expr::SizeofType(_)
        | Expr::AlignofType(_) => false,
    }
}

/// Moves one child expression out of `child`, leaving a cheap leaf placeholder.
#[inline]
fn take_box(child: &mut Box<Expr>, out: &mut Vec<Expr>) {
    let expr = std::mem::replace(&mut **child, Expr::BoolLiteral(false));
    if has_children(&expr) {
        out.push(expr);
    }
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
            out.extend(args.drain(..).filter(has_children));
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
            out.extend(elements.drain(..).filter(has_children));
        }
        Expr::New {
            args,
            array_size,
            placement,
            ..
        } => {
            out.extend(args.drain(..).filter(has_children));
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
                if let Some(def) = p.default_val.take()
                    && has_children(&def)
                {
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
