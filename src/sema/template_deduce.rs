//! Template argument deduction and explicit template argument resolution.

use crate::ast::decl::FunctionDecl;
use crate::ast::expr::{Expr, ExprPos};
use crate::ast::template::TemplateParam;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::lexer::TokenKind;
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
pub fn parse_types_list(s: &str) -> Result<Vec<Type>, CompileError> {
    let mut lexer = crate::lexer::Lexer::new(s);
    let tokens = lexer.tokenize()?;
    let mut parser = crate::parser::Parser::new(tokens);
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
        let is_lval = arg_expr.is_lvalue();
        deduce_type(&p.ty, arg_ty, &mut deduced, is_lval, pos)?;
    }

    for param in template_params {
        if !deduced.contains_key(&param.name) {
            return Err(CompileError::new(
                format!("could not deduce template argument for '{}'", param.name),
                pos.line,
                pos.col,
            ));
        }
    }

    Ok(deduced)
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
        Type::Const(inner) => {
            deduce_type(inner, arg_ty.strip_const(), deduced, is_lvalue, pos)?;
        }
        Type::Pointer(inner) => {
            if let Type::Pointer(arg_inner) = arg_ty.strip_reference() {
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
        if name == a_name && args.len() == a_args.len() {
            for (p_arg, a_arg) in args.iter().zip(a_args.iter()) {
                deduce_type(p_arg, a_arg, deduced, true, pos)?;
            }
        }
    } else if let Type::Struct(sname) = clean_arg
        && let Some((s_base, s_args_str)) = split_template_call_name(sname)
        && s_base == name
        && let Ok(s_args) = parse_types_list(s_args_str)
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
        if existing != &deduced_arg {
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
