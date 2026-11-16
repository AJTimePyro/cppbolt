//! Type-level deduction logic for function templates and partial ordering.

use crate::ast::expr::ExprPos;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::name::QualifiedName;
use crate::sema::pack_expansion::TemplateArg;
use crate::sema::template_deduce::{parse_types_list, split_template_call_name};
use std::collections::HashMap;

pub fn type_has_template_param(ty: &Type) -> bool {
    match ty {
        Type::TypeParam(_) => true,
        Type::Pointer(inner) | Type::Reference(inner) | Type::RvalueReference(inner)
        | Type::Const(inner) | Type::PackExpansion(inner) | Type::Array(inner, _) => {
            type_has_template_param(inner)
        }
        Type::TemplateInstance { args, .. } => args.iter().any(type_has_template_param),
        Type::Function { return_type, param_types } => {
            type_has_template_param(return_type) || param_types.iter().any(type_has_template_param)
        }
        Type::MemberPointer { class_name, member_type } => {
            !class_name.contains("::") || type_has_template_param(member_type)
        }
        _ => false,
    }
}

fn collect_type_param_names(ty: &Type, out: &mut Vec<String>) {
    match ty {
        Type::TypeParam(name) => out.push(name.clone()),
        Type::Pointer(inner) | Type::Reference(inner) | Type::RvalueReference(inner)
        | Type::Const(inner) | Type::PackExpansion(inner) | Type::Array(inner, _) => {
            collect_type_param_names(inner, out);
        }
        Type::TemplateInstance { args, .. } => {
            for a in args { collect_type_param_names(a, out); }
        }
        Type::Function { return_type, param_types } => {
            collect_type_param_names(return_type, out);
            for p in param_types { collect_type_param_names(p, out); }
        }
        Type::MemberPointer { class_name, member_type } => {
            if !class_name.contains("::") {
                out.push(class_name.clone());
            }
            collect_type_param_names(member_type, out);
        }
        _ => {}
    }
}

fn deduce_reference(
    inner: &Type,
    arg_ty: &Type,
    deduced: &mut HashMap<String, TemplateArg>,
    is_lvalue: bool,
    pos: ExprPos,
) -> Result<(), CompileError> {
    if let Type::Const(tparam) = inner
        && let Type::TypeParam(name) = &**tparam
    {
        let deduced_arg = arg_ty.strip_reference().strip_const().clone();
        record_deduced(name, TemplateArg::Type(deduced_arg), deduced, pos)?;
        return Ok(());
    }
    if let Type::TypeParam(name) = inner {
        if !is_lvalue {
            return Err(CompileError::new(
                format!(
                    "cannot bind non-const lvalue reference of type '{}&' to an rvalue",
                    name
                ),
                pos.line,
                pos.col,
            ));
        }
        let deduced_arg = arg_ty.strip_reference().clone();
        record_deduced(name, TemplateArg::Type(deduced_arg), deduced, pos)?;
        return Ok(());
    }
    deduce_type(inner, arg_ty.strip_reference(), deduced, is_lvalue, pos)
}

fn deduce_rvalue_reference(
    inner: &Type,
    arg_ty: &Type,
    deduced: &mut HashMap<String, TemplateArg>,
    is_lvalue: bool,
    pos: ExprPos,
) -> Result<(), CompileError> {
    if let Type::TypeParam(name) = inner {
        let deduced_arg = if is_lvalue {
            Type::Reference(Box::new(arg_ty.strip_reference().clone()))
        } else {
            arg_ty.strip_reference().clone()
        };
        record_deduced(name, TemplateArg::Type(deduced_arg), deduced, pos)?;
        return Ok(());
    }
    if is_lvalue {
        return Err(CompileError::new(
            "cannot bind rvalue reference to an lvalue".to_string(),
            pos.line,
            pos.col,
        ));
    }
    deduce_type(inner, arg_ty.strip_reference(), deduced, is_lvalue, pos)
}

