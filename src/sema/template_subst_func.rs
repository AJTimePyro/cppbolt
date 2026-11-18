//! Function-level template AST substitution.

use super::pack_expansion::{
    TemplateArg, build_slice_mapping, collect_packs_in_expr, collect_packs_in_type,
    pack_element_name, rename_pack_identifiers, validate_pack_lengths,
};
use super::template_name::substitute_template_name;
use super::template_subst::{substitute_expr, substitute_type};
use super::template_subst_stmt::substitute_stmt;
use crate::ast::FunctionQualifiers;
use crate::ast::decl::{FunctionDecl, Param};
use crate::ast::expr::Expr;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use std::collections::{HashMap, HashSet};

fn substitute_func_name(func_name: &str, injected_class: Option<(&str, &Type)>) -> String {
    if let Some((cname, target_ty)) = injected_class {
        let short_cname = cname.rsplit_once("::").map(|(_, s)| s).unwrap_or(cname);
        let base_cname = short_cname.split('<').next().unwrap_or(short_cname);
        if func_name == cname || func_name == short_cname || func_name == base_cname {
            return target_ty.to_c_type();
        }
        if func_name == format!("~{}", cname)
            || func_name == format!("~{}", short_cname)
            || func_name == format!("~{}", base_cname)
        {
            return format!("~{}", target_ty.to_c_type());
        }
    }
    func_name.to_string()
}

