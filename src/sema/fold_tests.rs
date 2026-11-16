//! Unit tests for C++17 fold expression expansion and semantic diagnostics.

use std::collections::HashMap;

use super::fold_expansion::expand_fold_expr;
use super::pack_expansion::TemplateArg;
use crate::ast::fold_op::FoldOp;
use crate::ast::types::Type;
use crate::ast::{BinaryOp, Expr, ExprPos};

fn int_lit(val: i64) -> Expr {
    Expr::IntLiteral(val, Type::Int)
}

fn id_expr(name: &str) -> Expr {
    Expr::Identifier(name.to_string())
}

fn bin_op(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
        pos: ExprPos::default(),
    }
}

#[test]
fn test_expand_fold_expr_unary_and_binary() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = HashMap::new();
    mapping.insert("args".to_string(), TemplateArg::NonTypePack(vec![1, 2, 3]));
    let unary_left = expand_fold_expr(
        FoldOp::Binary(BinaryOp::Add),
        None,
        Some(&id_expr("args")),
        &mapping,
        ExprPos::default(),
    )?;
    assert_eq!(unary_left, bin_op(BinaryOp::Add, bin_op(BinaryOp::Add, int_lit(1), int_lit(2)), int_lit(3)));

    let mut mapping_bin = HashMap::new();
    mapping_bin.insert("args".to_string(), TemplateArg::NonTypePack(vec![1, 2]));
    let bin_left = expand_fold_expr(
        FoldOp::Binary(BinaryOp::Shl),
        Some(&id_expr("s")),
        Some(&id_expr("args")),
        &mapping_bin,
        ExprPos::default(),
    )?;
    assert_eq!(bin_left, bin_op(BinaryOp::Shl, bin_op(BinaryOp::Shl, id_expr("s"), int_lit(1)), int_lit(2)));

    let mut empty_mapping = HashMap::new();
    empty_mapping.insert("args".to_string(), TemplateArg::NonTypePack(vec![]));
    let empty_and = expand_fold_expr(
        FoldOp::Binary(BinaryOp::LogicalAnd),
        Some(&id_expr("args")),
        None,
        &empty_mapping,
        ExprPos::default(),
    )?;
    assert_eq!(empty_and, Expr::BoolLiteral(true));

    let empty_add = expand_fold_expr(
        FoldOp::Binary(BinaryOp::Add),
        Some(&id_expr("args")),
        None,
        &empty_mapping,
        ExprPos::default(),
    );
    assert!(empty_add.is_err_and(|e| e.message.contains("empty expansion")));

    let empty_bin = expand_fold_expr(
        FoldOp::Binary(BinaryOp::Add),
        Some(&int_lit(0)),
        Some(&id_expr("args")),
        &empty_mapping,
        ExprPos::default(),
    )?;
    assert_eq!(empty_bin, int_lit(0));
    Ok(())
}

#[test]
fn test_expand_fold_expr_unary_right_and_binary_right() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = HashMap::new();
    mapping.insert(
        "args".to_string(),
        TemplateArg::NonTypePack(vec![1, 2, 3]),
    );

    // Unary Right: (args + ...) with [1, 2, 3] -> (1 + (2 + 3))
    let unary_right = expand_fold_expr(
        FoldOp::Binary(BinaryOp::Add),
        Some(&id_expr("args")),
        None,
        &mapping,
        ExprPos::default(),
    )?;
    let expected_right = Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(int_lit(1)),
        right: Box::new(Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(int_lit(2)),
            right: Box::new(int_lit(3)),
            pos: ExprPos::default(),
        }),
        pos: ExprPos::default(),
    };
    assert_eq!(unary_right, expected_right);

    // Binary Right: (args + ... + 0) with [1, 2] -> (1 + (2 + 0))
    let mut mapping_bin = HashMap::new();
    mapping_bin.insert(
        "args".to_string(),
        TemplateArg::NonTypePack(vec![1, 2]),
    );
    let bin_right = expand_fold_expr(
        FoldOp::Binary(BinaryOp::Add),
        Some(&id_expr("args")),
        Some(&int_lit(0)),
        &mapping_bin,
        ExprPos::default(),
    )?;
    let expected_bin_right = Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(int_lit(1)),
        right: Box::new(Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(int_lit(2)),
            right: Box::new(int_lit(0)),
            pos: ExprPos::default(),
        }),
        pos: ExprPos::default(),
    };
    assert_eq!(bin_right, expected_bin_right);

    Ok(())
}

