//! Dependent array bound evaluation during template substitution.

use super::pack_expansion::TemplateArg;
use super::template_subst::substitute_expr;
use crate::ast::expr::Expr;
use crate::ast::types::Type;
use crate::diagnostics::{CompileError, ErrorKind};
use std::collections::HashMap;

pub fn evaluate_dependent_array_size(
    param_name: &str,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
    mut sema: Option<&mut crate::sema::Sema>,
) -> Result<usize, CompileError> {
    if let Some(TemplateArg::NonType(n)) = mapping.get(param_name)
        && *n >= 0
    {
        return Ok(*n as usize);
    }
    if let Some(pack) = param_name
        .strip_prefix("sizeof...(")
        .and_then(|s| s.strip_suffix(')'))
        && let Some(len) = mapping.get(pack).and_then(|a| a.pack_len())
    {
        return Ok(len);
    }
    let mut lexer = crate::lexer::Lexer::new(param_name);
    let tokens = lexer.tokenize()?;
    let mut parser = crate::parser::Parser::new(tokens);
    // The bound came from spelling a dependent type back to source. Read its
    // identifiers as types so `sizeof(T::member)` stays a sizeof-type node.
    parser.allow_unknown_types = true;
    let mut expr = parser.parse_expr()?;
    substitute_expr(&mut expr, mapping, injected_class)?;
    if let Some(sema) = sema.as_deref_mut() {
        resolve_sizeof_operands(&mut expr, sema, "in array bound")?;
    }
    // Layout lives on Sema; without it `Type::size_of` falls back to 4.
    let env = crate::sema::constexpr_eval::ConstexprEnv {
        struct_sizes: sema.as_ref().map(|s| &s.struct_sizes),
        constants: None,
        pack_lengths: None,
        struct_aligns: sema.as_ref().map(|s| &s.struct_aligns),
    };
    match crate::sema::constexpr_eval::eval_integral_constexpr_with_env(&expr, env) {
        Ok(val) if val >= 0 => Ok(val as usize),
        Ok(_) => Err(CompileError::new(
            "Array dimension must be non-negative integer constant",
            0,
            0,
        )),
        Err(error) => Err(error),
    }
}

/// Resolves qualified struct operands of every `sizeof`/`alignof` in `expr`,
/// including ones nested inside composite expressions such as
/// `sizeof(T::type) + 1`. `context` is used in the diagnostic for a missing
/// nested type.
pub(super) fn resolve_sizeof_operands(
    expr: &mut Expr,
    sema: &mut crate::sema::Sema,
    context: &str,
) -> Result<(), CompileError> {
    match expr {
        Expr::SizeofType(ty) | Expr::AlignofType(ty) => {
            if let Type::Struct(name) = ty
                && name.contains("::")
            {
                let resolved = sema.resolve_type_name_for_sizeof(name)?.ok_or_else(|| {
                    CompileError::new(format!("unresolved type '{name}' {context}"), 0, 0)
                        .with_kind(ErrorKind::Substitution)
                })?;
                *ty = resolved;
            }
        }
        Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::SizeofExpr(expr) => {
            resolve_sizeof_operands(expr, sema, context)?;
        }
        Expr::Binary { left, right, .. } => {
            resolve_sizeof_operands(left, sema, context)?;
            resolve_sizeof_operands(right, sema, context)?;
        }
        Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => {
            resolve_sizeof_operands(cond, sema, context)?;
            resolve_sizeof_operands(then_expr, sema, context)?;
            resolve_sizeof_operands(else_expr, sema, context)?;
        }
        _ => {}
    }
    Ok(())
}
