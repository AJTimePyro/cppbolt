//! Expression substitution helpers.

use super::pack_expansion::{TemplateArg, expand_expr_list};
use super::template_name::substitute_template_name;
use super::template_subst::substitute_type;
use super::template_subst_pack::has_unexpanded_member_pack;
use crate::ast::decl::FunctionDecl;
use crate::ast::expr::Expr;
use crate::ast::types::{FunctionQualifiers, Type};
use crate::diagnostics::CompileError;
use std::collections::HashMap;

pub(super) fn substitute_identifier(
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
    } else if let Some(TemplateArg::NonTypeAddress(addr)) = mapping.get(&name) {
        let pos = expr.pos();
        *expr = Expr::Unary {
            op: crate::ast::UnaryOp::AddrOf,
            expr: Box::new(Expr::Identifier(addr.clone())),
            pos,
        };
    } else if let Some(TemplateArg::NonTypeNull(declared)) = mapping.get(&name) {
        let null = Expr::IntLiteral(0, Type::Nullptr);
        *expr = match declared {
            Some(ty) => Expr::Cast {
                kind: crate::ast::CastKind::CStyle,
                target_type: ty.clone(),
                expr: Box::new(null),
            },
            None => null,
        };
    } else if let Some(arg) = mapping.get(&name)
        && arg.is_pack()
    {
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
    } else if let Some((base, member)) = name.split_once("::")
        && let Some(TemplateArg::Type(target)) = mapping.get(base)
        && let Expr::Identifier(id) = expr
    {
        *id = format!("{}::{}", target.to_c_type(), member);
    }
    Ok(())
}
pub(super) fn substitute_expr_typed(
    expr: &mut Expr,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<(), CompileError> {
    match expr {
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
        Expr::SizeofType(ty) | Expr::AlignofType(ty) => {
            *ty = substitute_type(ty, mapping, injected_class)?
        }
        Expr::TypedInitList { ty, elements, pos } => {
            *ty = substitute_type(ty, mapping, injected_class)?;
            substitute_init_elements(elements, mapping, injected_class, *pos)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

pub(super) fn substitute_sizeof_pack(
    expr: &mut Expr,
    mapping: &HashMap<String, TemplateArg>,
) -> Result<(), CompileError> {
    if let Expr::SizeofPack { name, pos } = expr {
        if let Some(len) = mapping.get(name).and_then(|a| a.pack_len()) {
            *expr = Expr::IntLiteral(len as i64, Type::UnsignedInt);
        } else if mapping.contains_key(name) {
            return Err(CompileError::new(
                format!("'{name}' does not refer to the name of a parameter pack"),
                pos.line,
                pos.col,
            ));
        }
    }
    Ok(())
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

pub(crate) fn substitute_noexcept_qualifiers(
    func: &FunctionDecl,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
    struct_sizes: Option<&HashMap<String, usize>>,
) -> FunctionQualifiers {
    let mut quals = func.qualifiers;
    if let Some(ref expr) = func.noexcept_expr {
        let mut subst_expr = expr.clone();
        if substitute_expr(&mut subst_expr, mapping, injected_class).is_ok() {
            if let Ok(val) = super::constexpr_eval::eval_constexpr_with_context(
                &subst_expr,
                struct_sizes,
                None,
                None,
            ) {
                quals.is_noexcept = val != 0;
            } else if let Ok(val) = super::constexpr_eval::eval_constexpr_with_mapping(
                &subst_expr,
                struct_sizes,
                mapping,
            ) {
                quals.is_noexcept = val != 0;
            }
        }
    }
    quals
}
