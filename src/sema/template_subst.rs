//! Template AST substitution: clones and replaces template type parameters
//! with concrete argument types across types, expressions, statements, and declarations.
use super::template_subst_expr_helpers::{
    substitute_expr_typed, substitute_identifier, substitute_sizeof_pack,
};
use super::template_subst_pack::has_unexpanded_member_pack;

use crate::ast::decl::Decl;
use crate::ast::expr::Expr;
use crate::ast::template::TemplateClass;
use crate::ast::types::{FunctionQualifiers, Type};
use crate::diagnostics::CompileError;
use crate::sema::pack_expansion::{TemplateArg, expand_expr_list, expand_type_list};
pub use crate::sema::template_name::substitute_template_name;
pub use crate::sema::template_subst_func::substitute_function;
use std::collections::HashMap;

pub fn substitute_type(
    ty: &Type,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<Type, CompileError> {
    substitute_type_full(ty, mapping, injected_class, (0, 0), None, None)
}
fn substitute_template_instance(
    name: &str,
    args: &[Type],
    mapping: &HashMap<String, TemplateArg>,
    pos: (usize, usize),
) -> Result<Type, CompileError> {
    let subst_name = if name.contains('<') {
        substitute_template_name(name, mapping)
    } else {
        name.to_string()
    };
    Ok(Type::TemplateInstance {
        name: subst_name,
        args: expand_type_list(args, mapping, pos)?,
    })
}

pub fn substitute_type_full(
    ty: &Type,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
    pos: (usize, usize),
    struct_aligns: Option<&HashMap<String, usize>>,
    sema: Option<&crate::sema::Sema>,
) -> Result<Type, CompileError> {
    if let Some(res) = substitute_modified_type(ty, mapping, injected_class, pos, sema)? {
        return Ok(res);
    }
    match ty {
        Type::TypeParam(name) => {
            super::template_subst_align::substitute_type_param(name, ty, mapping, pos)
        }
        Type::Struct(name) => Ok(substitute_struct_type(name, mapping, injected_class)),
        Type::TemplateInstance { name, args } => {
            substitute_template_instance(name, args, mapping, pos)
        }
        Type::Decltype {
            expr,
            parenthesized,
        } => {
            let mut substituted = (**expr).clone();
            substitute_expr(&mut substituted, mapping, injected_class)?;
            Ok(Type::Decltype {
                expr: Box::new(substituted),
                parenthesized: *parenthesized,
            })
        }
        Type::DependentArray(inner, param_name) => {
            let subst_inner =
                substitute_type_full(inner, mapping, injected_class, pos, struct_aligns, sema)?;
            let size = super::template_subst_array::evaluate_dependent_array_size(
                param_name,
                mapping,
                injected_class,
                sema,
            )?;
            Ok(Type::Array(Box::new(subst_inner), size))
        }
        Type::Function {
            return_type,
            param_types,
            ..
        } => substitute_function_type(return_type, param_types, mapping, injected_class, sema),
        Type::PackExpansion(inner) => Ok(Type::PackExpansion(Box::new(substitute_type_full(
            inner,
            mapping,
            injected_class,
            pos,
            struct_aligns,
            sema,
        )?))),
        Type::DependentAlignas(inner, pack_name) => {
            super::template_subst_align::substitute_dependent_alignas(
                inner,
                pack_name,
                mapping,
                injected_class,
                struct_aligns,
            )
        }
        Type::Aligned(inner, align) => Ok(Type::Aligned(
            Box::new(substitute_type_full(
                inner,
                mapping,
                injected_class,
                pos,
                struct_aligns,
                sema,
            )?),
            *align,
        )),
        Type::DependentNonType(expr_str)
            if expr_str.starts_with(crate::sema::pack_expansion::ADDRESS_ARG_PREFIX) =>
        {
            Ok(ty.clone())
        }
        Type::DependentNonType(expr_str) => {
            super::template_subst_nontype::evaluate_dependent_nontype(
                expr_str,
                mapping,
                injected_class,
            )
        }
        _ => Ok(ty.clone()),
    }
}

fn substitute_modified_type(
    ty: &Type,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
    pos: (usize, usize),
    sema: Option<&crate::sema::Sema>,
) -> Result<Option<Type>, CompileError> {
    match ty {
        Type::Const(inner) => {
            let subst = substitute_type_full(inner, mapping, injected_class, pos, None, sema)?;
            if subst.is_reference() {
                Ok(Some(subst))
            } else {
                Ok(Some(Type::Const(Box::new(subst))))
            }
        }
        Type::Pointer(inner) => Ok(Some(Type::Pointer(Box::new(substitute_type_full(
            inner,
            mapping,
            injected_class,
            pos,
            None,
            sema,
        )?)))),
        Type::Reference(inner) => Ok(Some(collapse_reference(
            false,
            substitute_type_full(inner, mapping, injected_class, pos, None, sema)?,
        ))),
        Type::RvalueReference(inner) => Ok(Some(collapse_reference(
            true,
            substitute_type_full(inner, mapping, injected_class, pos, None, sema)?,
        ))),
        Type::Array(inner, size) => Ok(Some(Type::Array(
            Box::new(substitute_type_full(
                inner,
                mapping,
                injected_class,
                pos,
                None,
                sema,
            )?),
            *size,
        ))),
        Type::MemberPointer {
            class_name,
            member_type,
        } => {
            let cls = match mapping.get(class_name) {
                Some(TemplateArg::Type(Type::Struct(s) | Type::TypeParam(s))) => s.clone(),
                _ => class_name.clone(),
            };
            let mty = substitute_type_full(member_type, mapping, injected_class, pos, None, sema)?;
            Ok(Some(Type::MemberPointer {
                class_name: cls,
                member_type: Box::new(mty),
            }))
        }
        _ => Ok(None),
    }
}

fn substitute_function_type(
    return_type: &Type,
    param_types: &[Type],
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
    sema: Option<&crate::sema::Sema>,
) -> Result<Type, CompileError> {
    let subst_params = super::pack_expansion::expand_type_list(param_types, mapping, (0, 0))?;
    Ok(Type::Function {
        return_type: Box::new(substitute_type_full(
            return_type,
            mapping,
            injected_class,
            (0, 0),
            None,
            sema,
        )?),
        param_types: subst_params,
        qualifiers: FunctionQualifiers::default(),
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
    if let Some((target, member)) = name.split_once("::").and_then(|(base, member)| {
        mapping
            .get(base)
            .and_then(|t| t.as_type())
            .map(|t| (t, member))
    }) {
        let base_str = target.to_c_type();
        return Type::Struct(format!("{}::{}", base_str, member));
    }
    if name.contains('<') {
        return Type::Struct(substitute_template_name(name, mapping));
    }
    Type::Struct(name.to_string())
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

pub(super) fn substitute_init_elements(
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

pub fn substitute_expr(
    expr: &mut Expr,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<(), CompileError> {
    match expr {
        Expr::Call {
            callee,
            args,
            pos,
            dependent,
        } => {
            let before_callee = callee.clone();
            let before_args = args.clone();
            substitute_call(callee, args, mapping, injected_class, *pos)?;
            // A changed callee or argument means the call depends on template
            // parameters, so its ADL must run at instantiation.
            *dependent |= *callee != before_callee || *args != before_args;
        }
        Expr::Unary { expr, .. } | Expr::SizeofExpr(expr) | Expr::Delete { expr, .. } => {
            substitute_expr(expr, mapping, injected_class)?;
        }
        Expr::Binary {
            left: t, right: v, ..
        }
        | Expr::Assign {
            target: t,
            value: v,
            ..
        }
        | Expr::CompoundAssign {
            target: t,
            value: v,
            ..
        } => {
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
        Expr::Ternary { .. }
        | Expr::Cast { .. }
        | Expr::SizeofType(_)
        | Expr::AlignofType(_)
        | Expr::TypedInitList { .. } => substitute_expr_typed(expr, mapping, injected_class)?,
        Expr::New { .. } => substitute_new(expr, mapping, injected_class)?,
        Expr::InitializerList { elements, pos } => {
            substitute_init_elements(elements, mapping, injected_class, *pos)?;
        }
        Expr::Identifier(_) => substitute_identifier(expr, mapping)?,
        Expr::SizeofPack { .. } => substitute_sizeof_pack(expr, mapping)?,
        Expr::PackExpansion { pattern, .. } => substitute_expr(pattern, mapping, injected_class)?,
        Expr::FoldExpr {
            op,
            left,
            right,
            pos,
        } => {
            *expr = crate::sema::fold_expansion::expand_fold_expr(
                *op,
                left.as_deref(),
                right.as_deref(),
                mapping,
                *pos,
            )?;
        }
        Expr::Lambda { .. } => {
            crate::sema::template_subst_lambda::substitute_lambda(expr, mapping, injected_class)?;
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

pub fn substitute_struct(
    template_class: &TemplateClass,
    mapping: &HashMap<String, TemplateArg>,
    instantiated_name: &str,
    pos: (usize, usize),
    struct_aligns: Option<&HashMap<String, usize>>,
    sema: Option<&crate::sema::Sema>,
) -> Result<Decl, CompileError> {
    let (line, col) = pos;
    let target_ty = Type::Struct(instantiated_name.to_string());
    let injected = Some((template_class.name.as_str(), &target_ty));

    let mut fields = Vec::new();
    for (fname, fty, finit) in &template_class.fields {
        let mut init = finit.clone();
        if let Some((init_expr, _, _)) = &mut init {
            substitute_expr(init_expr, mapping, injected)?;
        }
        let subst_fty = substitute_type_full(fty, mapping, injected, pos, struct_aligns, sema)
            .map_err(|mut e| {
                if e.line == 0 && e.col == 0 {
                    e.line = line;
                    e.col = col;
                }
                e
            })?;
        fields.push((fname.clone(), subst_fty, init));
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
            substitute_type_full(sty, mapping, injected, pos, struct_aligns, sema)?,
            init,
        ));
    }

    let base_classes = super::template_subst_align::substitute_base_classes(
        &template_class.base_classes,
        mapping,
        line,
        col,
    )?;
    let alignment = template_class
        .alignment
        .as_ref()
        .map(|ty| substitute_type_full(ty, mapping, injected, pos, struct_aligns, sema))
        .transpose()?;

    Ok(Decl::StructDecl {
        name: instantiated_name.to_string(),
        is_class: template_class.is_class,
        is_final: template_class.is_final,
        base_classes,
        base_access: template_class.base_access.clone(),
        fields,
        methods,
        static_fields,
        private_members: template_class.private_members.clone(),
        nested_structs: Vec::new(),
        alignment,
        line,
        col,
    })
}

pub use super::template_subst_align::collapse_reference;
