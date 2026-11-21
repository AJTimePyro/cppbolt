//! Confirmation of non-deduced pattern nodes after deduction.
//!
//! Only `decltype` nodes need work: they are non-deduced contexts, so
//! substitute the bindings, resolve them, and compare the result. Everything
//! else was already checked structurally by `match_pattern_arg`.

use super::pack_expansion::TemplateArg;
use super::template_deduce::split_template_call_name;
use super::*;
use crate::ast::expr::Expr;
use crate::diagnostics::ErrorKind;
use std::collections::HashMap;

/// Bindings, a disposable probe clone and the source position, shared across
/// every node confirmed for one candidate.
pub(super) struct ConfirmCtx<'a> {
    pub(super) bindings: &'a HashMap<String, TemplateArg>,
    pub(super) probe: &'a mut Option<Sema>,
    pub(super) pos: (usize, usize),
}

pub(super) fn confirm_resolved_pattern(
    pattern: &Type,
    concrete: &Type,
    sema: &mut Sema,
    ctx: &mut ConfirmCtx<'_>,
) -> Result<bool, CompileError> {
    match pattern {
        Type::Decltype {
            expr,
            parenthesized,
            ..
        } => confirm_decltype(expr, *parenthesized, concrete, sema, ctx),
        Type::Struct(name) if name.contains("decltype") => {
            confirm_stringified_decltype(name, concrete, sema, ctx)
        }
        Type::TemplateInstance { name, args } => {
            confirm_template_instance(name, args, concrete, sema, ctx)
        }
        Type::Const(p) => match concrete {
            Type::Const(c) => confirm_resolved_pattern(p, c, sema, ctx),
            _ => Ok(false),
        },
        Type::Pointer(p) => match concrete {
            Type::Pointer(c) => confirm_resolved_pattern(p, c, sema, ctx),
            _ => Ok(false),
        },
        Type::Reference(p) => match concrete {
            Type::Reference(c) => confirm_resolved_pattern(p, c, sema, ctx),
            _ => Ok(false),
        },
        Type::RvalueReference(p) => match concrete {
            Type::RvalueReference(c) => confirm_resolved_pattern(p, c, sema, ctx),
            _ => Ok(false),
        },
        // Leaves and packs were already confirmed structurally.
        _ => Ok(true),
    }
}

fn confirm_decltype(
    expr: &Expr,
    parenthesized: bool,
    concrete: &Type,
    sema: &mut Sema,
    ctx: &mut ConfirmCtx<'_>,
) -> Result<bool, CompileError> {
    let original = Type::Decltype {
        expr: Box::new(expr.clone()),
        parenthesized,
        alias: None,
    };
    let mut substituted = super::template_subst::substitute_type(&original, ctx.bindings, None)?;
    super::template_subst_func::mark_dependent_nested(&original, &mut substituted, ctx.bindings);
    // `resolve_decltype_type` analyzes the operand on its own clone and
    // never mutates `sema`, so no extra probe copy is needed here.
    let Type::Decltype {
        expr: resolved_expr,
        parenthesized,
        alias,
    } = &substituted
    else {
        return Ok(false);
    };
    match sema.resolve_decltype_type(
        resolved_expr,
        *parenthesized,
        alias.as_deref(),
        ctx.pos.0,
        ctx.pos.1,
    ) {
        Ok(resolved) => Ok(&resolved == concrete),
        Err(error) => {
            if error.kind == ErrorKind::Substitution {
                Ok(false)
            } else {
                Err(error)
            }
        }
    }
}

fn confirm_stringified_decltype(
    name: &str,
    concrete: &Type,
    sema: &mut Sema,
    ctx: &mut ConfirmCtx<'_>,
) -> Result<bool, CompileError> {
    // The qualified name was built by stringifying its template arguments,
    // including the `decltype` operand. Substitute the deduced bindings into
    // that spelling, then resolve the concrete member type in the disposable
    // probe so a missing nested type is a substitution failure rather than a
    // hard error.
    let substituted = super::template_name::substitute_template_name(name, ctx.bindings);
    // An operand that could not be rendered as source cannot be confirmed.
    // Rejecting it as a substitution failure would silently select the
    // primary, so surface a hard error instead.
    if substituted.contains(crate::ast::expr::UNRENDERABLE_EXPR) {
        return Err(CompileError::new(
            "unsupported expression in dependent decltype pattern",
            ctx.pos.0,
            ctx.pos.1,
        ));
    }
    let mut resolved = Type::Struct(substituted);
    let probe = ctx.probe.get_or_insert_with(|| {
        let mut p = sema.clone();
        p.in_substitution_context = true;
        p
    });
    match probe.resolve_and_instantiate_type(&mut resolved, ctx.pos.0, ctx.pos.1) {
        Ok(()) => Ok(&resolved == concrete),
        Err(error) if error.kind == ErrorKind::Substitution => Ok(false),
        Err(error) => Err(error),
    }
}

