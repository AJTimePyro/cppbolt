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

pub(super) fn has_pack_in_expr(expr: &crate::ast::expr::Expr) -> bool {
    use crate::ast::expr::Expr;
    match expr {
        Expr::SizeofPack { .. } | Expr::PackExpansion { .. } | Expr::FoldExpr { .. } => true,
        Expr::Unary { expr: inner, .. } | Expr::Cast { expr: inner, .. } => has_pack_in_expr(inner),
        Expr::Binary { left, right, .. } => has_pack_in_expr(left) || has_pack_in_expr(right),
        _ => false,
    }
}
