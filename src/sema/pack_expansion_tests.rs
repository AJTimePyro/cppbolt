//! Unit tests for variadic template pack expansion and TemplateArg environment.
use crate::ast::types::FunctionQualifiers;

use super::constexpr_eval::{eval_constexpr_with_context, eval_constexpr_with_mapping};
use super::pack_expansion::*;
use crate::ast::decl::{FunctionDecl, Param, RefQualifier};
use crate::ast::expr::{Expr, ExprPos};
use crate::ast::template::TemplateParam;
use crate::ast::types::Type;
use crate::sema::template_deduce::deduce_template_arguments;
use std::collections::HashMap;

#[test]
fn test_template_arg_conversions() {
    let ty_arg = TemplateArg::from(Type::Int);
    assert_eq!(ty_arg, TemplateArg::Type(Type::Int));
    assert_eq!(ty_arg.as_type(), Some(&Type::Int));
    assert!(!ty_arg.is_pack());
    assert_eq!(ty_arg.pack_len(), None);

    let nttp_arg = TemplateArg::from(Type::NonType(42));
    assert_eq!(nttp_arg, TemplateArg::NonType(42));

    let pack = TemplateArg::TypePack(vec![Type::Int, Type::Double]);
    assert!(pack.is_pack());
    assert_eq!(pack.pack_len(), Some(2));
}

#[test]
fn test_expand_type_list() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = HashMap::new();
    mapping.insert(
        "Ts".to_string(),
        TemplateArg::TypePack(vec![Type::Int, Type::Double]),
    );

    let types = vec![
        Type::Char,
        Type::PackExpansion(Box::new(Type::TypeParam("Ts".to_string()))),
        Type::Bool,
    ];
    let expanded = expand_type_list(&types, &mapping, (1, 1))?;
    assert_eq!(
        expanded,
        vec![Type::Char, Type::Int, Type::Double, Type::Bool]
    );
    Ok(())
}

#[test]
fn test_expand_nontype_pack_expr_list() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = HashMap::new();
    mapping.insert("Ns".to_string(), TemplateArg::NonTypePack(vec![10, 20, 30]));

    let exprs = vec![Expr::PackExpansion {
        pattern: Box::new(Expr::Identifier("Ns".to_string())),
        pos: ExprPos::default(),
    }];
    let expanded = expand_expr_list(&exprs, &mapping)?;
    assert_eq!(expanded.len(), 3);
    assert_eq!(expanded[0], Expr::IntLiteral(10, Type::Int));
    assert_eq!(expanded[1], Expr::IntLiteral(20, Type::Int));
    assert_eq!(expanded[2], Expr::IntLiteral(30, Type::Int));
    Ok(())
}

#[test]
fn test_pack_length_mismatch_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = HashMap::new();
    mapping.insert(
        "Ts".to_string(),
        TemplateArg::TypePack(vec![Type::Int, Type::Double]),
    );
    mapping.insert("Us".to_string(), TemplateArg::TypePack(vec![Type::Char]));

    let pattern = Expr::Call {
        callee: Box::new(Expr::Identifier("pair".to_string())),
        args: vec![
            Expr::Identifier("Ts".to_string()),
            Expr::Identifier("Us".to_string()),
        ],
        pos: ExprPos::new(1, 1),
        dependent: false,
    };
    let exprs = vec![Expr::PackExpansion {
        pattern: Box::new(pattern),
        pos: ExprPos::new(1, 1),
    }];
    let err = match expand_expr_list(&exprs, &mapping) {
        Err(e) => e,
        Ok(_) => return Err("expected mismatch error".into()),
    };
    assert!(
        err.message
            .contains("pack expansion contains parameter packs of different lengths")
    );
    Ok(())
}

#[test]
fn test_unexpanded_pack_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = HashMap::new();
    mapping.insert("Ts".to_string(), TemplateArg::TypePack(vec![Type::Int]));

    let exprs = vec![Expr::Identifier("Ts".to_string())];
    let err = match expand_expr_list(&exprs, &mapping) {
        Err(e) => e,
        Ok(_) => return Err("expected unexpanded pack error".into()),
    };
    assert!(err.message.contains("parameter pack 'Ts' used unexpanded"));

    let types = vec![Type::TypeParam("Ts".to_string())];
    let err_ty = match expand_type_list(&types, &mapping, (1, 1)) {
        Err(e) => e,
        Ok(_) => return Err("expected unexpanded pack error in type".into()),
    };
    assert!(
        err_ty
            .message
            .contains("parameter pack 'Ts' used unexpanded")
    );
    Ok(())
}

#[test]
fn test_pack_expansion_without_packs_error() -> Result<(), Box<dyn std::error::Error>> {
    let mapping = HashMap::new();
    let exprs = vec![Expr::PackExpansion {
        pattern: Box::new(Expr::IntLiteral(42, Type::Int)),
        pos: ExprPos::new(1, 1),
    }];
    let err = match expand_expr_list(&exprs, &mapping) {
        Err(e) => e,
        Ok(_) => return Err("expected missing packs error".into()),
    };
    assert!(
        err.message
            .contains("pack expansion does not contain any unexpanded parameter packs")
    );
    Ok(())
}