fn confirm_template_instance(
    name: &str,
    args: &[Type],
    concrete: &Type,
    sema: &mut Sema,
    ctx: &mut ConfirmCtx<'_>,
) -> Result<bool, CompileError> {
    if sema.lookup_alias_template(name).is_some() {
        return confirm_alias_template(name, args, concrete, sema, ctx);
    }
    confirm_template_args(name, args, concrete, sema, ctx)
}

fn confirm_alias_template(
    name: &str,
    args: &[Type],
    concrete: &Type,
    sema: &mut Sema,
    ctx: &mut ConfirmCtx<'_>,
) -> Result<bool, CompileError> {
    // Substitute the deduced bindings into the alias arguments, resolve them
    // (an invalid dependent type rejects the candidate), then expand the alias
    // and compare. The caller supplies one disposable clone per candidate, so
    // a rejected candidate leaves no semantic state behind.
    let probe = ctx.probe.get_or_insert_with(|| {
        let mut p = sema.clone();
        p.in_substitution_context = true;
        p
    });
    let mut resolved_args = Vec::new();
    for arg in args {
        // Expand pack arguments before resolving so a `Ts...` in the alias
        // pattern becomes one argument per element.
        let expanded = match super::pack_expansion::expand_type_list(
            std::slice::from_ref(arg),
            ctx.bindings,
            ctx.pos,
        ) {
            Ok(expanded) => expanded,
            Err(error) if error.kind == ErrorKind::Substitution => return Ok(false),
            Err(error) => return Err(error),
        };
        for mut resolved in expanded {
            super::template_subst_func::mark_dependent_nested(arg, &mut resolved, ctx.bindings);
            if let Err(error) =
                probe.resolve_and_instantiate_type(&mut resolved, ctx.pos.0, ctx.pos.1)
            {
                return if error.kind == ErrorKind::Substitution {
                    Ok(false)
                } else {
                    Err(error)
                };
            }
            resolved_args.push(resolved);
        }
    }
    match probe.expand_alias_template(name, &resolved_args, ctx.pos.0, ctx.pos.1) {
        Ok(Some(expanded)) => {
            // A concrete argument recovered from a template-id string can be a
            // `TemplateInstance` while the expansion is an instantiated
            // `Struct`. Normalize it the same way the alias result was before
            // comparing the two.
            let mut canonical = concrete.clone();
            if probe
                .resolve_and_instantiate_type(&mut canonical, ctx.pos.0, ctx.pos.1)
                .is_ok()
            {
                Ok(expanded == canonical)
            } else {
                Ok(&expanded == concrete)
            }
        }
        Ok(None) => Ok(false),
        Err(error) if error.kind == ErrorKind::Substitution => Ok(false),
        Err(error) => Err(error),
    }
}

fn confirm_template_args(
    name: &str,
    pattern_args: &[Type],
    concrete: &Type,
    sema: &mut Sema,
    ctx: &mut ConfirmCtx<'_>,
) -> Result<bool, CompileError> {
    let concrete_args: Vec<Type> = match concrete {
        Type::TemplateInstance { name: c_name, args } => {
            if !QualifiedName::new(name).matches(QualifiedName::new(c_name)) {
                return Ok(false);
            }
            args.clone()
        }
        Type::Struct(sname) => {
            let Some((base, args_str)) = split_template_call_name(sname) else {
                return Ok(false);
            };
            if !QualifiedName::new(name).matches(QualifiedName::new(base)) {
                return Ok(false);
            }
            let Ok(args) = super::template_deduce::parse_types_list(args_str, &[name.to_string()])
            else {
                return Ok(false);
            };
            args
        }
        _ => return Ok(false),
    };
    let fixed = pattern_args.len().saturating_sub(usize::from(
        pattern_args
            .last()
            .is_some_and(|a| matches!(a, Type::PackExpansion(_))),
    ));
    for (p, c) in pattern_args[..fixed].iter().zip(concrete_args.iter()) {
        if !confirm_resolved_pattern(p, c, sema, ctx)? {
            return Ok(false);
        }
    }
    Ok(true)
}
