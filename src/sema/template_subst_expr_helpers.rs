//! Expression substitution helpers.

use super::pack_expansion::TemplateArg;
use super::template_name::substitute_template_name;
use super::template_subst::{substitute_expr, substitute_init_elements, substitute_type};
use crate::ast::expr::Expr;
use crate::ast::types::Type;
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