fn substitute_params(
    params: &[Param],
    unbound_pack_names: &HashSet<String>,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<Vec<Param>, CompileError> {
    let mut result = Vec::new();
    for p in params {
        if p.is_pack {
            let mut still_free = HashSet::new();
            collect_packs_in_type(&p.ty, unbound_pack_names, &mut still_free);
            if !still_free.is_empty() {
                result.push(Param {
                    name: p.name.clone(),
                    ty: substitute_type(&p.ty, mapping, injected_class)?,
                    default_val: None,
                    is_pack: true,
                });
                continue;
            }
            let mut known = HashSet::new();
            collect_packs_in_type(&p.ty, &mapping.keys().cloned().collect(), &mut known);
            let pack_len = known
                .iter()
                .find_map(|k| mapping.get(k).and_then(|a| a.pack_len()))
                .unwrap_or(0);
            for i in 0..pack_len {
                let subst_map = build_slice_mapping(mapping, &known, i);
                let ty = substitute_type(&p.ty, &subst_map, injected_class)?;
                let stripped = match ty {
                    Type::PackExpansion(inner) => *inner,
                    other => other,
                };
                result.push(Param {
                    name: pack_element_name(&p.name, i),
                    ty: stripped,
                    default_val: None,
                    is_pack: false,
                });
            }
        } else {
            let mut default_val = p.default_val.clone();
            if let Some(def) = &mut default_val {
                substitute_expr(def, mapping, injected_class)?;
            }
            result.push(Param {
                name: p.name.clone(),
                ty: substitute_type(&p.ty, mapping, injected_class)?,
                default_val,
                is_pack: false,
            });
        }
    }
    Ok(result)
}

fn substitute_ctor_initializers(
    initializers: &[(String, Vec<Expr>)],
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<Vec<(String, Vec<Expr>)>, CompileError> {
    let mut result = Vec::new();
    for (m, inits) in initializers {
        if let Some(base_name) = m.strip_suffix("...") {
            let mut known = HashSet::new();
            for init in inits {
                collect_packs_in_expr(init, &mapping.keys().cloned().collect(), &mut known);
            }
            if mapping.get(base_name).is_some_and(|a| a.is_pack()) {
                known.insert(base_name.to_string());
            } else {
                for (k, v) in mapping {
                    if v.is_pack() && (base_name == k || base_name.contains(k)) {
                        known.insert(k.clone());
                    }
                }
            }
            let pack_len = validate_pack_lengths(&known, mapping, crate::ast::ExprPos::default())?;
            for i in 0..pack_len {
                let subst_map = build_slice_mapping(mapping, &known, i);
                let mut subst_inits = inits.clone();
                for init in &mut subst_inits {
                    rename_pack_identifiers(init, &known, mapping, i);
                    substitute_expr(init, &subst_map, injected_class)?;
                }
                let inst_base = if let Some(TemplateArg::Type(ty)) = mapping.get(base_name) {
                    ty.to_c_type()
                } else if let Some(TemplateArg::TypePack(types)) = mapping.get(base_name) {
                    types
                        .get(i)
                        .map(|t| t.to_c_type())
                        .unwrap_or_else(|| base_name.to_string())
                } else {
                    substitute_template_name(base_name, &subst_map)
                };
                result.push((inst_base, subst_inits));
            }
        } else {
            let mut subst_inits = inits.clone();
            let known: HashSet<String> = mapping
                .iter()
                .filter(|(_, v)| v.is_pack())
                .map(|(k, _)| k.clone())
                .collect();
            let has_unexpanded_member_pack = subst_inits.iter().any(|init| {
                if let Expr::PackExpansion { pattern, .. } = init {
                    let mut packs = HashSet::new();
                    collect_packs_in_expr(pattern, &known, &mut packs);
                    packs.is_empty()
                } else {
                    false
                }
            });
            if !has_unexpanded_member_pack {
                subst_inits = crate::sema::pack_expansion::expand_expr_list(&subst_inits, mapping)?;
            } else {
                for init in &mut subst_inits {
                    substitute_expr(init, mapping, injected_class)?;
                }
            }
            if injected_class.is_some() {
                for init in &mut subst_inits {
                    substitute_expr(init, mapping, injected_class)?;
                }
            }
            let substituted_m = if m.contains('<') {
                substitute_template_name(m, mapping)
            } else {
                m.clone()
            };
            result.push((substituted_m, subst_inits));
        }
    }
    Ok(result)
}

pub fn substitute_function(
    func: &FunctionDecl,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
    new_name: Option<String>,
) -> Result<FunctionDecl, CompileError> {
    let mut remaining_template_params = Vec::new();
    let mut active_mapping = mapping.clone();
    for p in &func.params {
        if p.is_pack {
            let mut known = HashSet::new();
            collect_packs_in_type(&p.ty, &mapping.keys().cloned().collect(), &mut known);
            if let Some(first_pack) = known.iter().next()
                && let Some(arg) = mapping.get(first_pack)
            {
                active_mapping.insert(p.name.clone(), arg.clone());
            }
        }
    }
    for p in &func.template_params {
        if injected_class.is_some() {
            remaining_template_params.push(p.clone());
            active_mapping.remove(&p.name);
        } else if !mapping.contains_key(&p.name) {
            remaining_template_params.push(p.clone());
        }
    }
    let mapping = &active_mapping;
    let unbound_pack_names: HashSet<String> = func
        .template_params
        .iter()
        .filter(|p| p.is_pack && !mapping.contains_key(&p.name))
        .map(|p| p.name.clone())
        .collect();
    let name = new_name.unwrap_or_else(|| substitute_func_name(&func.name, injected_class));
    let return_type = substitute_type(&func.return_type, mapping, injected_class)?;
    let params = substitute_params(&func.params, &unbound_pack_names, mapping, injected_class)?;
    let ctor_initializers =
        substitute_ctor_initializers(&func.ctor_initializers, mapping, injected_class)?;

    let mut body = func.body.clone();
    if let Some(body_stmt) = &mut body {
        substitute_stmt(body_stmt, mapping, injected_class)?;
    }

    Ok(FunctionDecl {
        qualifiers: FunctionQualifiers::default(),
        name,
        return_type,
        params,
        body,
        is_virtual: func.is_virtual,
        is_pure_virtual: func.is_pure_virtual,
        is_const: func.is_const,
        is_explicit: func.is_explicit,
        ref_qualifier: func.ref_qualifier,
        is_static: func.is_static,
        ctor_initializers,
        template_params: remaining_template_params,
        line: func.line,
        col: func.col,
    })
}
