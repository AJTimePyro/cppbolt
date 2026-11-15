//! Dependent array bound evaluation during template substitution.

use super::pack_expansion::TemplateArg;
use super::template_subst::substitute_expr;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use std::collections::HashMap;

pub fn evaluate_dependent_array_size(
    param_name: &str,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<usize, CompileError> {
    if let Some(TemplateArg::NonType(n)) = mapping.get(param_name)
        && *n >= 0
    {
        return Ok(*n as usize);
    }
    if let Some(pack) = param_name.strip_prefix("sizeof...(").and_then(|s| s.strip_suffix(')'))
        && let Some(len) = mapping.get(pack).and_then(|a| a.pack_len())
    {
        return Ok(len);
    }
    let mut lexer = crate::lexer::Lexer::new(param_name);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(_) => return Ok(0),
    };
    let mut parser = crate::parser::Parser::new(tokens);
    let mut expr = match parser.parse_expr() {
        Ok(e) => e,
        Err(_) => return Ok(0),
    };
    substitute_expr(&mut expr, mapping, injected_class)?;
    match crate::sema::constexpr_eval::eval_integral_constexpr(&expr, None) {
        Ok(val) if val >= 0 => Ok(val as usize),
        Ok(_) => Err(CompileError::new(
            "Array dimension must be non-negative integer constant",
            0,
            0,
        )),
        Err(_) => Ok(0),
    }
}
