//! Partial specialization pattern matching and ranking.

use super::pack_expansion::TemplateArg;
use super::template_deduce::{
    build_template_arg_map, fill_default_template_args, parse_types_list_with_params,
    split_template_call_name,
};
use super::*;
use crate::ast::template::{TemplateClass, TemplateParam};
use crate::ast::types::Type;
use crate::diagnostics::{CompileError, ErrorKind};
use crate::name::QualifiedName;
use std::collections::{HashMap, HashSet};

type MatchedPartial = (TemplateClass, HashMap<String, TemplateArg>);
type AliasMap = HashMap<String, (Vec<TemplateParam>, Type)>;

#[derive(Debug, Clone)]
pub struct PartialSpec {
    pub pattern_args: Vec<Type>,
    pub template_class: TemplateClass,
    /// Params must all be deduced for a candidate to be viable. A param used
    /// only in a non-deduced context cannot be bound and makes the
    /// specialization unusable, as in C++.
    pub params: Vec<TemplateParam>,
}

fn pattern_needs_confirmation(ty: &Type, aliases: &AliasMap) -> bool {
    match ty {
        Type::Struct(name) if name.contains("decltype") => true,
        Type::Decltype { .. } => true,
        Type::Const(inner)
        | Type::Pointer(inner)
        | Type::Reference(inner)
        | Type::RvalueReference(inner)
        | Type::PackExpansion(inner) => pattern_needs_confirmation(inner, aliases),
        Type::TemplateInstance { name, args } => {
            alias_definition(name, aliases).is_some()
                || args.iter().any(|a| pattern_needs_confirmation(a, aliases))
        }
        _ => false,
    }
}

/// Looks up an alias template by the spelling used in a pattern, accepting both
/// the qualified and unqualified forms.
fn alias_definition<'a>(
    name: &str,
    aliases: &'a AliasMap,
) -> Option<&'a (Vec<TemplateParam>, Type)> {
    aliases
        .get(name)
        .or_else(|| aliases.get(QualifiedName::new(name).plain()))
}

/// Expands a top-level alias template in a pattern into its target type with
/// the pattern arguments substituted in. C++ treats alias templates as
/// transparent, so `S<Ptr<T>>` deduces `T` from the expansion `T*` instead of
/// treating the alias as a non-deduced context.
///
/// Qualified names that still mention a pattern parameter (for example
/// `typename T::type`) are marked `DependentNested`. Deduction skips those
/// nodes, while confirmation resolves them after the other parameters are
/// bound, so a dependent component does not block deduction from the rest of
/// the pattern. Returns `None` only when the alias cannot be expanded.
fn expand_alias_pattern(pattern: &Type, aliases: &AliasMap) -> Option<Type> {
    let Type::TemplateInstance { name, args } = pattern else {
        return None;
    };
    let (params, target) = alias_definition(name, aliases)?;
    let effective = fill_default_template_args(name, params, args, (0, 0)).ok()?;
    let mapping = build_template_arg_map(params, &effective);
    let mut expanded = super::template_subst::substitute_type(target, &mapping, None).ok()?;
    super::template_subst_func::mark_dependent_nested(target, &mut expanded, &mapping);
    Some(expanded)
}

