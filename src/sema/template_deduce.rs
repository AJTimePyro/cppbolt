//! Template argument deduction and explicit template argument resolution.

use crate::ast::decl::FunctionDecl;
use crate::ast::expr::{Expr, ExprPos};
use crate::ast::template::TemplateParam;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::lexer::TokenKind;
use crate::name::QualifiedName;
use crate::sema::pack_expansion::TemplateArg;
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
        types.push(parser.parse_template_arg_type()?);
        if parser.peek() != &TokenKind::Comma { break; }
        parser.advance();
    }
    Ok(types)
}

/// Convenience wrapper for parsing type lists without active type parameters.
pub fn parse_types_list(s: &str, known_templates: &[String]) -> Result<Vec<Type>, CompileError> {
    parse_types_list_with_params(s, known_templates, &[])
}

/// Deduces template arguments for a function template call.
fn deduce_pack_arguments(
    pack_param: &crate::ast::decl::Param,
    template_params: &[TemplateParam],
    call_args: (&[Type], &[Expr]),
    start: usize,
    pos: ExprPos,
) -> Result<(String, Vec<Type>), CompileError> {
    let (arg_types, args) = call_args;
    let pattern_ty = match &pack_param.ty {
        Type::PackExpansion(inner) => &**inner,
        other => other,
    };
    let pack_tparam = template_params.iter().find(|p| p.is_pack);
    let pack_name = pack_tparam
        .map(|p| p.name.clone())
        .unwrap_or_else(|| pack_param.name.clone());

    let mut pack_types = Vec::new();
    for (i, arg_ty) in arg_types.iter().enumerate().skip(start) {
        let is_lval = args.get(i).is_some_and(|e| e.is_lvalue()) || arg_ty.is_lvalue_reference();
        let adjusted_arg_ty = if !pattern_ty.is_reference() {
            arg_ty.strip_reference().decay_array()
        } else {
            arg_ty.clone()
        };
        let mut elem_deduced = HashMap::new();
        deduce_type(pattern_ty, &adjusted_arg_ty, &mut elem_deduced, is_lval, pos)?;
        if let Some(t) = elem_deduced.remove(&pack_name) {
            pack_types.push(t);
        } else if let Some((_, t)) = elem_deduced.into_iter().next() {
            pack_types.push(t);
        } else {
            return Err(CompileError::new(
                format!("could not deduce template argument for '{}'", pack_name),
                pos.line,
                pos.col,
            ));
        }
    }
    Ok((pack_name, pack_types))
}

fn validate_arg_counts(
    has_pack: bool,
    non_pack_params: &[crate::ast::decl::Param],
    total_params: usize,
    arg_count: usize,
    pos: ExprPos,
) -> Result<(), CompileError> {
    if !has_pack && arg_count > total_params {
        return Err(CompileError::new(
            format!(
                "too many arguments for function template: expected at most {}, found {}",
                total_params, arg_count
            ),
            pos.line,
            pos.col,
        ));
    }
    let min_args = non_pack_params.iter().take_while(|p| p.default_val.is_none()).count();
    if arg_count < min_args {
        return Err(CompileError::new(
            format!(
                "too few arguments for function template: expected at least {}, found {}",
                min_args, arg_count
            ),
            pos.line,
            pos.col,
        ));
    }
    Ok(())
}

fn fill_missing_deduced(
    template_params: &[TemplateParam],
    result: &mut HashMap<String, TemplateArg>,
    pos: ExprPos,
) -> Result<(), CompileError> {
    for param in template_params {
        if !result.contains_key(&param.name) {
            if param.is_pack {
                result.insert(param.name.clone(), TemplateArg::TypePack(Vec::new()));
            } else if let Some(ref def_ty) = param.default_type {
                let subst_def = crate::sema::template_subst::substitute_type(def_ty, result, None);
                result.insert(param.name.clone(), TemplateArg::from(subst_def));
            } else {
                return Err(CompileError::new(
                    format!("could not deduce template argument for '{}'", param.name),
                    pos.line,
                    pos.col,
                ));
            }
        }
    }
    Ok(())
}