pub fn deduce_type(
    param_ty: &Type,
    arg_ty: &Type,
    deduced: &mut HashMap<String, TemplateArg>,
    is_lvalue: bool,
    pos: ExprPos,
) -> Result<(), CompileError> {
    match param_ty {
        Type::TypeParam(name) => {
            let deduced_arg = arg_ty.strip_reference().strip_const().clone();
            record_deduced(name, TemplateArg::Type(deduced_arg), deduced, pos)?;
        }
        Type::Reference(inner) => {
            deduce_reference(inner, arg_ty, deduced, is_lvalue, pos)?;
        }
        Type::RvalueReference(inner) => {
            deduce_rvalue_reference(inner, arg_ty, deduced, is_lvalue, pos)?;
        }
        Type::Const(inner) => {
            deduce_type(inner, arg_ty.strip_const(), deduced, is_lvalue, pos)?;
        }
        Type::Pointer(inner) => {
            let stripped = arg_ty.strip_reference().strip_const();
            match stripped {
                Type::Pointer(arg_inner) => {
                    deduce_type(inner, arg_inner, deduced, true, pos)?;
                }
                Type::Function { .. } if matches!(&**inner, Type::Function { .. }) => {
                    deduce_type(inner, stripped, deduced, true, pos)?;
                }
                _ => {
                    return Err(CompileError::new(
                        "type mismatch in deduction: expected pointer".to_string(),
                        pos.line,
                        pos.col,
                    ));
                }
            }
        }
        Type::MemberPointer {
            class_name,
            member_type,
        } => {
            deduce_member_pointer(class_name, member_type, arg_ty, deduced, pos)?;
        }
        Type::Array(inner, size) => {
            if let Type::Array(arg_inner, arg_size) = arg_ty.strip_reference().strip_const()
                && size == arg_size
            {
                deduce_type(inner, arg_inner, deduced, true, pos)?;
            } else {
                return Err(CompileError::new(
                    "type mismatch in deduction: expected array".to_string(),
                    pos.line,
                    pos.col,
                ));
            }
        }
        Type::TemplateInstance { name, args } => {
            deduce_template_instance(name, args, arg_ty, deduced, pos)?;
        }
        Type::Function {
            return_type,
            param_types,
        } => {
            deduce_function(return_type, param_types, arg_ty, deduced, pos)?;
        }
        _ => {}
    }
    Ok(())
}

fn deduce_member_pointer(
    class_name: &str,
    member_type: &Type,
    arg_ty: &Type,
    deduced: &mut HashMap<String, TemplateArg>,
    pos: ExprPos,
) -> Result<(), CompileError> {
    let stripped = arg_ty.strip_reference().strip_const();
    if let Type::MemberPointer { class_name: arg_class, member_type: arg_member } = stripped {
        if deduced.contains_key(class_name) || !class_name.contains("::") {
            let _ = deduce_type(
                &Type::TypeParam(class_name.to_string()),
                &Type::Struct(arg_class.clone()),
                deduced,
                true,
                pos,
            );
        }
        deduce_type(member_type, arg_member, deduced, true, pos)
    } else {
        Err(CompileError::new(
            "type mismatch in deduction: expected pointer-to-member".to_string(),
            pos.line,
            pos.col,
        ))
    }
}

fn deduce_function(
    param_ret: &Type,
    param_params: &[Type],
    arg_ty: &Type,
    deduced: &mut HashMap<String, TemplateArg>,
    pos: ExprPos,
) -> Result<(), CompileError> {
    let stripped = arg_ty.strip_reference().strip_const();
    let (arg_ret, arg_params) = match stripped {
        Type::Function {
            return_type,
            param_types,
        } => (&**return_type, param_types.as_slice()),
        Type::Pointer(inner) => match &**inner {
            Type::Function {
                return_type,
                param_types,
            } => (&**return_type, param_types.as_slice()),
            _ => {
                return Err(CompileError::new(
                    "type mismatch in deduction: expected function".to_string(),
                    pos.line,
                    pos.col,
                ));
            }
        },
        _ => {
            return Err(CompileError::new(
                "type mismatch in deduction: expected function".to_string(),
                pos.line,
                pos.col,
            ));
        }
    };
    deduce_type(param_ret, arg_ret, deduced, false, pos)?;
    let has_pack = param_params.last().is_some_and(|p| matches!(p, Type::PackExpansion(_)));
    let non_pack_count = if has_pack {
        param_params.len().saturating_sub(1)
    } else {
        param_params.len()
    };
    if (!has_pack && arg_params.len() != non_pack_count)
        || (has_pack && arg_params.len() < non_pack_count)
    {
        return Err(CompileError::new(
            "function parameter count mismatch in deduction".to_string(),
            pos.line,
            pos.col,
        ));
    }
    for i in 0..non_pack_count {
        deduce_type(&param_params[i], &arg_params[i], deduced, false, pos)?;
    }
    if has_pack && let Some(Type::PackExpansion(inner)) = param_params.last() {
        deduce_function_pack(inner, &arg_params[non_pack_count..], deduced, pos)?;
    }
    Ok(())
}