/// Collects template parameters that appear in a deduced context of a
/// partial-specialization pattern. This mirrors `match_pattern_arg`: a plain
/// parameter, a wrapper around one, template arguments, and a bare non-type
/// parameter deduce; qualified names (`typename T::type`), `decltype`, and
/// dependent expressions do not. Alias templates are expanded first so `Ptr<T>`
/// deduces `T` even though `void_t<typename T::type>` does not.
fn collect_deducible_params(
    ty: &Type,
    aliases: &AliasMap,
    param_names: &HashSet<String>,
    out: &mut HashSet<String>,
) {
    match ty {
        Type::TypeParam(name) => {
            out.insert(name.clone());
        }
        Type::Const(inner)
        | Type::Pointer(inner)
        | Type::Reference(inner)
        | Type::RvalueReference(inner)
        | Type::PackExpansion(inner)
        | Type::Aligned(inner, _)
        | Type::Array(inner, _) => {
            collect_deducible_params(inner, aliases, param_names, out);
        }
        Type::DependentArray(inner, bound) => {
            // `T[N]` deduces the bound N as well as the element type.
            if param_names.contains(bound.as_str()) {
                out.insert(bound.clone());
            }
            collect_deducible_params(inner, aliases, param_names, out);
        }
        Type::TemplateInstance { .. } => {
            if let Some(expanded) = expand_alias_pattern(ty, aliases) {
                collect_deducible_params(&expanded, aliases, param_names, out);
            } else if let Type::TemplateInstance { args, .. } = ty {
                for arg in args {
                    collect_deducible_params(arg, aliases, param_names, out);
                }
            }
        }
        // A bare non-type parameter is deduced; an expression that only
        // mentions parameters (for example `N + 1`) is not.
        Type::DependentNonType(text) if param_names.contains(text.as_str()) => {
            out.insert(text.clone());
        }
        Type::Function {
            return_type,
            param_types,
            ..
        } => {
            collect_deducible_params(return_type, aliases, param_names, out);
            for param in param_types {
                collect_deducible_params(param, aliases, param_names, out);
            }
        }
        Type::MemberPointer {
            class_name,
            member_type,
        }
        | Type::BoundMemberFunction {
            class_name,
            member_type,
        } => {
            // The class of a pointer to member is a deduced context.
            if param_names.contains(class_name.as_str()) {
                out.insert(class_name.clone());
            }
            collect_deducible_params(member_type, aliases, param_names, out);
        }
        // `Struct`/`Enum` names, `DependentNested`, `Decltype`, constants and
        // other dependent leaves cannot deduce a parameter by structure.
        _ => {}
    }
}

