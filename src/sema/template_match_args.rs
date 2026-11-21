//! Deduction over template argument lists, and specificity ranking.

use super::pack_expansion::TemplateArg;
use super::template_match::AliasMap;
use super::template_match_deduce::match_pattern_arg;
use super::*;

/// Matches a pattern argument list against a concrete one, including a
/// trailing pack expansion in the pattern.
pub(super) fn match_pattern_args(
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

/// Specificity of a pattern, used to rank competing partial specializations.
pub(super) fn calculate_specificity(pattern: &Type) -> usize {
    match pattern {
        Type::TypeParam(_) => 0,
        Type::PackExpansion(inner) => calculate_specificity(inner),
        Type::Pointer(inner) => 2 + calculate_specificity(inner),
        Type::Const(inner) => 1 + calculate_specificity(inner),
        Type::Reference(inner) | Type::RvalueReference(inner) => 2 + calculate_specificity(inner),
        // `T[]` is the least specific array pattern, `T[N]` more so, and a
        // written bound (including `T[0]`) most specific.
        Type::Array(inner, crate::ast::types::UNBOUNDED_ARRAY) => 2 + calculate_specificity(inner),
        Type::DependentArray(inner, _) => 3 + calculate_specificity(inner),
        Type::Array(inner, _) => 4 + calculate_specificity(inner),
        Type::TemplateInstance { args, .. } => {
            4 + args.iter().map(calculate_specificity).sum::<usize>()
        }
        _ => 10,
    }
}
