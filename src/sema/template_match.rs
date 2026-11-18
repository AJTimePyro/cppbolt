//! Partial specialization pattern matching and ranking.

use super::pack_expansion::TemplateArg;
use super::template_deduce::{parse_types_list_with_params, split_template_call_name};
use super::*;
use crate::ast::template::{TemplateClass, TemplateParam};
use crate::ast::types::Type;
use crate::diagnostics::{CompileError, ErrorKind};
use crate::name::QualifiedName;
use std::collections::HashMap;

type MatchedPartial = (TemplateClass, HashMap<String, TemplateArg>);

#[derive(Debug, Clone)]
pub struct PartialSpec {
    pub pattern_args: Vec<Type>,
    pub template_class: TemplateClass,
    /// Params must all be deduced for a candidate to be viable. A param used
    /// only in a non-deduced context cannot be bound and makes the
    /// specialization unusable, as in C++.
    pub params: Vec<TemplateParam>,
}

fn pattern_needs_confirmation(ty: &Type, aliases: &std::collections::HashSet<String>) -> bool {
    match ty {
        Type::Decltype { .. } => true,
        Type::Const(inner)
        | Type::Pointer(inner)
        | Type::Reference(inner)
        | Type::RvalueReference(inner)
        | Type::PackExpansion(inner) => pattern_needs_confirmation(inner, aliases),
        Type::TemplateInstance { name, args } => {
            aliases.contains(QualifiedName::new(name).plain())
                || args.iter().any(|a| pattern_needs_confirmation(a, aliases))
        }
        _ => false,
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
                    let mut resolved = super::template_subst::substitute_type(arg, bindings, None)?;
                    super::template_subst_func::mark_dependent_nested(arg, &mut resolved, bindings);
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
                return match probe.expand_alias_template(p_name, &resolved_args, pos.0, pos.1) {
                    Ok(Some(expanded)) => Ok(&expanded == concrete),
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
    alias_names: &std::collections::HashSet<String>,
) -> bool {
    // An alias template in a pattern is a non-deduced context: expand it after
    // deduction in `confirm_resolved_pattern`, where it can be resolved with
    // the concrete types substituted.
    if let Type::TemplateInstance { name, .. } = pattern
        && alias_names.contains(QualifiedName::new(name).plain())
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
            Type::Const(c_inner) => match_pattern_arg(p_inner, c_inner, bindings, alias_names),
            _ => false,
        },
        Type::Pointer(p_inner) => match concrete {
            Type::Pointer(c_inner) => match_pattern_arg(p_inner, c_inner, bindings, alias_names),
            _ => false,
        },
        Type::Reference(p_inner) => match concrete {
            Type::Reference(c_inner) => match_pattern_arg(p_inner, c_inner, bindings, alias_names),
            _ => false,
        },
        Type::RvalueReference(p_inner) => match concrete {
            Type::RvalueReference(c_inner) => {
                match_pattern_arg(p_inner, c_inner, bindings, alias_names)
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
                match_pattern_args(p_args, c_args, bindings, alias_names)
            }
            Type::Struct(sname) => {
                if let Some((s_base, s_args_str)) = split_template_call_name(sname)
                    && QualifiedName::new(p_name).matches(QualifiedName::new(s_base))
                    && let Ok(s_args) = super::template_deduce::parse_types_list(
                        s_args_str,
                        std::slice::from_ref(p_name),
                    )
                {
                    match_pattern_args(p_args, &s_args, bindings, alias_names)
                } else {
                    false
                }
            }
            _ => false,
        },
        // A `decltype` operand is a non-deduced context. It cannot bind
        // parameters by structure; `select_partial_specialization` resolves it
        // after the other arguments have been deduced.
        Type::Decltype { .. } => true,
        _ => pattern == concrete,
    }
}

pub fn match_pattern_args(
    patterns: &[Type],
    concrete_args: &[Type],
    bindings: &mut HashMap<String, TemplateArg>,
    alias_names: &std::collections::HashSet<String>,
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
            .all(|(p, c)| match_pattern_arg(p, c, bindings, alias_names));
    }
    let num_fixed = patterns.len().saturating_sub(1);
    if concrete_args.len() < num_fixed {
        return false;
    }
    for i in 0..num_fixed {
        if !match_pattern_arg(&patterns[i], &concrete_args[i], bindings, alias_names) {
            return false;
        }
    }
    let Some(Type::PackExpansion(inner)) = patterns.last() else {
        return false;
    };
    let trailing = &concrete_args[num_fixed..];
    match_trailing_pack(inner, trailing, bindings, alias_names)
}

fn match_trailing_pack(
    inner_pattern: &Type,
    trailing: &[Type],
    bindings: &mut HashMap<String, TemplateArg>,
    alias_names: &std::collections::HashSet<String>,
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
        if !match_pattern_arg(inner_pattern, arg, &mut elem_bindings, alias_names) {
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
        Type::TemplateInstance { args, .. } => {
            4 + args.iter().map(calculate_specificity).sum::<usize>()
        }
        _ => 10,
    }
}

impl PartialSpec {
    pub fn score(&self) -> usize {
        let mut score: usize = self.pattern_args.iter().map(calculate_specificity).sum();
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
        let alias_names: std::collections::HashSet<String> =
            self.alias_templates.keys().cloned().collect();
        let mut matches = Vec::new();
        'candidate: for spec in specs {
            let mut bindings = HashMap::new();
            if !match_pattern_args(&spec.pattern_args, args, &mut bindings, &alias_names) {
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
                .any(|p| pattern_needs_confirmation(p, &alias_names))
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
            let score = spec.score();
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
