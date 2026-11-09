//! Partial specialization pattern matching and ranking.

use super::template_deduce::{
    parse_types_list_with_params, split_template_call_name,
};
use super::*;
use crate::ast::template::{TemplateClass, TemplateParam};
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::name::QualifiedName;
use std::collections::HashMap;

type MatchedPartial = (TemplateClass, HashMap<String, Type>);

#[derive(Debug, Clone)]
pub struct PartialSpec {
    pub pattern_args: Vec<Type>,
    pub template_class: TemplateClass,
}

pub fn match_pattern_arg(
    pattern: &Type,
    concrete: &Type,
    bindings: &mut HashMap<String, Type>,
) -> bool {
    match pattern {
        Type::TypeParam(p) => {
            if let Some(existing) = bindings.get(p) {
                existing == concrete
            } else {
                bindings.insert(p.clone(), concrete.clone());
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
                if p_name != c_name || p_args.len() != c_args.len() {
                    return false;
                }
                p_args
                    .iter()
                    .zip(c_args.iter())
                    .all(|(pa, ca)| match_pattern_arg(pa, ca, bindings))
            }
            _ => false,
        },
        _ => pattern == concrete,
    }
}

pub fn calculate_specificity(pattern: &Type) -> usize {
    match pattern {
        Type::TypeParam(_) => 0,
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
            if spec.pattern_args.len() != args.len() {
                continue;
            }
            let mut bindings = HashMap::new();
            if spec
                .pattern_args
                .iter()
                .zip(args.iter())
                .all(|(p, a)| match_pattern_arg(p, a, &mut bindings))
            {
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
                format!("ambiguous partial specializations for '{}<{}>'", name, canon),
                pos.0,
                pos.1,
            ));
        }
        let best = matches.remove(0);
        Ok(Some((best.0.template_class.clone(), best.1)))
    }
}
