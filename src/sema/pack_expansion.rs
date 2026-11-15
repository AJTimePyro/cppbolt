//! Pack expansion engine for C++11 variadic templates.

use crate::ast::expr::{Expr, ExprPos};
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use std::collections::{HashMap, HashSet};

/// Concrete or deduced template argument in substitution environments.
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateArg {
    Type(Type),
    NonType(i128),
    TypePack(Vec<Type>),
    NonTypePack(Vec<i128>),
}

impl From<Type> for TemplateArg {
    fn from(ty: Type) -> Self {
        match ty {
            Type::NonType(val) => TemplateArg::NonType(val),
            _ => TemplateArg::Type(ty),
        }
    }
}

impl TemplateArg {
    pub fn is_pack(&self) -> bool {
        matches!(self, TemplateArg::TypePack(_) | TemplateArg::NonTypePack(_))
    }

    pub fn pack_len(&self) -> Option<usize> {
        match self {
            TemplateArg::TypePack(v) => Some(v.len()),
            TemplateArg::NonTypePack(v) => Some(v.len()),
            _ => None,
        }
    }

    pub fn as_type(&self) -> Option<&Type> {
        match self {
            TemplateArg::Type(ty) => Some(ty),
            _ => None,
        }
    }
}

pub fn to_template_arg_map(mapping: &HashMap<String, Type>) -> HashMap<String, TemplateArg> {
    mapping.iter().map(|(k, v)| (k.clone(), TemplateArg::from(v.clone()))).collect()
}

pub fn pack_element_name(name: &str, i: usize) -> String {
    format!("__pack_{}_{}", name, i)
}

fn for_each_child_expr<'a, F: FnMut(&'a Expr)>(expr: &'a Expr, mut f: F) {
    match expr {
        Expr::Call { callee, args, .. } => {
            f(callee);
            args.iter().for_each(&mut f);
        }
        Expr::Binary { left, right, .. } => {
            f(left);
            f(right);
        }
        Expr::Assign { target, value, .. } | Expr::CompoundAssign { target, value, .. } => {
            f(target);
            f(value);
        }
        Expr::Unary { expr, .. } | Expr::SizeofExpr(expr) | Expr::Delete { expr, .. }
        | Expr::Cast { expr, .. } => f(expr),
        Expr::ArraySubscript { array, index, .. } => {
            f(array);
            f(index);
        }
        Expr::MemberAccess { object, .. } => f(object),
        Expr::Ternary { cond, then_expr, else_expr } => {
            f(cond);
            f(then_expr);
            f(else_expr);
        }
        Expr::InitializerList { elements, .. } | Expr::TypedInitList { elements, .. } => {
            elements.iter().for_each(&mut f);
        }
        _ => {}
    }
}

fn expr_target_type(expr: &Expr) -> Option<&Type> {
    match expr {
        Expr::Cast { target_type, .. } => Some(target_type),
        Expr::SizeofType(ty)
        | Expr::AlignofType(ty)
        | Expr::TypedInitList { ty, .. }
        | Expr::New { ty, .. } => Some(ty),
        _ => None,
    }
}

pub fn rename_pack_identifiers(
    expr: &mut Expr,
    packs: &HashSet<String>,
    mapping: &HashMap<String, TemplateArg>,
    index: usize,
) {
    match expr {
        Expr::Identifier(name) if packs.contains(name) && matches!(mapping.get(name), Some(TemplateArg::TypePack(_))) => {
            *name = pack_element_name(name, index);
        }
        Expr::Call { callee, args, .. } => {
            rename_pack_identifiers(callee, packs, mapping, index);
            args.iter_mut().for_each(|a| rename_pack_identifiers(a, packs, mapping, index));
        }
        Expr::Binary { left, right, .. } => {
            rename_pack_identifiers(left, packs, mapping, index);
            rename_pack_identifiers(right, packs, mapping, index);
        }
        Expr::Unary { expr, .. } | Expr::Cast { expr, .. } => rename_pack_identifiers(expr, packs, mapping, index),
        Expr::Assign { target, value, .. } | Expr::CompoundAssign { target, value, .. } => {
            rename_pack_identifiers(target, packs, mapping, index);
            rename_pack_identifiers(value, packs, mapping, index);
        }
        Expr::ArraySubscript { array, index: idx, .. } => {
            rename_pack_identifiers(array, packs, mapping, index);
            rename_pack_identifiers(idx, packs, mapping, index);
        }
        Expr::MemberAccess { object, .. } => rename_pack_identifiers(object, packs, mapping, index),
        Expr::Ternary { cond, then_expr, else_expr } => {
            rename_pack_identifiers(cond, packs, mapping, index);
            rename_pack_identifiers(then_expr, packs, mapping, index);
            rename_pack_identifiers(else_expr, packs, mapping, index);
        }
        Expr::InitializerList { elements, .. } | Expr::TypedInitList { elements, .. } => {
            elements.iter_mut().for_each(|el| rename_pack_identifiers(el, packs, mapping, index));
        }
        _ => {}
    }
}

