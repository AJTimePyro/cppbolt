//! Template-argument comparison helpers.

use super::pack_expansion::TemplateArg;
use crate::ast::expr::ExprPos;
use crate::diagnostics::CompileError;
use std::collections::HashMap;

pub(super) fn template_args_match(a: &TemplateArg, b: &TemplateArg) -> bool {
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
        (TemplateArg::NonTypeAddress(a), TemplateArg::NonTypeAddress(b)) => a == b,
        (TemplateArg::NonTypeNull(_), TemplateArg::NonTypeNull(_)) => true,
        _ => false,
    }
}
pub(super) fn format_template_arg(arg: &TemplateArg) -> String {
    match arg {
        TemplateArg::Type(t) => t.to_c_type(),
        TemplateArg::NonType(v) => v.to_string(),
        TemplateArg::NonTypeAddress(addr) => format!("&{addr}"),
        TemplateArg::NonTypeNull(_) => "nullptr".to_string(),
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
pub(super) fn record_deduced(
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
