#![cfg(test)]
//! Pack-expansion expression list test.

use super::pack_expansion;
use crate::ast::expr::{Expr, ExprPos};
use crate::ast::types::Type;
use std::collections::HashMap;

#[test]
fn test_pack_expansion_expr_list() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = HashMap::new();
    mapping.insert(
        "args".to_string(),
        pack_expansion::TemplateArg::TypePack(vec![Type::Int, Type::Double]),
    );
    let pattern = Expr::Call {
        callee: Box::new(Expr::Identifier("f".to_string())),
        args: vec![Expr::Identifier("args".to_string())],
        pos: ExprPos::default(),
        dependent: false,
    };
    let exprs = vec![Expr::PackExpansion {
        pattern: Box::new(pattern),
        pos: ExprPos::default(),
    }];
    let expanded =
        pack_expansion::expand_expr_list(&exprs, &mapping).map_err(|e| format!("{e:?}"))?;
    assert_eq!(expanded.len(), 2);
    Ok(())
}