fn scan_packs_in_type(ty: &Type, known: &HashSet<String>, expanded: bool, out: &mut Vec<(String, bool)>) {
    match ty {
        Type::PackExpansion(inner) => scan_packs_in_type(inner, known, true, out),
        Type::TypeParam(n) if known.contains(n) => out.push((n.clone(), expanded)),
        Type::TemplateInstance { args, .. } => args.iter().for_each(|a| scan_packs_in_type(a, known, expanded, out)),
        Type::Pointer(i) | Type::Reference(i) | Type::RvalueReference(i)
        | Type::Const(i) | Type::Array(i, _) | Type::Aligned(i, _) => scan_packs_in_type(i, known, expanded, out),
        Type::DependentAlignas(i, pack) => {
            scan_packs_in_type(i, known, expanded, out);
            if known.contains(pack) {
                out.push((pack.clone(), true));
            }
        }
        Type::Function { return_type, param_types } => {
            scan_packs_in_type(return_type, known, expanded, out);
            param_types.iter().for_each(|p| scan_packs_in_type(p, known, expanded, out));
        }
        _ => {}
    }
}

fn scan_packs_in_expr(expr: &Expr, known: &HashSet<String>, expanded: bool, out: &mut Vec<(String, bool)>) {
    match expr {
        Expr::PackExpansion { pattern, .. } => scan_packs_in_expr(pattern, known, true, out),
        Expr::SizeofPack { .. } => {}
        Expr::Identifier(name) if known.contains(name) => out.push((name.clone(), expanded)),
        Expr::Identifier(name) if name.contains('<') => {
            known.iter().filter(|p| name.contains(p.as_str())).for_each(|p| out.push((p.clone(), expanded)));
        }
        _ => {
            if let Some(ty) = expr_target_type(expr) {
                scan_packs_in_type(ty, known, expanded, out);
            }
            for_each_child_expr(expr, |c| scan_packs_in_expr(c, known, expanded, out));
        }
    }
}

pub fn collect_packs_in_expr(expr: &Expr, known: &HashSet<String>, out: &mut HashSet<String>) {
    let mut recorded = Vec::new();
    scan_packs_in_expr(expr, known, true, &mut recorded);
    out.extend(recorded.into_iter().map(|(p, _)| p));
}

pub fn collect_packs_in_type(ty: &Type, known: &HashSet<String>, out: &mut HashSet<String>) {
    let mut recorded = Vec::new();
    scan_packs_in_type(ty, known, true, &mut recorded);
    out.extend(recorded.into_iter().map(|(p, _)| p));
}

pub fn find_unexpanded_packs(expr: &Expr, known: &HashSet<String>) -> Vec<String> {
    let mut recorded = Vec::new();
    scan_packs_in_expr(expr, known, false, &mut recorded);
    recorded.into_iter().filter(|(_, exp)| !*exp).map(|(p, _)| p).collect()
}

pub fn find_unexpanded_packs_in_type(ty: &Type, known: &HashSet<String>) -> Vec<String> {
    let mut recorded = Vec::new();
    scan_packs_in_type(ty, known, false, &mut recorded);
    recorded.into_iter().filter(|(_, exp)| !*exp).map(|(p, _)| p).collect()
}