#[test]
fn test_expand_fold_expr_empty_standard_fallbacks() -> Result<(), Box<dyn std::error::Error>> {
    let mut empty_mapping = HashMap::new();
    empty_mapping.insert("args".to_string(), TemplateArg::NonTypePack(vec![]));

    // Logical Or: (args || ...) -> false
    let or_res = expand_fold_expr(
        FoldOp::Binary(BinaryOp::LogicalOr),
        Some(&id_expr("args")),
        None,
        &empty_mapping,
        ExprPos::default(),
    )?;
    assert_eq!(or_res, Expr::BoolLiteral(false));

    // Comma: (... , args) -> 0 void
    let comma_res = expand_fold_expr(
        FoldOp::Binary(BinaryOp::Comma),
        None,
        Some(&id_expr("args")),
        &empty_mapping,
        ExprPos::default(),
    )?;
    assert_eq!(comma_res, Expr::IntLiteral(0, Type::Void));

    Ok(())
}

#[test]
fn test_expand_fold_expr_single_element() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = HashMap::new();
    mapping.insert(
        "args".to_string(),
        TemplateArg::NonTypePack(vec![42]),
    );

    // Unary: (... + args) with [42] -> 42
    let unary_left = expand_fold_expr(
        FoldOp::Binary(BinaryOp::Add),
        None,
        Some(&id_expr("args")),
        &mapping,
        ExprPos::default(),
    )?;
    assert_eq!(unary_left, int_lit(42));

    // Binary Left: (0 + ... + args) with [42] -> (0 + 42)
    let bin_left = expand_fold_expr(
        FoldOp::Binary(BinaryOp::Add),
        Some(&int_lit(0)),
        Some(&id_expr("args")),
        &mapping,
        ExprPos::default(),
    )?;
    assert_eq!(
        bin_left,
        Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(int_lit(0)),
            right: Box::new(int_lit(42)),
            pos: ExprPos::default(),
        }
    );

    Ok(())
}

#[test]
fn test_expand_fold_expr_diagnostics() {
    let mut mapping = HashMap::new();
    mapping.insert("a".to_string(), TemplateArg::NonTypePack(vec![1, 2]));
    mapping.insert("b".to_string(), TemplateArg::NonTypePack(vec![3, 4]));

    // Both operands have packs
    let err_both = expand_fold_expr(
        FoldOp::Binary(BinaryOp::Add),
        Some(&id_expr("a")),
        Some(&id_expr("b")),
        &mapping,
        ExprPos::default(),
    );
    assert!(matches!(
        err_both,
        Err(e) if e.message.contains("binary fold expression has unexpanded parameter packs in both operands")
    ));

    // Neither operand has packs
    let err_none = expand_fold_expr(
        FoldOp::Binary(BinaryOp::Add),
        Some(&int_lit(1)),
        Some(&int_lit(2)),
        &mapping,
        ExprPos::default(),
    );
    assert!(matches!(
        err_none,
        Err(e) if e.message.contains("pack expansion does not contain any unexpanded parameter packs")
    ));
}

#[test]
fn test_substitute_expr_fold() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = HashMap::new();
    mapping.insert(
        "vals".to_string(),
        TemplateArg::NonTypePack(vec![10, 20]),
    );

    let mut fold_expr = Expr::FoldExpr {
        op: FoldOp::Binary(BinaryOp::Add),
        left: None,
        right: Some(Box::new(id_expr("vals"))),
        pos: ExprPos::default(),
    };

    crate::sema::template_subst::substitute_expr(&mut fold_expr, &mapping, None)?;
    assert_eq!(
        fold_expr,
        Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(int_lit(10)),
            right: Box::new(int_lit(20)),
            pos: ExprPos::default(),
        }
    );
    Ok(())
}

#[test]
fn test_expand_fold_expr_type_pack_renaming() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = HashMap::new();
    mapping.insert(
        "args".to_string(),
        TemplateArg::TypePack(vec![Type::Int, Type::Double]),
    );

    let fold_res = expand_fold_expr(
        FoldOp::Binary(BinaryOp::Add),
        None,
        Some(&id_expr("args")),
        &mapping,
        ExprPos::default(),
    )?;

    let expected = Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(Expr::Identifier(
            super::pack_expansion::pack_element_name("args", 0),
        )),
        right: Box::new(Expr::Identifier(
            super::pack_expansion::pack_element_name("args", 1),
        )),
        pos: ExprPos::default(),
    };
    assert_eq!(fold_res, expected);
    Ok(())
}

#[test]
fn test_expand_fold_compound_assign() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = HashMap::new();
    mapping.insert(
        "args".to_string(),
        TemplateArg::NonTypePack(vec![1, 2]),
    );

    let fold_res = expand_fold_expr(
        FoldOp::Compound(BinaryOp::Add),
        Some(&id_expr("total")),
        Some(&id_expr("args")),
        &mapping,
        ExprPos::default(),
    )?;

    let expected = Expr::CompoundAssign {
        op: BinaryOp::Add,
        target: Box::new(Expr::CompoundAssign {
            op: BinaryOp::Add,
            target: Box::new(id_expr("total")),
            value: Box::new(int_lit(1)),
            pos: ExprPos::default(),
        }),
        value: Box::new(int_lit(2)),
        pos: ExprPos::default(),
    };
    assert_eq!(fold_res, expected);
    Ok(())
}