/// Confirms a pattern against a concrete argument after deduction. Only
/// `decltype` nodes need work: they are non-deduced contexts, so substitute the
/// bindings, resolve them, and compare the result. Everything else was already
/// checked structurally by `match_pattern_arg`.
fn confirm_resolved_pattern(
    pattern: &Type,
    concrete: &Type,
    bindings: &HashMap<String, TemplateArg>,
    sema: &mut Sema,
    probe: &mut Option<Sema>,
    pos: (usize, usize),
) -> Result<bool, CompileError> {
    match pattern {
        Type::Decltype {
            expr,
            parenthesized,
        } => {
            let original = Type::Decltype {
                expr: expr.clone(),
                parenthesized: *parenthesized,
            };
            let mut substituted =
                super::template_subst::substitute_type(&original, bindings, None)?;
            super::template_subst_func::mark_dependent_nested(
                &original,
                &mut substituted,
                bindings,
            );
            // `resolve_decltype_type` analyzes the operand on its own clone and
            // never mutates `sema`, so no extra probe copy is needed here.
            let Type::Decltype {
                expr: resolved_expr,
                parenthesized,
            } = &substituted
            else {
                return Ok(false);
            };
            match sema.resolve_decltype_type(resolved_expr, *parenthesized, pos.0, pos.1) {
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
        Type::Struct(name) if name.contains("decltype") => {
            // The qualified name was built by stringifying its template
            // arguments, including the `decltype` operand. Substitute the
            // deduced bindings into that spelling, then resolve the concrete
            // member type in the disposable probe so a missing nested type is
            // a substitution failure rather than a hard error.
            let substituted = super::template_name::substitute_template_name(name, bindings);
            // An operand that could not be rendered as source cannot be
            // confirmed. Rejecting it as a substitution failure would silently
            // select the primary, so surface a hard error instead.
            if substituted.contains(crate::ast::expr::UNRENDERABLE_EXPR) {
                return Err(CompileError::new(
                    "unsupported expression in dependent decltype pattern",
                    pos.0,
                    pos.1,
                ));
            }
            let mut resolved = Type::Struct(substituted);
            let probe = probe.get_or_insert_with(|| sema.clone());
            match probe.resolve_and_instantiate_type(&mut resolved, pos.0, pos.1) {
                Ok(()) => Ok(&resolved == concrete),
                Err(error) if error.kind == ErrorKind::Substitution => Ok(false),
                Err(error) => Err(error),
            }
        }
        Type::TemplateInstance {
            name: p_name,
            args: p_args,
        } => {
            if sema.lookup_alias_template(p_name).is_some() {
                // Substitute the deduced bindings into the alias arguments,
                // resolve them (an invalid dependent type rejects the
                // candidate), then expand the alias and compare. The caller
                // supplies one disposable clone per candidate, so a rejected
                // candidate leaves no semantic state behind.
                let probe = probe.get_or_insert_with(|| sema.clone());
                let mut resolved_args = Vec::new();
                for arg in p_args {
                    // Expand pack arguments before resolving so a `Ts...` in
                    // the alias pattern becomes one argument per element.
                    let expanded = match super::pack_expansion::expand_type_list(
                        std::slice::from_ref(arg),
                        bindings,
                        pos,
                    ) {
                        Ok(expanded) => expanded,
                        Err(error) if error.kind == ErrorKind::Substitution => return Ok(false),
                        Err(error) => return Err(error),
                    };
                    for mut resolved in expanded {
                        super::template_subst_func::mark_dependent_nested(
                            arg,
                            &mut resolved,
                            bindings,
                        );
                        if let Err(error) =
                            probe.resolve_and_instantiate_type(&mut resolved, pos.0, pos.1)
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
                return match probe.expand_alias_template(p_name, &resolved_args, pos.0, pos.1) {
                    Ok(Some(expanded)) => {
                        // A concrete argument recovered from a template-id string
                        // can be a `TemplateInstance` while the expansion is an
                        // instantiated `Struct`. Normalize it the same way the
                        // alias result was before comparing the two.
                        let mut canonical = concrete.clone();
                        if probe
                            .resolve_and_instantiate_type(&mut canonical, pos.0, pos.1)
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
                };
            }
            let c_args: Vec<Type> = match concrete {
                Type::TemplateInstance { name, args } => {
                    if !QualifiedName::new(p_name).matches(QualifiedName::new(name)) {
                        return Ok(false);
                    }
                    args.clone()
                }
                Type::Struct(sname) => {
                    let Some((base, args_str)) = split_template_call_name(sname) else {
                        return Ok(false);
                    };
                    if !QualifiedName::new(p_name).matches(QualifiedName::new(base)) {
                        return Ok(false);
                    }
                    let Ok(args) = super::template_deduce::parse_types_list(
                        args_str,
                        std::slice::from_ref(p_name),
                    ) else {
                        return Ok(false);
                    };
                    args
                }
                _ => return Ok(false),
            };
            let fixed = p_args.len().saturating_sub(usize::from(
                p_args
                    .last()
                    .is_some_and(|a| matches!(a, Type::PackExpansion(_))),
            ));
            for (p, c) in p_args[..fixed].iter().zip(c_args.iter()) {
                if !confirm_resolved_pattern(p, c, bindings, sema, probe, pos)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Type::Const(p) => match concrete {
            Type::Const(c) => confirm_resolved_pattern(p, c, bindings, sema, probe, pos),
            _ => Ok(false),
        },
        Type::Pointer(p) => match concrete {
            Type::Pointer(c) => confirm_resolved_pattern(p, c, bindings, sema, probe, pos),
            _ => Ok(false),
        },
        Type::Reference(p) => match concrete {
            Type::Reference(c) => confirm_resolved_pattern(p, c, bindings, sema, probe, pos),
            _ => Ok(false),
        },
        Type::RvalueReference(p) => match concrete {
            Type::RvalueReference(c) => confirm_resolved_pattern(p, c, bindings, sema, probe, pos),
            _ => Ok(false),
        },
        // Leaves and packs were already confirmed structurally.
        _ => Ok(true),
    }
}

pub fn match_pattern_arg(
    pattern: &Type,
    concrete: &Type,
    bindings: &mut HashMap<String, TemplateArg>,
    aliases: &AliasMap,
) -> bool {
    // Expand alias templates so their arguments are deducible. A dependent
    // expansion is left as a non-deduced context and confirmed after deduction
    // in `confirm_resolved_pattern`.
    if let Some(expanded) = expand_alias_pattern(pattern, aliases) {
        return match_pattern_arg(&expanded, concrete, bindings, aliases);
    }
    if let Type::TemplateInstance { name, .. } = pattern
        && alias_definition(name, aliases).is_some()
    {
        return true;
    }
    match pattern {
        Type::TypeParam(p) => {
            let arg = TemplateArg::from(concrete.clone());
            if let Some(existing) = bindings.get(p) {
                existing == &arg
            } else {
                bindings.insert(p.clone(), arg);
                true
            }
        }
        Type::Const(p_inner) => match concrete {
            Type::Const(c_inner) => match_pattern_arg(p_inner, c_inner, bindings, aliases),
            _ => false,
        },
        Type::Pointer(p_inner) => match concrete {
            Type::Pointer(c_inner) => match_pattern_arg(p_inner, c_inner, bindings, aliases),
            _ => false,
        },
        Type::Reference(p_inner) => match concrete {
            Type::Reference(c_inner) => match_pattern_arg(p_inner, c_inner, bindings, aliases),
            _ => false,
        },
        Type::RvalueReference(p_inner) => match concrete {
            Type::RvalueReference(c_inner) => {
                match_pattern_arg(p_inner, c_inner, bindings, aliases)
            }
            _ => false,
        },
        Type::Function {
            return_type: p_ret,
            param_types: p_params,
            qualifiers: p_qual,
        } => match concrete {
            Type::Function {
                return_type: c_ret,
                param_types: c_params,
                qualifiers: c_qual,
            } => {
                // cv/ref/noexcept are part of the function type. Ignoring them
                // would let a `const`/`noexcept` pattern match a plain function.
                p_qual == c_qual
                    && match_pattern_arg(p_ret, c_ret, bindings, aliases)
                    && match_pattern_args(p_params, c_params, bindings, aliases)
            }
            _ => false,
        },
        Type::MemberPointer {
            class_name: p_class,
            member_type: p_member,
        } => match concrete {
            Type::MemberPointer {
                class_name: c_class,
                member_type: c_member,
            } => {
                // An unqualified class name in the pattern is a deduced type
                // parameter (the `C` in `T C::*`), mirroring function-template
                // deduction for pointers to members.
                if !p_class.contains("::") {
                    let arg = TemplateArg::Type(Type::Struct(c_class.clone()));
                    if bindings.get(p_class).is_some_and(|e| e != &arg) {
                        return false;
                    }
                    bindings.insert(p_class.clone(), arg);
                } else if p_class != c_class {
                    return false;
                }
                match_pattern_arg(p_member, c_member, bindings, aliases)
            }
            _ => false,
        },
        Type::Array(p_inner, p_n) => match concrete {
            // `T[]` (the unbounded marker) matches any array bound; a written
            // bound, including `T[0]`, matches only that exact bound.
            Type::Array(c_inner, c_n)
                if *p_n == crate::ast::types::UNBOUNDED_ARRAY || *p_n == *c_n =>
            {
                match_pattern_arg(p_inner, c_inner, bindings, aliases)
            }
            _ => false,
        },
        Type::DependentArray(p_inner, bound) => match concrete {
            Type::Array(c_inner, c_n) => {
                let arg = TemplateArg::NonType(*c_n as i128);
                if bindings.get(bound).is_some_and(|e| e != &arg) {
                    return false;
                }
                bindings.insert(bound.clone(), arg);
                match_pattern_arg(p_inner, c_inner, bindings, aliases)
            }
            _ => false,
        },
        Type::TemplateInstance {
            name: p_name,
            args: p_args,
        } => match concrete {
            Type::TemplateInstance {
                name: c_name,
                args: c_args,
            } => {
                if !QualifiedName::new(p_name).matches(QualifiedName::new(c_name)) {
                    return false;
                }
                match_pattern_args(p_args, c_args, bindings, aliases)
            }
            Type::Struct(sname) => {
                if let Some((s_base, s_args_str)) = split_template_call_name(sname)
                    && QualifiedName::new(p_name).matches(QualifiedName::new(s_base))
                    && let Ok(s_args) = super::template_deduce::parse_types_list(
                        s_args_str,
                        std::slice::from_ref(p_name),
                    )
                {
                    match_pattern_args(p_args, &s_args, bindings, aliases)
                } else {
                    false
                }
            }
            _ => false,
        },
        // A `decltype` operand, a qualified type that still mentions a
        // template parameter, or a qualified type whose template arguments
        // contain a `decltype` is a non-deduced context. It cannot bind
        // parameters by structure; `select_partial_specialization` resolves it
        // after the other arguments have been deduced.
        Type::Decltype { .. } | Type::DependentNested(_) => true,
        Type::Struct(name) if name.contains("decltype") => true,
        _ => pattern == concrete,
    }
}

pub fn match_pattern_args(
    patterns: &[Type],
    concrete_args: &[Type],
    bindings: &mut HashMap<String, TemplateArg>,
    aliases: &AliasMap,
) -> bool {
    let has_pack = patterns
        .last()
        .is_some_and(|p| matches!(p, Type::PackExpansion(_)));
    if !has_pack {
        if patterns.len() != concrete_args.len() {
            return false;
        }
        return patterns
            .iter()
            .zip(concrete_args.iter())
            .all(|(p, c)| match_pattern_arg(p, c, bindings, aliases));
    }
    let num_fixed = patterns.len().saturating_sub(1);
    if concrete_args.len() < num_fixed {
        return false;
    }
    for i in 0..num_fixed {
        if !match_pattern_arg(&patterns[i], &concrete_args[i], bindings, aliases) {
            return false;
        }
    }
    let Some(Type::PackExpansion(inner)) = patterns.last() else {
        return false;
    };
    let trailing = &concrete_args[num_fixed..];
    match_trailing_pack(inner, trailing, bindings, aliases)
}

fn match_trailing_pack(
    inner_pattern: &Type,
    trailing: &[Type],
    bindings: &mut HashMap<String, TemplateArg>,
    aliases: &AliasMap,
) -> bool {
    if let Type::TypeParam(p) = inner_pattern {
        let is_nontype = trailing.iter().any(|t| matches!(t, Type::NonType(_)));
        let arg = if is_nontype {
            let vals: Vec<i128> = trailing
                .iter()
                .filter_map(|t| match t {
                    Type::NonType(v) => Some(*v),
                    _ => None,
                })
                .collect();
            TemplateArg::NonTypePack(vals)
        } else {
            TemplateArg::TypePack(trailing.to_vec())
        };
        bindings.insert(p.clone(), arg);
        return true;
    }
    if trailing.is_empty() {
        let mut params = Vec::new();
        collect_type_param_names(inner_pattern, &mut params);
        for p in params {
            bindings.insert(p, TemplateArg::TypePack(Vec::new()));
        }
        return true;
    }
    let mut collected: HashMap<String, Vec<Type>> = HashMap::new();
    for arg in trailing {
        let mut elem_bindings = HashMap::new();
        if !match_pattern_arg(inner_pattern, arg, &mut elem_bindings, aliases) {
            return false;
        }
        for (k, v) in elem_bindings {
            if let TemplateArg::Type(t) = v {
                collected.entry(k).or_default().push(t);
            }
        }
    }
    for (k, types) in collected {
        bindings.insert(k, TemplateArg::TypePack(types));
    }
    true
}

fn collect_type_param_names(ty: &Type, out: &mut Vec<String>) {
    match ty {
        Type::TypeParam(name) => out.push(name.clone()),
        Type::Pointer(inner)
        | Type::Reference(inner)
        | Type::RvalueReference(inner)
        | Type::Const(inner)
        | Type::PackExpansion(inner) => {
            collect_type_param_names(inner, out);
        }
        Type::TemplateInstance { args, .. } => {
            for a in args {
                collect_type_param_names(a, out);
            }
        }
        _ => {}
    }
}

pub fn calculate_specificity(pattern: &Type) -> usize {
    match pattern {
        Type::TypeParam(_) => 0,
        Type::PackExpansion(inner) => calculate_specificity(inner),
        Type::Pointer(inner) => 2 + calculate_specificity(inner),
        Type::Const(inner) => 1 + calculate_specificity(inner),
        Type::Reference(inner) | Type::RvalueReference(inner) => 2 + calculate_specificity(inner),
        // `T[]` is the least specific array pattern, `T[N]` more so, and a
        // written bound (including `T[0]`) most specific.
        Type::Array(inner, crate::ast::types::UNBOUNDED_ARRAY) => {
            2 + calculate_specificity(inner)
        }
        Type::DependentArray(inner, _) => 3 + calculate_specificity(inner),
        Type::Array(inner, _) => 4 + calculate_specificity(inner),
        Type::TemplateInstance { args, .. } => {
            4 + args.iter().map(calculate_specificity).sum::<usize>()
        }
        _ => 10,
    }
}

impl PartialSpec {
    pub fn score(&self, aliases: &AliasMap) -> usize {
        // Rank aliases by the type they expand to, not by the `TemplateInstance`
        // wrapper, so `Ptr<T>` scores like `T*` during partial ordering.
        let mut score: usize = self
            .pattern_args
            .iter()
            .map(|arg| {
                let expanded = expand_alias_pattern(arg, aliases);
                calculate_specificity(expanded.as_ref().unwrap_or(arg))
            })
            .sum();
        let fixed_count = self
            .pattern_args
            .iter()
            .filter(|a| !matches!(a, Type::PackExpansion(_)))
            .count();
        score += fixed_count * 10;
        let mut seen = std::collections::HashSet::new();
        for arg in &self.pattern_args {
            if let Type::TypeParam(p) = arg
                && !seen.insert(p)
            {
                score += 5;
            }
        }
        score
    }
}

impl Sema {
    pub fn register_partial_specialization(
        &mut self,
        decl: TemplateClass,
        params: Vec<TemplateParam>,
        pos: (usize, usize),
    ) -> Result<(), CompileError> {
        let (primary, args_str) = {
            let (p, a) = split_template_call_name(&decl.name).ok_or_else(|| {
                CompileError::new("invalid partial specialization header", pos.0, pos.1)
            })?;
            (p.to_string(), a.to_string())
        };
        let mut known: Vec<String> = self.class_templates.keys().cloned().collect();
        let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
        known.extend(param_names.clone());
        // The canonical name string is lossy for `decltype` operands, so use
        // the parsed arguments captured at parse time when available.
        let pattern_args = match decl.pattern_args.clone() {
            Some(args) => args,
            None => parse_types_list_with_params(&args_str, &known, &param_names)?,
        };
        // Every parameter of a partial specialization must be deducible from
        // the argument list, otherwise the specialization can never be used.
        // Look across all arguments so a parameter deduced in one position is
        // allowed to appear in a non-deduced context elsewhere.
        let names: HashSet<String> = param_names.iter().cloned().collect();
        let mut deducible = HashSet::new();
        for arg in &pattern_args {
            collect_deducible_params(arg, &self.alias_templates, &names, &mut deducible);
        }
        if let Some(name) = params
            .iter()
            .map(|p| &p.name)
            .find(|name| !deducible.contains(*name))
        {
            return Err(CompileError::new(
                format!(
                    "template parameter '{}' cannot be deduced from the partial specialization argument list",
                    name
                ),
                pos.0,
                pos.1,
            ));
        }
        let plain_primary = QualifiedName::new(&primary).plain().to_string();
        let spec = PartialSpec {
            pattern_args,
            template_class: decl,
            params,
        };
        self.partial_specializations
            .entry(primary.clone())
            .or_default()
            .push(spec.clone());
        if plain_primary != primary {
            self.partial_specializations
                .entry(plain_primary)
                .or_default()
                .push(spec);
        }
        Ok(())
    }

    pub fn select_partial_specialization(
        &mut self,
        name: &str,
        args: &[Type],
        pos: (usize, usize),
    ) -> Result<Option<MatchedPartial>, CompileError> {
        let Some(specs) = self.partial_specializations.get(name).cloned() else {
            return Ok(None);
        };
        let mut matches = Vec::new();
        'candidate: for spec in specs {
            let mut bindings = HashMap::new();
            if !match_pattern_args(
                &spec.pattern_args,
                args,
                &mut bindings,
                &self.alias_templates,
            ) {
                continue;
            }
            // A param reachable only through a non-deduced context is never
            // bound; the specialization is unusable, not a match.
            if spec.params.iter().any(|p| !bindings.contains_key(&p.name)) {
                continue;
            }
            // Resolve non-deduced `decltype` nodes. Resolution is speculative
            // inside `resolve_decltype_type`, so a rejected candidate leaves no
            // semantic state behind.
            if spec
                .pattern_args
                .iter()
                .any(|p| pattern_needs_confirmation(p, &self.alias_templates))
            {
                // One disposable clone per candidate, shared by all its alias
                // patterns, so rejected candidates cannot leak state and a
                // candidate with several aliases is cloned only once.
                let mut probe: Option<Sema> = None;
                for (pattern, concrete) in spec.pattern_args.iter().zip(args.iter()) {
                    if !confirm_resolved_pattern(
                        pattern, concrete, &bindings, self, &mut probe, pos,
                    )? {
                        continue 'candidate;
                    }
                }
            }
            let score = spec.score(&self.alias_templates);
            matches.push((spec, bindings, score));
        }
        if matches.is_empty() {
            return Ok(None);
        }
        matches.sort_by_key(|a| std::cmp::Reverse(a.2));
        if matches.len() > 1 && matches[0].2 == matches[1].2 {
            let canon = args
                .iter()
                .map(|a| a.to_c_type())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(CompileError::new(
                format!(
                    "ambiguous partial specializations for '{}<{}>'",
                    name, canon
                ),
                pos.0,
                pos.1,
            ));
        }
        let best = matches.remove(0);
        Ok(Some((best.0.template_class.clone(), best.1)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::Expr;
    use crate::ast::template::TemplateClass;

    fn pack_param(name: &str) -> TemplateParam {
        TemplateParam {
            name: name.to_string(),
            default_type: None,
            param_type: None,
            is_pack: true,
            line: 1,
            col: 1,
        }
    }

    fn dummy_class() -> TemplateClass {
        TemplateClass {
            name: "Pick".to_string(),
            pattern_args: None,
            is_class: false,
            is_final: false,
            base_classes: Vec::new(),
            base_access: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            friends: Vec::new(),
            static_fields: Vec::new(),
            private_members: Vec::new(),
            nested_structs: Vec::new(),
            alignment: None,
        }
    }

    fn list(arg: Type) -> Type {
        Type::TemplateInstance {
            name: "List".to_string(),
            args: vec![arg],
        }
    }

    #[test]
    fn non_deducible_partial_spec_is_not_selected() -> Result<(), Box<dyn std::error::Error>> {
        let mut sema = Sema::new();
        let decltype = Type::Decltype {
            expr: Box::new(Expr::AlignofType(Type::Struct("T::type".to_string()))),
            parenthesized: false,
        };
        sema.partial_specializations
            .entry("Pick".to_string())
            .or_default()
            .push(PartialSpec {
                pattern_args: vec![list(Type::PackExpansion(Box::new(decltype)))],
                template_class: dummy_class(),
                params: vec![pack_param("Ts")],
            });

        let concrete = list(Type::Int);
        assert!(
            sema.select_partial_specialization("Pick", std::slice::from_ref(&concrete), (1, 1))?
                .is_none()
        );

        let specs = sema
            .partial_specializations
            .get_mut("Pick")
            .ok_or("missing specialization list")?;
        specs.push(PartialSpec {
            pattern_args: vec![list(Type::PackExpansion(Box::new(Type::TypeParam(
                "Us".to_string(),
            ))))],
            template_class: dummy_class(),
            params: vec![pack_param("Us")],
        });
        let selected = sema
            .select_partial_specialization("Pick", std::slice::from_ref(&concrete), (1, 1))?
            .ok_or("viable fallback should be selected")?;
        assert!(selected.1.contains_key("Us"));
        Ok(())
    }
}