pub(crate) fn validate_pack_lengths(packs: &HashSet<String>, mapping: &HashMap<String, TemplateArg>, pos: ExprPos) -> Result<usize, CompileError> {
    let mut expected: Option<usize> = None;
    for pack in packs {
        let len = mapping.get(pack).and_then(|a| a.pack_len()).unwrap_or(0);
        if let Some(exp) = expected {
            if len != exp {
                return Err(CompileError::new("pack expansion contains parameter packs of different lengths", pos.line, pos.col));
            }
        } else {
            expected = Some(len);
        }
    }
    Ok(expected.unwrap_or(0))
}

pub fn build_slice_mapping(mapping: &HashMap<String, TemplateArg>, packs: &HashSet<String>, index: usize) -> HashMap<String, TemplateArg> {
    let mut subst_map = mapping.clone();
    for pack in packs {
        if let Some(arg) = mapping.get(pack) {
            let elem = pack_element_name(pack, index);
            match arg {
                TemplateArg::TypePack(types) => if let Some(ty) = types.get(index) {
                    let arg = TemplateArg::from(ty.clone());
                    subst_map.insert(pack.clone(), arg.clone());
                    subst_map.insert(elem, arg);
                },
                TemplateArg::NonTypePack(vals) => if let Some(val) = vals.get(index) {
                    subst_map.insert(pack.clone(), TemplateArg::NonType(*val));
                    subst_map.insert(elem, TemplateArg::NonType(*val));
                },
                _ => {}
            }
        }
    }
    subst_map
}

pub fn expand_expr_list(exprs: &[Expr], mapping: &HashMap<String, TemplateArg>) -> Result<Vec<Expr>, CompileError> {
    let known: HashSet<String> = mapping.iter().filter(|(_, v)| v.is_pack()).map(|(k, _)| k.clone()).collect();
    let mut result = Vec::new();
    for expr in exprs {
        match expr {
            Expr::PackExpansion { pattern, pos } => {
                let mut packs = HashSet::new();
                collect_packs_in_expr(pattern, &known, &mut packs);
                if packs.is_empty() {
                    return Err(CompileError::new("pack expansion does not contain any unexpanded parameter packs", pos.line, pos.col));
                }
                let len = validate_pack_lengths(&packs, mapping, *pos)?;
                for i in 0..len {
                    let subst_map = build_slice_mapping(mapping, &packs, i);
                    let mut copy = (**pattern).clone();
                    rename_pack_identifiers(&mut copy, &packs, mapping, i);
                    crate::sema::template_subst::substitute_expr(&mut copy, &subst_map, None)?;
                    result.push(copy);
                }
            }
            _ => {
                let unexp = find_unexpanded_packs(expr, &known);
                if let Some(first) = unexp.first() {
                    let pos = expr.pos();
                    return Err(CompileError::new(format!("parameter pack '{}' used unexpanded", first), pos.line, pos.col));
                }
                let mut copy = expr.clone();
                crate::sema::template_subst::substitute_expr(&mut copy, mapping, None)?;
                result.push(copy);
            }
        }
    }
    Ok(result)
}

pub fn expand_type_list(types: &[Type], mapping: &HashMap<String, TemplateArg>) -> Result<Vec<Type>, CompileError> {
    let known: HashSet<String> = mapping.iter().filter(|(_, v)| v.is_pack()).map(|(k, _)| k.clone()).collect();
    let mut result = Vec::new();
    for ty in types {
        match ty {
            Type::PackExpansion(inner) => {
                let mut packs = HashSet::new();
                collect_packs_in_type(inner, &known, &mut packs);
                if packs.is_empty() {
                    return Err(CompileError::new("pack expansion does not contain any unexpanded parameter packs", 0, 0));
                }
                let len = validate_pack_lengths(&packs, mapping, ExprPos::default())?;
                for i in 0..len {
                    let subst_map = build_slice_mapping(mapping, &packs, i);
                    result.push(crate::sema::template_subst::substitute_type(inner, &subst_map, None)?);
                }
            }
            _ => {
                let unexp = find_unexpanded_packs_in_type(ty, &known);
                if let Some(first) = unexp.first() {
                    return Err(CompileError::new(format!("parameter pack '{}' used unexpanded", first), 0, 0));
                }
                result.push(crate::sema::template_subst::substitute_type(ty, mapping, None)?);
            }
        }
    }
    Ok(result)
}