#[test]
fn test_fold_expr_in_sema_analyze_expr_fails() {
    let mut sema = crate::sema::Sema::new();
    let mut expr = Expr::FoldExpr {
        op: FoldOp::Binary(BinaryOp::Add),
        left: None,
        right: Some(Box::new(int_lit(1))),
        pos: ExprPos::default(),
    };
    let res = sema.analyze_expr(&mut expr);
    assert!(matches!(
        res,
        Err(e) if e.message.contains("pack expansion does not contain any unexpanded parameter packs")
    ));
}

#[test]
fn test_nttp_fold_expression_evaluation() -> Result<(), Box<dyn std::error::Error>> {
    use super::template_subst_nontype::evaluate_dependent_nontype;

    // Serialization via expr_to_dependent_str
    use crate::parser::types_array::expr_to_dependent_str;
    let unary_left_expr = Expr::FoldExpr {
        op: FoldOp::Binary(BinaryOp::Add),
        left: None,
        right: Some(Box::new(id_expr("Sizes"))),
        pos: ExprPos::default(),
    };
    assert_eq!(
        expr_to_dependent_str(&unary_left_expr),
        Some("(... + Sizes)".to_string())
    );

    let unary_right_expr = Expr::FoldExpr {
        op: FoldOp::Binary(BinaryOp::Add),
        left: Some(Box::new(id_expr("Sizes"))),
        right: None,
        pos: ExprPos::default(),
    };
    assert_eq!(
        expr_to_dependent_str(&unary_right_expr),
        Some("(Sizes + ...)".to_string())
    );

    let binary_expr = Expr::FoldExpr {
        op: FoldOp::Binary(BinaryOp::Add),
        left: Some(Box::new(int_lit(0))),
        right: Some(Box::new(id_expr("Sizes"))),
        pos: ExprPos::default(),
    };
    assert_eq!(
        expr_to_dependent_str(&binary_expr),
        Some("(0 + ... + Sizes)".to_string())
    );

    // Evaluation with evaluate_dependent_nontype
    let mut mapping = HashMap::new();
    mapping.insert(
        "Sizes".to_string(),
        TemplateArg::NonTypePack(vec![1, 2, 3]),
    );
    let res_right = evaluate_dependent_nontype("(Sizes + ...)", &mapping, None)?;
    assert_eq!(res_right, Type::NonType(6));

    let res_left = evaluate_dependent_nontype("(... + Sizes)", &mapping, None)?;
    assert_eq!(res_left, Type::NonType(6));

    let res_bin = evaluate_dependent_nontype("(0 + ... + Sizes)", &mapping, None)?;
    assert_eq!(res_bin, Type::NonType(6));

    let mut empty_mapping = HashMap::new();
    empty_mapping.insert("Sizes".to_string(), TemplateArg::NonTypePack(vec![]));
    let res_empty = evaluate_dependent_nontype("(Sizes + ... + 10)", &empty_mapping, None)?;
    assert_eq!(res_empty, Type::NonType(10));

    Ok(())
}

#[test]
fn test_nttp_fold_expression_template_parsing_and_subst() -> Result<(), Box<dyn std::error::Error>> {
    let mut lexer = crate::lexer::Lexer::new("FixedBuffer<(Sizes + ...)>");
    let tokens = lexer.tokenize()?;
    let mut parser = crate::parser::Parser::new(tokens);
    parser.allow_unknown_types = true;
    let parsed_type = parser.parse_type()?;
    assert_eq!(
        parsed_type,
        Type::TemplateInstance {
            name: "FixedBuffer".to_string(),
            args: vec![Type::DependentNonType("(Sizes + ...)".to_string())],
        }
    );

    let mut mapping = HashMap::new();
    mapping.insert(
        "Sizes".to_string(),
        TemplateArg::NonTypePack(vec![1, 2, 3]),
    );
    let subst_type = crate::sema::template_subst::substitute_type(&parsed_type, &mapping, None)?;
    assert_eq!(
        subst_type,
        Type::TemplateInstance {
            name: "FixedBuffer".to_string(),
            args: vec![Type::NonType(6)],
        }
    );

    let dep_array = Type::DependentArray(Box::new(Type::Int), "(Sizes + ...)".to_string());
    let subst_array = crate::sema::template_subst::substitute_type(&dep_array, &mapping, None)?;
    assert_eq!(subst_array, Type::Array(Box::new(Type::Int), 6));

    Ok(())
}

