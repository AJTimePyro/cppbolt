//! Template AST substitution: clones and replaces template type parameters
//! with concrete argument types across types, expressions, statements, and declarations.

use crate::ast::decl::Decl;
use crate::ast::expr::Expr;
use crate::ast::template::TemplateClass;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::sema::pack_expansion::{
    collect_packs_in_expr, expand_expr_list, expand_type_list, TemplateArg,
};
pub use crate::sema::template_name::substitute_template_name;
pub use crate::sema::template_subst_func::substitute_function;
use std::collections::HashMap;

pub fn substitute_type(
    ty: &Type,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<Type, CompileError> {
    match ty {
        Type::TypeParam(name) => match mapping.get(name) {
            Some(TemplateArg::Type(t)) => Ok(t.clone()),
            Some(TemplateArg::NonType(n)) => Ok(Type::NonType(*n)),
            Some(TemplateArg::TypePack(_)) | Some(TemplateArg::NonTypePack(_)) => {
                Err(CompileError::new(
                    format!("parameter pack '{}' used unexpanded", name),
                    0,
                    0,
                ))
            }
            _ => Ok(ty.clone()),
        },
        Type::Struct(name) => Ok(substitute_struct_type(name, mapping, injected_class)),
        Type::TemplateInstance { name, args } => {
            let subst_args = expand_type_list(args, mapping)?;
            Ok(Type::TemplateInstance {
                name: name.clone(),
                args: subst_args,
            })
        }
        Type::Const(inner) => {
            let subst_inner = substitute_type(inner, mapping, injected_class)?;
            if subst_inner.is_reference() {
                Ok(subst_inner)
            } else {
                Ok(Type::Const(Box::new(subst_inner)))
            }
        }
        Type::Pointer(inner) => Ok(Type::Pointer(Box::new(substitute_type(
            inner,
            mapping,
            injected_class,
        )?))),
        Type::Reference(inner) => Ok(collapse_reference(
            false,
            substitute_type(inner, mapping, injected_class)?,
        )),
        Type::RvalueReference(inner) => Ok(collapse_reference(
            true,
            substitute_type(inner, mapping, injected_class)?,
        )),
        Type::Array(inner, size) => Ok(Type::Array(
            Box::new(substitute_type(inner, mapping, injected_class)?),
            *size,
        )),
        Type::DependentArray(inner, param_name) => {
            let subst_inner = substitute_type(inner, mapping, injected_class)?;
            let size = match mapping.get(param_name) {
                Some(TemplateArg::NonType(n)) if *n >= 0 => *n as usize,
                Some(TemplateArg::NonType(_)) => usize::MAX,
                _ => 0,
            };
            Ok(Type::Array(Box::new(subst_inner), size))
        }
        Type::Function {
            return_type,
            param_types,
        } => substitute_function_type(return_type, param_types, mapping, injected_class),
        Type::PackExpansion(inner) => Ok(Type::PackExpansion(Box::new(substitute_type(
            inner,
            mapping,
            injected_class,
        )?))),
        _ => Ok(ty.clone()),
    }
}

fn substitute_function_type(
    return_type: &Type,
    param_types: &[Type],
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<Type, CompileError> {
    let mut subst_params = Vec::new();
    for p in param_types {
        subst_params.push(substitute_type(p, mapping, injected_class)?);
    }
    Ok(Type::Function {
        return_type: Box::new(substitute_type(return_type, mapping, injected_class)?),
        param_types: subst_params,
    })
}

fn substitute_struct_type(
    name: &str,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Type {
    if let Some((c, target_ty)) = injected_class {
        let short_c = c.rsplit_once("::").map(|(_, s)| s).unwrap_or(c);
        if (name == c
            || name == short_c
            || crate::QualifiedName::new(c).matches(crate::QualifiedName::new(name)))
            && (!name.contains('<') || c.contains('<'))
        {
            return target_ty.clone();
        }
    }
    if let Some(TemplateArg::Type(target)) = mapping.get(name) {
        return target.clone();
    }
    if let Some((target, member)) = name
        .split_once("::")
        .and_then(|(base, member)| mapping.get(base).and_then(|t| t.as_type()).map(|t| (t, member)))
    {
        let base_str = target.to_c_type();
        return Type::Struct(format!("{}::{}", base_str, member));
    }
    if name.contains('<') {
        return Type::Struct(substitute_template_name(name, mapping));
    }
    Type::Struct(name.to_string())
}

fn has_unexpanded_member_pack(exprs: &[Expr], mapping: &HashMap<String, TemplateArg>) -> bool {
    let known: std::collections::HashSet<String> = mapping
        .iter()
        .filter(|(_, v)| v.is_pack())
        .map(|(k, _)| k.clone())
        .collect();
    exprs.iter().any(|e| {
        if let Expr::PackExpansion { pattern, .. } = e {
            let mut packs = std::collections::HashSet::new();
            collect_packs_in_expr(pattern, &known, &mut packs);
            packs.is_empty()
        } else {
            false
        }
    })
}

fn substitute_call(
    callee: &mut Expr,
    args: &mut Vec<Expr>,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
    call_pos: crate::ast::expr::ExprPos,
) -> Result<(), CompileError> {
    if let Expr::Identifier(name) = callee {
        substitute_callee_identifier(name, mapping, injected_class);
    }
    substitute_expr(callee, mapping, injected_class)?;
    if !has_unexpanded_member_pack(args, mapping) {
        *args = expand_expr_list(args, mapping).map_err(|e| {
            if e.line == 0 && e.col == 0 {
                CompileError::new(e.message, call_pos.line, call_pos.col)
            } else {
                e
            }
        })?;
    } else {
        for arg in args.iter_mut() {
            substitute_expr(arg, mapping, injected_class)?;
        }
    }
    if injected_class.is_some() {
        for arg in args {
            substitute_expr(arg, mapping, injected_class)?;
        }
    }
    Ok(())
}

fn substitute_init_elements(
    elements: &mut Vec<Expr>,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
    pos: crate::ast::expr::ExprPos,
) -> Result<(), CompileError> {
    if !has_unexpanded_member_pack(elements, mapping) {
        *elements = expand_expr_list(elements, mapping).map_err(|e| {
            if e.line == 0 && e.col == 0 {
                CompileError::new(e.message, pos.line, pos.col)
            } else {
                e
            }
        })?;
    } else {
        for el in elements.iter_mut() {
            substitute_expr(el, mapping, injected_class)?;
        }
    }
    if injected_class.is_some() {
        for el in elements {
            substitute_expr(el, mapping, injected_class)?;
        }
    }
    Ok(())
}

fn substitute_identifier(
    expr: &mut Expr,
    mapping: &HashMap<String, TemplateArg>,
) -> Result<(), CompileError> {
    let name = match expr {
        Expr::Identifier(name) => name.clone(),
        _ => return Ok(()),
    };
    if let Some(TemplateArg::NonType(val)) = mapping.get(&name) {
        let ty = if (*val as i32 as i128) == *val {
            Type::Int
        } else {
            Type::LongLong
        };
        *expr = Expr::IntLiteral(*val as i64, ty);
    } else if let Some(arg) = mapping.get(&name) && arg.is_pack() {
        let pos = expr.pos();
        return Err(CompileError::new(
            format!("parameter pack '{}' used unexpanded", name),
            pos.line,
            pos.col,
        ));
    } else if name.contains('<')
        && let Expr::Identifier(id) = expr
    {
        *id = substitute_template_name(&name, mapping);
    }
    Ok(())
}

pub fn substitute_expr(
    expr: &mut Expr,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<(), CompileError> {
    match expr {
        Expr::Call { callee, args, pos } => {
            substitute_call(callee, args, mapping, injected_class, *pos)?
        }
        Expr::Unary { expr, .. } | Expr::SizeofExpr(expr) | Expr::Delete { expr, .. } => {
            substitute_expr(expr, mapping, injected_class)?;
        }
        Expr::Binary { left: t, right: v, .. }
        | Expr::Assign { target: t, value: v, .. }
        | Expr::CompoundAssign { target: t, value: v, .. } => {
            substitute_expr(t, mapping, injected_class)?;
            substitute_expr(v, mapping, injected_class)?;
        }
        Expr::ArraySubscript { array, index, .. } => {
            substitute_expr(array, mapping, injected_class)?;
            substitute_expr(index, mapping, injected_class)?;
        }
        Expr::MemberAccess { object, member, .. } => {
            substitute_expr(object, mapping, injected_class)?;
            substitute_member_name(member, mapping);
        }
        Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => {
            for e in [cond, then_expr, else_expr] {
                substitute_expr(e, mapping, injected_class)?;
            }
        }
        Expr::Cast {
            target_type, expr, ..
        } => {
            *target_type = substitute_type(target_type, mapping, injected_class)?;
            substitute_expr(expr, mapping, injected_class)?;
        }
        Expr::SizeofType(ty) => *ty = substitute_type(ty, mapping, injected_class)?,
        Expr::TypedInitList { ty, elements, pos } => {
            *ty = substitute_type(ty, mapping, injected_class)?;
            substitute_init_elements(elements, mapping, injected_class, *pos)?;
        }
        Expr::New { .. } => substitute_new(expr, mapping, injected_class)?,
        Expr::InitializerList { elements, pos } => {
            substitute_init_elements(elements, mapping, injected_class, *pos)?;
        }
        Expr::Identifier(_) => substitute_identifier(expr, mapping)?,
        Expr::SizeofPack { name, .. } => {
            if let Some(len) = mapping.get(name).and_then(|a| a.pack_len()) {
                *expr = Expr::IntLiteral(len as i64, Type::UnsignedInt);
            }
        }
        Expr::PackExpansion { pattern, .. } => {
            substitute_expr(pattern, mapping, injected_class)?;
        }
        _ => {}
    }
    Ok(())
}

fn substitute_member_name(member: &mut String, mapping: &HashMap<String, TemplateArg>) {
    if let Some(rest) = member.strip_prefix('~')
        && let Some(TemplateArg::Type(ty)) = mapping.get(rest)
    {
        let c_ty = ty.to_c_type();
        let short = c_ty.rsplit("::").next().unwrap_or(&c_ty);
        *member = format!("~{short}");
    }
}

fn substitute_new(
    expr: &mut Expr,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<(), CompileError> {
    let new_pos = expr.pos();
    let Expr::New {
        ty,
        args,
        array_size,
        placement,
        ..
    } = expr
    else {
        return Ok(());
    };
    *ty = substitute_type(ty, mapping, injected_class)?;
    if !has_unexpanded_member_pack(args, mapping) {
        *args = expand_expr_list(args, mapping).map_err(|e| {
            if e.line == 0 && e.col == 0 {
                CompileError::new(e.message, new_pos.line, new_pos.col)
            } else {
                e
            }
        })?;
    } else {
        for arg in args.iter_mut() {
            substitute_expr(arg, mapping, injected_class)?;
        }
    }
    if injected_class.is_some() {
        for arg in args {
            substitute_expr(arg, mapping, injected_class)?;
        }
    }
    if let Some(size) = array_size {
        substitute_expr(size, mapping, injected_class)?;
    }
    if let Some(place) = placement {
        substitute_expr(place, mapping, injected_class)?;
    }
    Ok(())
}

fn substitute_callee_identifier(
    name: &mut String,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) {
    if let Some((_, target_ty)) = injected_class.filter(|(c, _)| *c == name.as_str()) {
        *name = target_ty.to_c_type();
    } else if let Some(TemplateArg::Type(target)) = mapping.get(name.as_str()) {
        *name = target.to_c_type();
    } else if let Some((target, member)) = name
        .split_once("::")
        .and_then(|(b, m)| mapping.get(b).and_then(|t| t.as_type()).map(|t| (t, m)))
    {
        *name = format!("{}::{}", target.to_c_type(), member);
    } else if name.contains('<') {
        *name = substitute_template_name(name, mapping);
    }
}

fn substitute_base_classes(
    bases: &[String],
    mapping: &HashMap<String, TemplateArg>,
    line: usize,
    col: usize,
) -> Result<Vec<String>, CompileError> {
    let mut base_classes = Vec::new();
    for base in bases {
        if let Some(stripped) = base.strip_suffix("...") {
            if let Some(TemplateArg::TypePack(types)) = mapping.get(stripped) {
                for ty in types {
                    base_classes.push(ty.to_c_type());
                }
            } else {
                base_classes.push(substitute_template_name(base, mapping));
            }
        } else if let Some(TemplateArg::Type(target)) = mapping.get(base) {
            base_classes.push(target.to_c_type());
        } else if base.contains('<') {
            base_classes.push(substitute_template_name(base, mapping));
        } else {
            base_classes.push(base.clone());
        }
    }
    if base_classes.len() > 1 {
        return Err(CompileError::new(
            "multiple inheritance is not supported",
            line,
            col,
        ));
    }
    Ok(base_classes)
}

pub fn substitute_struct(
    template_class: &TemplateClass,
    mapping: &HashMap<String, TemplateArg>,
    instantiated_name: &str,
    line: usize,
    col: usize,
) -> Result<Decl, CompileError> {
    let target_ty = Type::Struct(instantiated_name.to_string());
    let injected = Some((template_class.name.as_str(), &target_ty));

    let mut fields = Vec::new();
    for (fname, fty, finit) in &template_class.fields {
        let mut init = finit.clone();
        if let Some((init_expr, _, _)) = &mut init {
            substitute_expr(init_expr, mapping, injected)?;
        }
        fields.push((
            fname.clone(),
            substitute_type(fty, mapping, injected)?,
            init,
        ));
    }

    let mut methods = Vec::new();
    for m in &template_class.methods {
        methods.push(substitute_function(m, mapping, injected, None)?);
    }

    let mut static_fields = Vec::new();
    for (sname, sty, sinit) in &template_class.static_fields {
        let mut init = sinit.clone();
        if let Some(init_expr) = &mut init {
            substitute_expr(init_expr, mapping, injected)?;
        }
        static_fields.push((
            sname.clone(),
            substitute_type(sty, mapping, injected)?,
            init,
        ));
    }

    let base_classes =
        substitute_base_classes(&template_class.base_classes, mapping, line, col)?;

    Ok(Decl::StructDecl {
        name: instantiated_name.to_string(),
        is_class: template_class.is_class,
        base_classes,
        fields,
        methods,
        static_fields,
        private_members: template_class.private_members.clone(),
        nested_structs: Vec::new(),
        line,
        col,
    })
}

pub fn collapse_reference(outer_is_rvalue: bool, inner: Type) -> Type {
    if inner.is_reference() {
        let is_lvalue = inner.is_lvalue_reference();
        let target = inner.deref_type().cloned().unwrap_or(inner);
        if !outer_is_rvalue || is_lvalue {
            Type::Reference(Box::new(target))
        } else {
            Type::RvalueReference(Box::new(target))
        }
    } else if outer_is_rvalue {
        Type::RvalueReference(Box::new(inner))
    } else {
        Type::Reference(Box::new(inner))
    }
}
