//! Pack detection helper for template substitution.

use super::pack_expansion::TemplateArg;
use crate::ast::expr::Expr;
use crate::sema::pack_expansion::collect_packs_in_expr;
use std::collections::HashMap;

pub(super) fn has_unexpanded_member_pack(
    exprs: &[Expr],
    mapping: &HashMap<String, TemplateArg>,
) -> bool {
    let known: std::collections::HashSet<String> = mapping
        .iter()
        .filter(|(_, v)| v.is_pack())
        .map(|(k, _)| k.clone())
        .collect();
    exprs.iter().any(|e| {
        if let Expr::PackExpansion { pattern, .. } = e {
            let mut packs = std::collections::HashSet::new();
            collect_packs_in_expr(pattern, &known, &mut packs);
            packs.is_empty()
        } else {
            false
        }
    })
}
