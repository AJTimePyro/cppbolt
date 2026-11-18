//! Dependent non-type template argument substitution and evaluation.

use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::sema::pack_expansion::TemplateArg;
use crate::sema::template_subst::substitute_expr;
use std::collections::HashMap;

pub fn evaluate_dependent_nontype(
    expr_str: &str,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<Type, CompileError> {
    if let Some(pack) = expr_str
        .strip_prefix("sizeof...")
        .and_then(|s| s.trim().strip_prefix('('))
        .and_then(|s| s.strip_suffix(')'))
    {
        let pack = pack.trim();
        if let Some(arg) = mapping.get(pack) {
            if let Some(len) = arg.pack_len() {
                return Ok(Type::NonType(len as i128));
            }
            return Err(CompileError::new(
                format!("'{pack}' does not refer to the name of a parameter pack"),
                0,
                0,
            ));
        }
    }
    if let Some(TemplateArg::NonType(n)) = mapping.get(expr_str) {
        return Ok(Type::NonType(*n));
    }

    let mut lexer = crate::lexer::Lexer::new(expr_str);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(_) => return Ok(Type::DependentNonType(expr_str.to_string())),
    };
    let mut parser = crate::parser::Parser::new(tokens);
    let mut expr = match parser.parse_expr() {
        Ok(e) => e,
        Err(_) => {
            // The dependent spelling may not re-parse (e.g. `!Trait<T>::v`).
            // Substitute the parameters textually and let sema resolve it.
            return Ok(Type::DependentNonType(
                super::template_name::substitute_dependent_text(expr_str, mapping),
            ));
        }
    };
    substitute_expr(&mut expr, mapping, injected_class)?;
    match crate::sema::constexpr_eval::eval_constexpr_with_mapping(&expr, None, mapping) {
        Ok(val) => Ok(Type::NonType(val)),
        Err(_) => match crate::sema::constexpr_eval::eval_integral_constexpr(&expr, None) {
            Ok(val) => Ok(Type::NonType(val)),
            Err(_) => {
                if let Some(dep) = crate::parser::types_array::expr_to_dependent_str(&expr) {
                    Ok(Type::DependentNonType(dep))
                } else {
                    Ok(Type::DependentNonType(expr_str.to_string()))
                }
            }
        },
    }
}
