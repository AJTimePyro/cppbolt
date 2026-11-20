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
    sema: Option<&crate::sema::Sema>,
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
    if let Expr::SizeofType(ty) = &mut expr
        && let Type::Struct(name) = ty
        && name.contains("::")
    {
        let resolved = sema
            .map(|sema| sema.resolve_type_name_for_sizeof(name))
            .transpose()?
            .flatten();
        if let Some(resolved) = resolved {
            *ty = resolved;
        } else {
            return Err(CompileError::new(
                format!("unresolved type '{}' in sizeof array bound", name),
                0,
                0,
            )
            .with_kind(ErrorKind::Substitution));
        }
    }
    match crate::sema::constexpr_eval::eval_integral_constexpr(&expr, None) {
        Ok(val) if val >= 0 => Ok(val as usize),
        Ok(_) => Err(CompileError::new(
            "Array dimension must be non-negative integer constant",
            0,
            0,
        )),
        Err(error) => Err(error),
    }
}
