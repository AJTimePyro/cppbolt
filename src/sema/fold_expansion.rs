//! C++17 fold expression expansion and template substitution.

use std::collections::HashSet;
use std::collections::HashMap;

use crate::ast::fold_op::FoldOp;
use crate::ast::types::Type;
use crate::ast::{BinaryOp, Expr, ExprPos};
use crate::diagnostics::CompileError;
use crate::sema::pack_expansion::{
    build_slice_mapping, find_unexpanded_packs, rename_pack_identifiers, validate_pack_lengths,
    TemplateArg,
};

enum FoldShape<'a> {
    Unary {
        pack_expr: &'a Expr,
        packs: HashSet<String>,
        is_left: bool,
    },
    Binary {
        pack_expr: &'a Expr,
        packs: HashSet<String>,
        init: &'a Expr,
        is_left: bool,
    },
}

fn classify_fold<'a>(
    left: Option<&'a Expr>,
    right: Option<&'a Expr>,
    known: &HashSet<String>,
    pos: ExprPos,
) -> Result<FoldShape<'a>, CompileError> {
    match (left, right) {
        (None, Some(r)) => {
            let packs: HashSet<String> = find_unexpanded_packs(r, known).into_iter().collect();
            if packs.is_empty() {
                return Err(CompileError::new(
                    "pack expansion does not contain any unexpanded parameter packs",
                    pos.line,
                    pos.col,
                ));
            }
            Ok(FoldShape::Unary {
                pack_expr: r,
                packs,
                is_left: true,
            })
        }
        (Some(l), None) => {
            let packs: HashSet<String> = find_unexpanded_packs(l, known).into_iter().collect();
            if packs.is_empty() {
                return Err(CompileError::new(
                    "pack expansion does not contain any unexpanded parameter packs",
                    pos.line,
                    pos.col,
                ));
            }
            Ok(FoldShape::Unary {
                pack_expr: l,
                packs,
                is_left: false,
            })
        }
        (Some(l), Some(r)) => {
            let l_packs: HashSet<String> = find_unexpanded_packs(l, known).into_iter().collect();
            let r_packs: HashSet<String> = find_unexpanded_packs(r, known).into_iter().collect();
            if !l_packs.is_empty() && !r_packs.is_empty() {
                return Err(CompileError::new(
                    "binary fold expression has unexpanded parameter packs in both operands",
                    pos.line,
                    pos.col,
                ));
            }
            if l_packs.is_empty() && r_packs.is_empty() {
                return Err(CompileError::new(
                    "pack expansion does not contain any unexpanded parameter packs",
                    pos.line,
                    pos.col,
                ));
            }
            if !r_packs.is_empty() {
                Ok(FoldShape::Binary {
                    pack_expr: r,
                    packs: r_packs,
                    init: l,
                    is_left: true,
                })
            } else {
                Ok(FoldShape::Binary {
                    pack_expr: l,
                    packs: l_packs,
                    init: r,
                    is_left: false,
                })
            }
        }
        (None, None) => Err(CompileError::new(
            "pack expansion does not contain any unexpanded parameter packs",
            pos.line,
            pos.col,
        )),
    }
}

fn empty_unary_fold(op: FoldOp, pos: ExprPos) -> Result<Expr, CompileError> {
    match op {
        FoldOp::Binary(BinaryOp::LogicalAnd) => Ok(Expr::BoolLiteral(true)),
        FoldOp::Binary(BinaryOp::LogicalOr) => Ok(Expr::BoolLiteral(false)),
        FoldOp::Binary(BinaryOp::Comma) => Ok(Expr::IntLiteral(0, Type::Void)),
        _ => Err(CompileError::new(
            format!(
                "unary fold expression has empty expansion for operator '{}' with no fallback value",
                op.as_str()
            ),
            pos.line,
            pos.col,
        )),
    }
}

fn build_fold_op_expr(op: FoldOp, left: Expr, right: Expr, pos: ExprPos) -> Result<Expr, CompileError> {
    match op {
        FoldOp::Binary(b_op) => Ok(Expr::Binary {
            op: b_op,
            left: Box::new(left),
            right: Box::new(right),
            pos,
        }),
        FoldOp::Assign => Ok(Expr::Assign {
            target: Box::new(left),
            value: Box::new(right),
            pos,
        }),
        FoldOp::Compound(b_op) => Ok(Expr::CompoundAssign {
            op: b_op,
            target: Box::new(left),
            value: Box::new(right),
            pos,
        }),
        FoldOp::DotStar | FoldOp::ArrowStar => Err(CompileError::new(
            format!("pointer-to-member fold operator '{}' is not supported yet", op.as_str()),
            pos.line,
            pos.col,
        )),
    }
}

fn slice_pack_expr(
    pack_expr: &Expr,
    packs: &HashSet<String>,
    mapping: &HashMap<String, TemplateArg>,
    len: usize,
) -> Result<Vec<Expr>, CompileError> {
    let mut elems = Vec::with_capacity(len);
    for i in 0..len {
        let subst_map = build_slice_mapping(mapping, packs, i);
        let mut copy = pack_expr.clone();
        rename_pack_identifiers(&mut copy, packs, mapping, i);
        crate::sema::template_subst::substitute_expr(&mut copy, &subst_map, None)?;
        elems.push(copy);
    }
    Ok(elems)
}

/// Expands a fold expression with substituted template arguments.
pub fn expand_fold_expr(
    op: FoldOp,
    left: Option<&Expr>,
    right: Option<&Expr>,
    mapping: &HashMap<String, TemplateArg>,
    pos: ExprPos,
) -> Result<Expr, CompileError> {
    let known: HashSet<String> = mapping
        .iter()
        .filter(|(_, v)| v.is_pack())
        .map(|(k, _)| k.clone())
        .collect();
    let shape = classify_fold(left, right, &known, pos)?;
    match shape {
        FoldShape::Unary { pack_expr, packs, is_left } => {
            let len = validate_pack_lengths(&packs, mapping, pos)?;
            if len == 0 {
                return empty_unary_fold(op, pos);
            }
            let mut elems = slice_pack_expr(pack_expr, &packs, mapping, len)?;
            if is_left {
                let mut acc = elems.remove(0);
                for elem in elems {
                    acc = build_fold_op_expr(op, acc, elem, pos)?;
                }
                Ok(acc)
            } else {
                let mut acc = match elems.pop() {
                    Some(e) => e,
                    None => return empty_unary_fold(op, pos),
                };
                while let Some(elem) = elems.pop() {
                    acc = build_fold_op_expr(op, elem, acc, pos)?;
                }
                Ok(acc)
            }
        }
        FoldShape::Binary { pack_expr, packs, init, is_left } => {
            let len = validate_pack_lengths(&packs, mapping, pos)?;
            let mut init_subst = init.clone();
            crate::sema::template_subst::substitute_expr(&mut init_subst, mapping, None)?;
            let mut elems = slice_pack_expr(pack_expr, &packs, mapping, len)?;
            let mut acc = init_subst;
            if is_left {
                for elem in elems {
                    acc = build_fold_op_expr(op, acc, elem, pos)?;
                }
            } else {
                while let Some(elem) = elems.pop() {
                    acc = build_fold_op_expr(op, elem, acc, pos)?;
                }
            }
            Ok(acc)
        }
    }
}