#[test]
fn test_constexpr_eval_sizeof_pack() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = HashMap::new();
    mapping.insert(
        "Args".to_string(),
        TemplateArg::TypePack(vec![Type::Int, Type::Double, Type::Char]),
    );

    let expr = Expr::SizeofPack {
        name: "Args".to_string(),
        pos: ExprPos::new(1, 1),
    };
    let len = eval_constexpr_with_mapping(&expr, None, &mapping)?;
    assert_eq!(len, 3);

    let mut pack_lengths = HashMap::new();
    pack_lengths.insert("Args".to_string(), 3);
    let len2 = eval_constexpr_with_context(&expr, None, None, Some(&pack_lengths))?;
    assert_eq!(len2, 3);
    Ok(())
}

fn make_tparam(name: &str, is_pack: bool) -> TemplateParam {
    TemplateParam {
        name: name.to_string(),
        default_type: None,
        param_type: None,
        is_pack,
        line: 1,
        col: 1,
    }
}

fn make_test_func(tmpl_params: Vec<TemplateParam>, params: Vec<Param>) -> FunctionDecl {
    FunctionDecl {
        qualifiers: FunctionQualifiers::default(),
        name: "f".to_string(),
        return_type: Type::Void,
        params,
        body: None,
        is_virtual: false,
        is_pure_virtual: false,
        is_const: false,
        is_explicit: false,
        ref_qualifier: RefQualifier::None,
        is_static: false,
        ctor_initializers: Vec::new(),
        template_params: tmpl_params,
        line: 1,
        col: 1,
    }
}

#[test]
fn test_deduce_variadic_function_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let tmpl_params = vec![make_tparam("T", false), make_tparam("Rest", true)];
    let params = vec![
        Param {
            name: "first".to_string(),
            ty: Type::TypeParam("T".to_string()),
            default_val: None,
            is_pack: false,
        },
        Param {
            name: "rest".to_string(),
            ty: Type::TypeParam("Rest".to_string()),
            default_val: None,
            is_pack: true,
        },
    ];
    let func = make_test_func(tmpl_params.clone(), params);
    let arg_types = vec![Type::Int, Type::Double, Type::Char];
    let args = vec![
        Expr::IntLiteral(1, Type::Int),
        Expr::FloatLiteral(2.0, Type::Double),
        Expr::CharLiteral('c'),
    ];
    let deduced = deduce_template_arguments(
        &tmpl_params,
        &func,
        &[],
        (&arg_types, &args),
        ExprPos::default(),
    )?;
    assert_eq!(deduced.get("T"), Some(&TemplateArg::Type(Type::Int)));
    assert_eq!(
        deduced.get("Rest"),
        Some(&TemplateArg::TypePack(vec![Type::Double, Type::Char]))
    );

    let (s_types, s_args) = (vec![Type::Int], vec![Expr::IntLiteral(10, Type::Int)]);
    let empty = deduce_template_arguments(
        &tmpl_params,
        &func,
        &[],
        (&s_types, &s_args),
        ExprPos::default(),
    )?;
    assert_eq!(empty.get("Rest"), Some(&TemplateArg::TypePack(vec![])));
    Ok(())
}

#[test]
fn test_deduce_variadic_forwarding_references() -> Result<(), Box<dyn std::error::Error>> {
    let tmpl_params = vec![make_tparam("Args", true)];
    let params = vec![Param {
        name: "args".to_string(),
        ty: Type::RvalueReference(Box::new(Type::TypeParam("Args".to_string()))),
        default_val: None,
        is_pack: true,
    }];
    let func = make_test_func(tmpl_params.clone(), params);
    let arg_types = vec![Type::Reference(Box::new(Type::Int)), Type::Int];
    let args = vec![
        Expr::Identifier("lval_x".to_string()),
        Expr::IntLiteral(42, Type::Int),
    ];
    let deduced = deduce_template_arguments(
        &tmpl_params,
        &func,
        &[],
        (&arg_types, &args),
        ExprPos::default(),
    )?;
    assert_eq!(
        deduced.get("Args"),
        Some(&TemplateArg::TypePack(vec![
            Type::Reference(Box::new(Type::Int)),
            Type::Int
        ]))
    );
    Ok(())
}

#[test]
fn test_pack_element_name_reserved_prefix() {
    let name0 = pack_element_name("args", 0);
    let name1 = pack_element_name("args", 1);
    assert_eq!(name0, "__pack_args_0");
    assert_eq!(name1, "__pack_args_1");
    assert_ne!(name0, "arg0");
    assert_ne!(name1, "arg1");
    assert_ne!(name0, "args0");

    let val0 = pack_element_name("val", 0);
    assert_eq!(val0, "__pack_val_0");
    assert_ne!(val0, "val0");
}
