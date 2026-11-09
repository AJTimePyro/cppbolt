//! Template argument deduction and explicit template argument resolution.

use crate::ast::decl::FunctionDecl;
use crate::ast::expr::{Expr, ExprPos};
use crate::ast::template::TemplateParam;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::lexer::TokenKind;
use crate::name::QualifiedName;
use std::collections::HashMap;

/// Splits a template call identifier such as `my_max<int>` into `("my_max", "int")`.
pub fn split_template_call_name(name: &str) -> Option<(&str, &str)> {
    if !name.ends_with('>') {
        return None;
    }
    let open = name.find('<')?;
    Some((&name[..open], &name[open + 1..name.len() - 1]))
}

/// Parses a comma-separated list of types using the compiler's lexer and parser.
pub fn parse_types_list_with_params(
    s: &str,
    known_templates: &[String],
    type_params: &[String],
) -> Result<Vec<Type>, CompileError> {
    let mut lexer = crate::lexer::Lexer::new(s);
    let tokens = lexer.tokenize()?;
    let mut parser = crate::parser::Parser::new(tokens);
    parser.allow_unknown_types = true;
    for t in known_templates {
        parser.register_template_name(t);
    }
    if !type_params.is_empty() {
        let set: std::collections::HashSet<String> = type_params.iter().cloned().collect();
        parser.active_type_params.push(set);
    }
    let mut types = Vec::new();
    while parser.peek() != &TokenKind::Eof {
        types.push(parser.parse_type()?);
        if parser.peek() == &TokenKind::Comma {
            parser.advance();
        } else {
            break;
        }
    }
    Ok(types)
}

/// Convenience wrapper for parsing type lists without active type parameters.
pub fn parse_types_list(s: &str, known_templates: &[String]) -> Result<Vec<Type>, CompileError> {
    parse_types_list_with_params(s, known_templates, &[])
}

/// Deduces template arguments for a function template call.
pub fn deduce_template_arguments(
    template_params: &[TemplateParam],
    func_decl: &FunctionDecl,
    explicit_args: &[Type],
    call_args: (&[Type], &[Expr]),
    pos: ExprPos,
) -> Result<HashMap<String, Type>, CompileError> {
    let (arg_types, args) = call_args;
    let mut deduced = HashMap::new();
    for (param, explicit_arg) in template_params.iter().zip(explicit_args.iter()) {
        deduced.insert(param.name.clone(), explicit_arg.clone());
    }

    if arg_types.len() > func_decl.params.len() {
        return Err(CompileError::new(
            format!(
                "too many arguments for function template: expected at most {}, found {}",
                func_decl.params.len(),
                arg_types.len()
            ),
            pos.line,
            pos.col,
        ));
    }

    let min_args = func_decl
        .params
        .iter()
        .take_while(|p| p.default_val.is_none())
        .count();
    if arg_types.len() < min_args {
        return Err(CompileError::new(
            format!(
                "too few arguments for function template: expected at least {}, found {}",
                min_args,
                arg_types.len()
            ),
            pos.line,
            pos.col,
        ));
    }

    for (p, (arg_ty, arg_expr)) in func_decl
        .params
        .iter()
        .zip(arg_types.iter().zip(args.iter()))
    {
        let is_lval = arg_expr.is_lvalue() || arg_ty.is_lvalue_reference();
        let adjusted_arg_ty = if !p.ty.is_reference() {
            arg_ty.strip_reference().decay_array()
        } else {
            arg_ty.clone()
        };
        deduce_type(&p.ty, &adjusted_arg_ty, &mut deduced, is_lval, pos)?;
    }

    for param in template_params {
        if !deduced.contains_key(&param.name) {
            if let Some(ref def_ty) = param.default_type {
                let subst_def =
                    crate::sema::template_subst::substitute_type(def_ty, &deduced, None);
                deduced.insert(param.name.clone(), subst_def);
            } else {
                return Err(CompileError::new(
                    format!("could not deduce template argument for '{}'", param.name),
                    pos.line,
                    pos.col,
                ));
            }
        }
    }

    Ok(deduced)
}

/// Pads missing template arguments from parameter default types.
pub fn fill_default_template_args(
    name: &str,
    template_params: &[TemplateParam],
    args: &[Type],
    pos: (usize, usize),
) -> Result<Vec<Type>, CompileError> {
    let min_args = template_params
        .iter()
        .take_while(|p| p.default_type.is_none())
        .count();
    if args.len() < min_args || args.len() > template_params.len() {
        let expected_str = if min_args == template_params.len() {
            format!("{}", template_params.len())
        } else {
            format!("at least {}", min_args)
        };
        return Err(CompileError::new(
            format!(
                "wrong number of template arguments for '{}': expected {}, found {}",
                name,
                expected_str,
                args.len()
            ),
            pos.0,
            pos.1,
        ));
    }

    let mut result = args.to_vec();
    let mut mapping = HashMap::new();
    for (param, arg) in template_params.iter().zip(args.iter()) {
        mapping.insert(param.name.clone(), arg.clone());
    }
    for param in &template_params[args.len()..] {
        let Some(def_ty) = param.default_type.as_ref() else {
            return Err(CompileError::new(
                format!(
                    "missing default argument for template parameter '{}'",
                    param.name
                ),
                pos.0,
                pos.1,
            ));
        };
        let subst_ty = crate::sema::template_subst::substitute_type(def_ty, &mapping, None);
        mapping.insert(param.name.clone(), subst_ty.clone());
        result.push(subst_ty);
    }
    Ok(result)
}

