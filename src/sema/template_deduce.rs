//! Template argument deduction and explicit template argument resolution.

use crate::ast::decl::FunctionDecl;
use crate::ast::expr::{Expr, ExprPos};
use crate::ast::template::TemplateParam;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::lexer::TokenKind;
use crate::sema::pack_expansion::TemplateArg;
use crate::sema::template_deduce_type::{deduce_type, type_has_template_param};
use std::collections::HashMap;

/// Splits a template call identifier such as `my_max<int>` into `("my_max", "int")`.
pub fn split_template_call_name(name: &str) -> Option<(&str, &str)> {
    if !name.ends_with('>') {
        return None;
    }
    let mut depth = 0;
    let mut open = None;
    for (i, &b) in name.as_bytes().iter().enumerate().rev() {
        if b == b'>' {
            depth += 1;
        } else if b == b'<' {
            depth -= 1;
            if depth == 0 {
                open = Some(i);
                break;
            }
        }
    }
    let open = open?;
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
        if parser.peek() != &TokenKind::Comma {
            break;
        }
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
    let pack_name = match pattern_ty {
        Type::TypeParam(name) => name.clone(),
        _ => template_params
            .iter()
            .find(|p| p.is_pack)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| pack_param.name.clone()),
    };

    let mut pack_types = Vec::new();
    for (i, arg_ty) in arg_types.iter().enumerate().skip(start) {
        let is_lval = args.get(i).is_some_and(|e| e.is_lvalue()) || arg_ty.is_lvalue_reference();
        let adjusted_arg_ty = if !pattern_ty.is_reference() {
            arg_ty.strip_reference().decay_array()
        } else {
            arg_ty.clone()
        };
        let mut elem_deduced = HashMap::new();
        deduce_type(
            pattern_ty,
            &adjusted_arg_ty,
            &mut elem_deduced,
            is_lval,
            pos,
        )?;
        let deduced_ty = elem_deduced
            .remove(&pack_name)
            .or_else(|| {
                template_params
                    .iter()
                    .find(|p| p.is_pack)
                    .and_then(|p| elem_deduced.remove(&p.name))
            })
            .or_else(|| {
                let mut items: Vec<_> = elem_deduced.into_iter().collect();
                items.sort_by(|a, b| a.0.cmp(&b.0));
                items.into_iter().next().map(|(_, ty)| ty)
            });
        match deduced_ty {
            Some(TemplateArg::Type(t)) => pack_types.push(t),
            Some(TemplateArg::NonType(v)) => pack_types.push(Type::NonType(v)),
            _ => {
                return Err(CompileError::new(
                    format!("could not deduce template argument for '{}'", pack_name),
                    pos.line,
                    pos.col,
                ));
            }
        }
    }
    Ok((pack_name, pack_types))
}

fn validate_arg_counts(
    has_pack: bool,
    fixed_params: &[&crate::ast::decl::Param],
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
    let min_args = fixed_params
        .iter()
        .take_while(|p| p.default_val.is_none())
        .count();
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
                let subst_def = crate::sema::template_subst::substitute_type(def_ty, result, None)?;
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

fn insert_explicit_template_args(
    template_params: &[TemplateParam],
    explicit_args: &[Type],
    deduced: &mut HashMap<String, TemplateArg>,
    explicit_pack: &mut Vec<Type>,
) {
    // A parameter pack need not be the last template parameter. Explicit
    // arguments fill the non-pack parameters before the pack, and every
    // remaining explicit argument is absorbed by the pack; parameters after
    // the pack are left for deduction from the call.
    let pack_index = template_params.iter().position(|p| p.is_pack);
    let non_pack_t_count = pack_index.unwrap_or(template_params.len());
    for (i, explicit_arg) in explicit_args.iter().enumerate() {
        if i < non_pack_t_count {
            let mut arg = TemplateArg::from(explicit_arg.clone());
            // A null non-type argument needs the declared parameter type so
            // substitution can emit a correctly typed null member pointer.
            if let TemplateArg::NonTypeNull(ty) = &mut arg
                && ty.is_none()
            {
                *ty = template_params[i].param_type.clone();
            }
            deduced.insert(template_params[i].name.clone(), arg);
        } else if pack_index.is_some() {
            explicit_pack.push(explicit_arg.clone());
        }
    }
}

fn validate_explicit_pack_count(
    has_pack: bool,
    fixed_params: usize,
    explicit_pack: usize,
    args: usize,
    pos: ExprPos,
) -> Result<(), CompileError> {
    if has_pack && args < fixed_params + explicit_pack {
        return Err(CompileError::new(
            format!(
                "too few arguments for function template: expected at least {}, found {}",
                fixed_params + explicit_pack,
                args
            ),
            pos.line,
            pos.col,
        ));
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
    // A function parameter pack before the deduced pack is a non-deduced
    // context: it contributes no arguments, so only fixed parameters consume
    // positions in the call.
    let fixed_params: Vec<_> = func_decl.params[..non_pack_count]
        .iter()
        .filter(|p| !p.is_pack)
        .collect();
    validate_arg_counts(
        has_pack,
        &fixed_params,
        func_decl.params.len(),
        arg_types.len(),
        pos,
    )?;

    let pack_index = template_params.iter().position(|p| p.is_pack);
    let mut deduced: HashMap<String, TemplateArg> = HashMap::new();
    let mut explicit_pack = Vec::new();
    insert_explicit_template_args(
        template_params,
        explicit_args,
        &mut deduced,
        &mut explicit_pack,
    );
    validate_explicit_pack_count(
        has_pack,
        fixed_params.len(),
        explicit_pack.len(),
        arg_types.len(),
        pos,
    )?;

    for (i, p) in fixed_params.iter().enumerate() {
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
        if type_has_template_param(&p.ty) {
            deduce_type(&p.ty, &adjusted_arg_ty, &mut deduced, is_lval, pos)?;
        }
    }

    let mut result = deduced;

    if has_pack {
        let (pack_name, deduced_pack) = deduce_pack_arguments(
            &func_decl.params[non_pack_count],
            template_params,
            call_args,
            fixed_params.len() + explicit_pack.len(),
            pos,
        )?;
        let mut pack_types = explicit_pack;
        pack_types.extend(deduced_pack);
        result.insert(pack_name, TemplateArg::TypePack(pack_types));
    } else if let Some(index) = pack_index
        && !explicit_pack.is_empty()
    {
        result.insert(
            template_params[index].name.clone(),
            TemplateArg::TypePack(explicit_pack),
        );
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
            let subst_ty = crate::sema::template_subst::substitute_type(def_ty, &subst_map, None)?;
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
        let is_nontype_arg = matches!(arg, Type::NonType(_) | Type::TypeParam(_))
            || matches!(arg, Type::DependentNonType(s) if s.starts_with(crate::sema::pack_expansion::ADDRESS_ARG_PREFIX));
        if param.param_type.is_some() && !is_nontype_arg {
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
    for (param, arg) in template_params[..non_pack_count]
        .iter()
        .zip(effective_args.iter())
    {
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