/// Deduces template arguments for a function template call.
pub fn deduce_template_arguments(
    template_params: &[TemplateParam],
    func_decl: &FunctionDecl,
    explicit_args: &[Type],
    call_args: (&[Type], &[Expr]),
    pos: ExprPos,
) -> Result<HashMap<String, TemplateArg>, CompileError> {
    let (arg_types, args) = call_args;
    let has_pack = func_decl.params.last().is_some_and(|p| p.is_pack);
    let non_pack_count = if has_pack {
        func_decl.params.len().saturating_sub(1)
    } else {
        func_decl.params.len()
    };
    let non_pack_params = &func_decl.params[..non_pack_count];
    validate_arg_counts(has_pack, non_pack_params, func_decl.params.len(), arg_types.len(), pos)?;

    let has_t_pack = template_params.last().is_some_and(|p| p.is_pack);
    let non_pack_t_count = if has_t_pack {
        template_params.len().saturating_sub(1)
    } else {
        template_params.len()
    };

    let mut deduced = HashMap::new();
    let mut explicit_pack = Vec::new();
    for (i, explicit_arg) in explicit_args.iter().enumerate() {
        if i < non_pack_t_count {
            deduced.insert(template_params[i].name.clone(), explicit_arg.clone());
        } else if has_t_pack {
            explicit_pack.push(explicit_arg.clone());
        }
    }

    for (i, p) in non_pack_params.iter().enumerate() {
        if i >= arg_types.len() {
            break;
        }
        let arg_ty = &arg_types[i];
        let is_lval = args.get(i).is_some_and(|e| e.is_lvalue()) || arg_ty.is_lvalue_reference();
        let adjusted_arg_ty = if !p.ty.is_reference() {
            arg_ty.strip_reference().decay_array()
        } else {
            arg_ty.clone()
        };
        deduce_type(&p.ty, &adjusted_arg_ty, &mut deduced, is_lval, pos)?;
    }

    let mut result: HashMap<String, TemplateArg> = HashMap::new();
    for (k, v) in deduced {
        result.insert(k, TemplateArg::from(v));
    }

    if has_pack {
        let (pack_name, mut pack_types) = deduce_pack_arguments(
            &func_decl.params[non_pack_count],
            template_params,
            call_args,
            non_pack_count,
            pos,
        )?;
        if !explicit_pack.is_empty() {
            pack_types.splice(0..0, explicit_pack);
        }
        result.insert(pack_name, TemplateArg::TypePack(pack_types));
    }

    fill_missing_deduced(template_params, &mut result, pos)?;
    Ok(result)
}

/// Pads missing template arguments from parameter default types.
pub fn fill_default_template_args(
    name: &str,
    template_params: &[TemplateParam],
    args: &[Type],
    pos: (usize, usize),
) -> Result<Vec<Type>, CompileError> {
    let has_pack = template_params.last().is_some_and(|p| p.is_pack);
    let non_pack_params = if has_pack {
        &template_params[..template_params.len().saturating_sub(1)]
    } else {
        template_params
    };
    let min_args = non_pack_params
        .iter()
        .take_while(|p| p.default_type.is_none())
        .count();
    if args.len() < min_args || (!has_pack && args.len() > template_params.len()) {
        let expected_str = if !has_pack && min_args == template_params.len() {
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
    for (param, arg) in non_pack_params.iter().zip(args.iter()) {
        mapping.insert(param.name.clone(), arg.clone());
    }
    if args.len() < non_pack_params.len() {
        for param in &non_pack_params[args.len()..] {
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
            let subst_map = crate::sema::pack_expansion::to_template_arg_map(&mapping);
            let subst_ty = crate::sema::template_subst::substitute_type(def_ty, &subst_map, None);
            mapping.insert(param.name.clone(), subst_ty.clone());
            result.push(subst_ty);
        }
    }
    validate_template_arg_kinds(template_params, non_pack_params.len(), &result, pos)?;
    Ok(result)
}

fn validate_template_arg_kinds(
    params: &[TemplateParam],
    non_pack_count: usize,
    args: &[Type],
    pos: (usize, usize),
) -> Result<(), CompileError> {
    for (i, arg) in args.iter().enumerate() {
        let param = if i < non_pack_count {
            &params[i]
        } else if let Some(last) = params.last() {
            last
        } else {
            continue;
        };
        if param.param_type.is_some() && !matches!(arg, Type::NonType(_) | Type::TypeParam(_)) {
            return Err(CompileError::new(
                "template argument for non-type parameter must be a constant expression",
                pos.0,
                pos.1,
            ));
        }
        if param.param_type.is_none() && matches!(arg, Type::NonType(_)) {
            return Err(CompileError::new(
                format!(
                    "type template parameter '{}' cannot take non-type argument",
                    param.name
                ),
                pos.0,
                pos.1,
            ));
        }
    }
    Ok(())
}

pub fn build_template_arg_map(
    template_params: &[TemplateParam],
    effective_args: &[Type],
) -> HashMap<String, crate::sema::pack_expansion::TemplateArg> {
    use crate::sema::pack_expansion::TemplateArg;
    let mut mapping = HashMap::new();
    let has_pack = template_params.last().is_some_and(|p| p.is_pack);
    let non_pack_count = if has_pack {
        template_params.len().saturating_sub(1)
    } else {
        template_params.len()
    };
    for (param, arg) in template_params[..non_pack_count].iter().zip(effective_args.iter()) {
        mapping.insert(param.name.clone(), TemplateArg::from(arg.clone()));
    }
    if has_pack && let Some(pack_param) = template_params.last() {
        let trailing = if effective_args.len() > non_pack_count {
            &effective_args[non_pack_count..]
        } else {
            &[]
        };
        if pack_param.param_type.is_some() {
            let vals = trailing
                .iter()
                .filter_map(|t| match t {
                    Type::NonType(v) => Some(*v),
                    _ => None,
                })
                .collect();
            mapping.insert(pack_param.name.clone(), TemplateArg::NonTypePack(vals));
        } else {
            mapping.insert(
                pack_param.name.clone(),
                TemplateArg::TypePack(trailing.to_vec()),
            );
        }
    }
    mapping
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
