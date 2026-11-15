//! Partial specialization pattern matching and ranking.

use super::pack_expansion::TemplateArg;
use super::template_deduce::{parse_types_list_with_params, split_template_call_name};
use super::*;
use crate::ast::template::{TemplateClass, TemplateParam};
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::name::QualifiedName;
use std::collections::HashMap;

type MatchedPartial = (TemplateClass, HashMap<String, TemplateArg>);

#[derive(Debug, Clone)]
pub struct PartialSpec {
    pub pattern_args: Vec<Type>,
    pub template_class: TemplateClass,
}

pub fn match_pattern_arg(
    pattern: &Type,
    concrete: &Type,
    bindings: &mut HashMap<String, TemplateArg>,
) -> bool {
    match pattern {
        Type::TypeParam(p) => {
            if let Some(existing) = bindings.get(p) {
                match existing {
                    TemplateArg::Type(ty) => ty == concrete,
                    _ => false,
                }
            } else {
                bindings.insert(p.clone(), TemplateArg::Type(concrete.clone()));
                true
            }
        }
        Type::Const(p_inner) => match concrete {
            Type::Const(c_inner) => match_pattern_arg(p_inner, c_inner, bindings),
            _ => false,
        },
        Type::Pointer(p_inner) => match concrete {
            Type::Pointer(c_inner) => match_pattern_arg(p_inner, c_inner, bindings),
            _ => false,
        },
        Type::Reference(p_inner) => match concrete {
            Type::Reference(c_inner) => match_pattern_arg(p_inner, c_inner, bindings),
            _ => false,
        },
        Type::RvalueReference(p_inner) => match concrete {
            Type::RvalueReference(c_inner) => match_pattern_arg(p_inner, c_inner, bindings),
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
                match_pattern_args(p_args, c_args, bindings)
            }
            Type::Struct(sname) => {
                if let Some((s_base, s_args_str)) = split_template_call_name(sname)
                    && QualifiedName::new(p_name).matches(QualifiedName::new(s_base))
                    && let Ok(s_args) = super::template_deduce::parse_types_list(s_args_str, std::slice::from_ref(p_name))
                {
                    match_pattern_args(p_args, &s_args, bindings)
                } else {
                    false
                }
            }
            _ => false,
        },
        _ => pattern == concrete,
    }
}

pub fn match_pattern_args(
    patterns: &[Type],
    concrete_args: &[Type],
    bindings: &mut HashMap<String, TemplateArg>,
) -> bool {
    let has_pack = patterns.last().is_some_and(|p| matches!(p, Type::PackExpansion(_)));
    if !has_pack {
        if patterns.len() != concrete_args.len() {
            return false;
        }
        return patterns
            .iter()
            .zip(concrete_args.iter())
            .all(|(p, c)| match_pattern_arg(p, c, bindings));
    }
    let num_fixed = patterns.len().saturating_sub(1);
    if concrete_args.len() < num_fixed {
        return false;
    }
    for i in 0..num_fixed {
        if !match_pattern_arg(&patterns[i], &concrete_args[i], bindings) {
            return false;
        }
    }
    let Some(Type::PackExpansion(inner)) = patterns.last() else {
        return false;
    };
    let trailing = &concrete_args[num_fixed..];
    match_trailing_pack(inner, trailing, bindings)
}

fn match_trailing_pack(
    inner_pattern: &Type,
    trailing: &[Type],
    bindings: &mut HashMap<String, TemplateArg>,
) -> bool {
    if let Type::TypeParam(p) = inner_pattern {
        bindings.insert(p.clone(), TemplateArg::TypePack(trailing.to_vec()));
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
        if !match_pattern_arg(inner_pattern, arg, &mut elem_bindings) {
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
        let pattern_args = parse_types_list_with_params(&args_str, &known, &param_names)?;
        let plain_primary = QualifiedName::new(&primary).plain().to_string();
        let spec = PartialSpec {
            pattern_args,
            template_class: decl,
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
        &self,
        name: &str,
        args: &[Type],
        pos: (usize, usize),
    ) -> Result<Option<MatchedPartial>, CompileError> {
        let candidates = self.partial_specializations.get(name);
        let Some(specs) = candidates else {
            return Ok(None);
        };
        let mut matches = Vec::new();
        for spec in specs {
            let mut bindings = HashMap::new();
            if match_pattern_args(&spec.pattern_args, args, &mut bindings) {
                matches.push((spec, bindings, spec.score()));
            }
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
