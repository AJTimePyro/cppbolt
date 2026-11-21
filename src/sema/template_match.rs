//! Partial specialization pattern matching and ranking.

use super::pack_expansion::TemplateArg;
use super::template_deduce::{
    build_template_arg_map, fill_default_template_args, parse_types_list_with_params,
    split_template_call_name,
};
use super::template_match_args::{calculate_specificity, match_pattern_args};
use super::template_match_confirm::{ConfirmCtx, confirm_resolved_pattern};
use super::*;
use crate::ast::template::{TemplateClass, TemplateParam};
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::name::QualifiedName;
use std::collections::{HashMap, HashSet};

pub(super) type MatchedPartial = (TemplateClass, HashMap<String, TemplateArg>);
pub(super) type AliasMap = HashMap<String, (Vec<TemplateParam>, Type)>;

#[derive(Debug, Clone)]
pub struct PartialSpec {
    pub pattern_args: Vec<Type>,
    pub template_class: TemplateClass,
    /// Params must all be deduced for a candidate to be viable. A param used
    /// only in a non-deduced context cannot be bound and makes the
    /// specialization unusable, as in C++.
    pub params: Vec<TemplateParam>,
}

pub(super) fn pattern_needs_confirmation(ty: &Type, aliases: &AliasMap) -> bool {
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
pub(super) fn alias_definition<'a>(
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
pub(super) fn expand_alias_pattern(pattern: &Type, aliases: &AliasMap) -> Option<Type> {
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
pub(super) fn collect_deducible_params(
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
                let mut ctx = ConfirmCtx {
                    bindings: &bindings,
                    probe: &mut probe,
                    pos,
                };
                for (pattern, concrete) in spec.pattern_args.iter().zip(args.iter()) {
                    if !confirm_resolved_pattern(pattern, concrete, self, &mut ctx)? {
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
            member_aliases: Vec::new(),
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
            alias: None,
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