fn deduce_function_pack(
    inner: &Type,
    pack_slice: &[Type],
    deduced: &mut HashMap<String, TemplateArg>,
    pos: ExprPos,
) -> Result<(), CompileError> {
    if let Type::TypeParam(pack_name) = inner {
        record_deduced(
            pack_name,
            TemplateArg::TypePack(pack_slice.to_vec()),
            deduced,
            pos,
        )?;
    } else {
        let mut elem_types = Vec::new();
        for arg_p in pack_slice {
            let mut elem_map = HashMap::new();
            deduce_type(inner, arg_p, &mut elem_map, false, pos)?;
            if let Some((_, TemplateArg::Type(t))) = elem_map.into_iter().next() {
                elem_types.push(t);
            }
        }
        let mut out = Vec::new();
        collect_type_param_names(inner, &mut out);
        if let Some(name) = out.first() {
            record_deduced(
                name,
                TemplateArg::TypePack(elem_types),
                deduced,
                pos,
            )?;
        }
    }
    Ok(())
}

fn extract_concrete_template_args(
    name: &str,
    arg_ty: &Type,
    pos: ExprPos,
) -> Result<Vec<Type>, CompileError> {
    let clean_arg = arg_ty.strip_reference().strip_const();
    if let Type::TemplateInstance {
        name: a_name,
        args: a_args,
    } = clean_arg
    {
        if !QualifiedName::new(name).matches(QualifiedName::new(a_name)) {
            return Err(CompileError::new(
                "type mismatch in deduction: template instance mismatch".to_string(),
                pos.line,
                pos.col,
            ));
        }
        return Ok(a_args.clone());
    } else if let Type::Struct(sname) = clean_arg
        && let Some((s_base, s_args_str)) = split_template_call_name(sname)
    {
        if !QualifiedName::new(s_base).matches(QualifiedName::new(name)) {
            return Err(CompileError::new(
                "type mismatch in deduction: template instance mismatch".to_string(),
                pos.line,
                pos.col,
            ));
        }
        let s_args = parse_types_list(s_args_str, &[name.to_string()])?;
        return Ok(s_args);
    }
    Err(CompileError::new(
        "type mismatch in deduction: expected template instance".to_string(),
        pos.line,
        pos.col,
    ))
}

fn deduce_pack_expansion(
    pack_inner: &Type,
    pack_slice: &[Type],
    deduced: &mut HashMap<String, TemplateArg>,
    pos: ExprPos,
) -> Result<(), CompileError> {
    if let Type::TypeParam(pname) = pack_inner {
        let is_nontype = pack_slice.iter().any(|t| matches!(t, Type::NonType(_)));
        let arg = if is_nontype {
            let vals: Vec<i128> = pack_slice
                .iter()
                .filter_map(|t| if let Type::NonType(v) = t { Some(*v) } else { None })
                .collect();
            TemplateArg::NonTypePack(vals)
        } else {
            TemplateArg::TypePack(pack_slice.to_vec())
        };
        record_deduced(pname, arg, deduced, pos)?;
    } else if type_has_template_param(pack_inner) {
        if pack_slice.is_empty() {
            let mut params = Vec::new();
            collect_type_param_names(pack_inner, &mut params);
            params.sort();
            params.dedup();
            for p in params {
                record_deduced(&p, TemplateArg::TypePack(Vec::new()), deduced, pos)?;
            }
        } else {
            let mut collected_types: HashMap<String, Vec<Type>> = HashMap::new();
            let mut collected_nontypes: HashMap<String, Vec<i128>> = HashMap::new();
            for elem in pack_slice {
                let mut elem_deduced = HashMap::new();
                deduce_type(pack_inner, elem, &mut elem_deduced, true, pos)?;
                for (k, v) in elem_deduced {
                    match v {
                        TemplateArg::Type(t) => collected_types.entry(k).or_default().push(t),
                        TemplateArg::NonType(n) => {
                            collected_nontypes.entry(k).or_default().push(n)
                        }
                        _ => {}
                    }
                }
            }
            for (k, types) in collected_types {
                record_deduced(&k, TemplateArg::TypePack(types), deduced, pos)?;
            }
            for (k, values) in collected_nontypes {
                record_deduced(&k, TemplateArg::NonTypePack(values), deduced, pos)?;
            }
        }
    }
    Ok(())
}

