//! Helpers for template parsing.


pub(super) fn has_pack_in_expr(expr: &crate::ast::expr::Expr) -> bool {
    use crate::ast::expr::Expr;
    match expr {
        Expr::SizeofPack { .. } | Expr::PackExpansion { .. } | Expr::FoldExpr { .. } => true,
        Expr::Unary { expr: inner, .. } | Expr::Cast { expr: inner, .. } => has_pack_in_expr(inner),
        Expr::Binary { left, right, .. } => has_pack_in_expr(left) || has_pack_in_expr(right),
        _ => false,
    }
}
