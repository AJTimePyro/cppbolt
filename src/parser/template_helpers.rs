//! Helpers for template parsing.

/// Whether a serialized dependent expression names an active template
/// parameter. Such an expression must stay dependent instead of being
/// evaluated eagerly at parse time, where a type parameter has no size yet.
pub(super) fn text_mentions_active(
    text: &str,
    active: &[std::collections::HashSet<String>],
) -> bool {
    text.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .any(|part| !part.is_empty() && active.iter().any(|set| set.contains(part)))
}

/// Whether an expression names an identifier (for example a namespace-scope
/// constexpr constant) that cannot be folded at parse time but may be known to
/// semantic analysis once globals are registered.
pub(super) fn has_identifier_in_expr(expr: &crate::ast::expr::Expr) -> bool {
    use crate::ast::expr::Expr;
    match expr {
        Expr::Identifier(_) => true,
        Expr::Unary { expr: inner, .. }
        | Expr::Cast { expr: inner, .. }
        | Expr::SizeofExpr(inner) => has_identifier_in_expr(inner),
        Expr::Binary { left, right, .. } => {
            has_identifier_in_expr(left) || has_identifier_in_expr(right)
        }
        Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => {
            has_identifier_in_expr(cond)
                || has_identifier_in_expr(then_expr)
                || has_identifier_in_expr(else_expr)
        }
        _ => false,
    }
}

pub(super) fn has_pack_in_expr(expr: &crate::ast::expr::Expr) -> bool {
    use crate::ast::expr::Expr;
    match expr {
        Expr::SizeofPack { .. } | Expr::PackExpansion { .. } | Expr::FoldExpr { .. } => true,
        Expr::Unary { expr: inner, .. } | Expr::Cast { expr: inner, .. } => has_pack_in_expr(inner),
        Expr::Binary { left, right, .. } => has_pack_in_expr(left) || has_pack_in_expr(right),
        _ => false,
    }
}