fn deduce_type(
    param_ty: &Type,
    arg_ty: &Type,
    deduced: &mut HashMap<String, Type>,
    is_lvalue: bool,
    pos: ExprPos,
) -> Result<(), CompileError> {
    match param_ty {
        Type::TypeParam(name) => {
            let deduced_arg = arg_ty.strip_reference().strip_const().clone();
            record_deduced(name, deduced_arg, deduced, pos)?;
        }
        Type::Reference(inner) => {
            if let Type::Const(tparam) = &**inner {
                if let Type::TypeParam(name) = &**tparam {
                    let deduced_arg = arg_ty.strip_reference().strip_const().clone();
                    record_deduced(name, deduced_arg, deduced, pos)?;
                    return Ok(());
                }
            } else if let Type::TypeParam(name) = &**inner {
                if !is_lvalue {
                    return Err(CompileError::new(
                        format!(
                            "cannot bind non-const lvalue reference of type '{}&' to an rvalue",
                            name
                        ),
                        pos.line,
                        pos.col,
                    ));
                }
                let deduced_arg = arg_ty.strip_reference().clone();
                record_deduced(name, deduced_arg, deduced, pos)?;
                return Ok(());
            }
            deduce_type(inner, arg_ty.strip_reference(), deduced, is_lvalue, pos)?;
        }
        Type::RvalueReference(inner) => {
            if let Type::TypeParam(name) = &**inner {
                let deduced_arg = if is_lvalue {
                    Type::Reference(Box::new(arg_ty.strip_reference().clone()))
                } else {
                    arg_ty.strip_reference().clone()
                };
                record_deduced(name, deduced_arg, deduced, pos)?;
                return Ok(());
            }
            if is_lvalue {
                return Err(CompileError::new(
                    "cannot bind rvalue reference to an lvalue".to_string(),
                    pos.line,
                    pos.col,
                ));
            }
            deduce_type(inner, arg_ty.strip_reference(), deduced, is_lvalue, pos)?;
        }
        Type::Const(inner) => {
            deduce_type(inner, arg_ty.strip_const(), deduced, is_lvalue, pos)?;
        }
        Type::Pointer(inner) => {
            if let Type::Pointer(arg_inner) = arg_ty.strip_reference().strip_const() {
                deduce_type(inner, arg_inner, deduced, true, pos)?;
            }
        }
        Type::Array(inner, size) => {
            if let Type::Array(arg_inner, arg_size) = arg_ty.strip_reference().strip_const()
                && size == arg_size
            {
                deduce_type(inner, arg_inner, deduced, true, pos)?;
            }
        }
        Type::TemplateInstance { name, args } => {
            deduce_template_instance(name, args, arg_ty, deduced, pos)?;
        }
        _ => {}
    }
    Ok(())
}

fn deduce_template_instance(
    name: &str,
    args: &[Type],
    arg_ty: &Type,
    deduced: &mut HashMap<String, Type>,
    pos: ExprPos,
) -> Result<(), CompileError> {
    let clean_arg = arg_ty.strip_reference().strip_const();
    if let Type::TemplateInstance {
        name: a_name,
        args: a_args,
    } = clean_arg
    {
        if QualifiedName::new(name).matches(QualifiedName::new(a_name))
            && args.len() == a_args.len()
        {
            for (p_arg, a_arg) in args.iter().zip(a_args.iter()) {
                deduce_type(p_arg, a_arg, deduced, true, pos)?;
            }
        }
    } else if let Type::Struct(sname) = clean_arg
        && let Some((s_base, s_args_str)) = split_template_call_name(sname)
        && QualifiedName::new(s_base).matches(QualifiedName::new(name))
        && let Ok(s_args) = parse_types_list(s_args_str, &[name.to_string()])
        && args.len() == s_args.len()
    {
        for (p_arg, a_arg) in args.iter().zip(s_args.iter()) {
            deduce_type(p_arg, a_arg, deduced, true, pos)?;
        }
    }
    Ok(())
}

fn record_deduced(
    name: &str,
    deduced_arg: Type,
    deduced: &mut HashMap<String, Type>,
    pos: ExprPos,
) -> Result<(), CompileError> {
    if let Some(existing) = deduced.get(name) {
        if existing != &deduced_arg && existing.to_c_type() != deduced_arg.to_c_type() {
            return Err(CompileError::new(
                format!(
                    "template argument deduction conflict for '{}': deduced '{}' and '{}'",
                    name,
                    existing.to_c_type(),
                    deduced_arg.to_c_type()
                ),
                pos.line,
                pos.col,
            ));
        }
    } else {
        deduced.insert(name.to_string(), deduced_arg);
    }
    Ok(())
}
