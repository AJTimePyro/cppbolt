//! Lambda expression template AST substitution.

use std::collections::HashMap;

use super::pack_expansion::{pack_element_name, TemplateArg};
use super::template_subst::{substitute_expr, substitute_type};
use super::template_subst_stmt::substitute_stmt;
use crate::ast::expr::{Expr, LambdaCapture};
use crate::ast::types::Type;
use crate::diagnostics::CompileError;

pub fn substitute_lambda(
    expr: &mut Expr,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<(), CompileError> {
    let Expr::Lambda {
        captures,
        return_type,
        params,
        body,
        pos,
        ..
    } = expr
    else {
        return Ok(());
    };

    let mut new_captures = Vec::new();
    for cap in captures.iter() {
        if cap.is_pack {
            let Some(pack_len) = mapping.get(&cap.name).and_then(|a| a.pack_len()) else {
                return Err(CompileError::new(
                    format!("'{}' does not refer to the name of a parameter pack", cap.name),
                    pos.line,
                    pos.col,
                ));
            };
            for i in 0..pack_len {
                new_captures.push(LambdaCapture {
                    name: pack_element_name(&cap.name, i),
                    is_reference: cap.is_reference,
                    is_pack: false,
                });
            }
        } else {
            new_captures.push(cap.clone());
        }
    }
    *captures = new_captures;

    if let Some(ret_ty) = return_type {
        *ret_ty = substitute_type(ret_ty, mapping, injected_class)?;
    }

    for param in params.iter_mut() {
        param.ty = substitute_type(&param.ty, mapping, injected_class)?;
        if let Some(def) = &mut param.default_val {
            substitute_expr(def, mapping, injected_class)?;
        }
    }

    substitute_stmt(body, mapping, injected_class)?;

    Ok(())
}