fn deduce_template_instance(
    name: &str,
    args: &[Type],
    arg_ty: &Type,
    deduced: &mut HashMap<String, TemplateArg>,
    pos: ExprPos,
) -> Result<(), CompileError> {
    let concrete_args = extract_concrete_template_args(name, arg_ty, pos)?;
    let pack_inner = args.last().and_then(|t| match t {
        Type::PackExpansion(inner) => Some(inner),
        _ => None,
    });

    if let Some(pack_inner) = pack_inner {
        let fixed_count = args.len() - 1;
        if concrete_args.len() < fixed_count {
            return Err(CompileError::new(
                "type mismatch in deduction: template instance mismatch".to_string(),
                pos.line,
                pos.col,
            ));
        }
        for i in 0..fixed_count {
            deduce_type(&args[i], &concrete_args[i], deduced, true, pos)?;
        }
        let pack_slice = &concrete_args[fixed_count..];
        deduce_pack_expansion(pack_inner, pack_slice, deduced, pos)?;
    } else {
        if args.len() != concrete_args.len() {
            return Err(CompileError::new(
                "type mismatch in deduction: template instance mismatch".to_string(),
                pos.line,
                pos.col,
            ));
        }
        for (p_arg, a_arg) in args.iter().zip(concrete_args.iter()) {
            deduce_type(p_arg, a_arg, deduced, true, pos)?;
        }
    }
    Ok(())
}

fn template_args_match(a: &TemplateArg, b: &TemplateArg) -> bool {
    match (a, b) {
        (TemplateArg::Type(t1), TemplateArg::Type(t2)) => {
            t1 == t2 || t1.to_c_type() == t2.to_c_type()
        }
        (TemplateArg::TypePack(p1), TemplateArg::TypePack(p2)) => {
            p1.len() == p2.len()
                && p1
                    .iter()
                    .zip(p2.iter())
                    .all(|(t1, t2)| t1 == t2 || t1.to_c_type() == t2.to_c_type())
        }
        (TemplateArg::NonType(v1), TemplateArg::NonType(v2)) => v1 == v2,
        (TemplateArg::NonTypePack(v1), TemplateArg::NonTypePack(v2)) => v1 == v2,
        _ => false,
    }
}

fn format_template_arg(arg: &TemplateArg) -> String {
    match arg {
        TemplateArg::Type(t) => t.to_c_type(),
        TemplateArg::NonType(v) => v.to_string(),
        TemplateArg::TypePack(ts) => {
            let names: Vec<_> = ts.iter().map(|t| t.to_c_type()).collect();
            names.join(", ")
        }
        TemplateArg::NonTypePack(vs) => {
            let names: Vec<_> = vs.iter().map(|v| v.to_string()).collect();
            names.join(", ")
        }
    }
}

fn record_deduced(
    name: &str,
    deduced_arg: TemplateArg,
    deduced: &mut HashMap<String, TemplateArg>,
    pos: ExprPos,
) -> Result<(), CompileError> {
    if let Some(existing) = deduced.get(name) {
        if !template_args_match(existing, &deduced_arg) {
            return Err(CompileError::new(
                format!(
                    "template argument deduction conflict for '{}': deduced '{}' and '{}'",
                    name,
                    format_template_arg(existing),
                    format_template_arg(&deduced_arg)
                ),
                pos.line,
                pos.col,
            ));
        }
    } else {
        deduced.insert(name.to_string(), deduced_arg);
    }
    Ok(())
}
