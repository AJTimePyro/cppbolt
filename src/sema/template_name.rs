//! Template name substitution and string manipulation utilities.

use crate::sema::pack_expansion::TemplateArg;
use std::collections::HashMap;

pub fn substitute_template_name(name: &str, mapping: &HashMap<String, TemplateArg>) -> String {
    let (Some(open), Some(close)) = (name.find('<'), name.rfind('>')) else {
        return name.to_string();
    };
    if open >= close {
        return name.to_string();
    }
    let (base, inside, suffix) = (&name[..open], &name[open + 1..close], &name[close + 1..]);
    let mut result = inside.to_string();
    for (param, arg) in mapping {
        match arg {
            TemplateArg::Type(ty) => {
                result = replace_whole_word(&result, param, &ty.to_c_type());
            }
            TemplateArg::NonType(val) => {
                result = replace_whole_word(&result, param, &val.to_string());
            }
            TemplateArg::NonTypeAddress(addr) => {
                result = replace_whole_word(&result, param, &format!("&{addr}"));
            }
            TemplateArg::NonTypeNull(_) => {
                result = replace_whole_word(&result, param, "nullptr");
            }
            TemplateArg::TypePack(types) => {
                let joined = types
                    .iter()
                    .map(|t| t.to_c_type())
                    .collect::<Vec<_>>()
                    .join(", ");
                result = substitute_pack_string(result, param, &joined, types.len());
            }
            TemplateArg::NonTypePack(vals) => {
                let joined = vals
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                result = substitute_pack_string(result, param, &joined, vals.len());
            }
        }
    }
    let clean = clean_template_arg_string(&result);
    let subst_base = if let Some(TemplateArg::Type(target)) = mapping.get(base)
        && !target.to_c_type().contains('<')
    {
        target.to_c_type()
    } else {
        base.to_string()
    };
    format!("{}<{}>{}", subst_base, clean, suffix)
}

fn substitute_pack_string(result: String, param: &str, joined: &str, pack_len: usize) -> String {
    let sizeof_pack = format!("sizeof...({param})");
    let sizeof_pack_space = format!("sizeof... ({param})");
    let len_str = pack_len.to_string();
    let mut res = result;
    if res.contains(&sizeof_pack) {
        res = res.replace(&sizeof_pack, &len_str);
    }
    if res.contains(&sizeof_pack_space) {
        res = res.replace(&sizeof_pack_space, &len_str);
    }
    let with_ellipsis = format!("{param}...");
    let with_space = format!("{param} ...");
    if res.contains(&with_ellipsis) {
        res.replace(&with_ellipsis, joined)
    } else if res.contains(&with_space) {
        res.replace(&with_space, joined)
    } else {
        replace_whole_word(&res, param, joined)
    }
}

fn clean_template_arg_string(s: &str) -> String {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for c in s.chars() {
        match c {
            '<' => {
                depth += 1;
                current.push(c);
            }
            '>' => {
                depth = depth.saturating_sub(1);
                current.push(c);
            }
            ',' if depth == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    parts.push(simplify_arg_str(trimmed));
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        parts.push(simplify_arg_str(trimmed));
    }
    parts.join(", ")
}

fn simplify_arg_str(part: &str) -> String {
    let mut lexer = crate::lexer::Lexer::new(part);
    if let Ok(tokens) = lexer.tokenize() {
        let mut parser = crate::parser::Parser::new(tokens);
        if let Ok(expr) = parser.parse_expr()
            && parser.peek() == &crate::lexer::TokenKind::Eof
            && let Ok(val) = crate::sema::constexpr_eval::eval_integral_constexpr(&expr, None)
        {
            return val.to_string();
        }
    }
    part.to_string()
}

pub(crate) fn replace_whole_word(text: &str, word: &str, replacement: &str) -> String {
    let mut result = String::new();
    let mut remainder = text;
    while let Some(idx) = remainder.find(word) {
        let bytes = remainder.as_bytes();
        let not_qualified = !(idx >= 2 && &bytes[idx - 2..idx] == b"::");
        let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        let before_ok = not_qualified && (idx == 0 || !is_ident(bytes[idx - 1]));
        let after_idx = idx + word.len();
        let after_ok = after_idx == bytes.len() || !is_ident(bytes[after_idx]);
        if before_ok && after_ok {
            result.push_str(&remainder[..idx]);
            result.push_str(replacement);
        } else {
            result.push_str(&remainder[..after_idx]);
        }
        remainder = &remainder[after_idx..];
    }
    result.push_str(remainder);
    result
}

/// Textually substitutes template parameters inside a dependent non-type
/// argument spelling that cannot be re-parsed as an expression.
pub(crate) fn substitute_dependent_text(
    text: &str,
    mapping: &HashMap<String, TemplateArg>,
) -> String {
    let mut result = text.to_string();
    for (param, arg) in mapping {
        let value = match arg {
            TemplateArg::Type(ty) => ty.to_c_type(),
            TemplateArg::NonType(n) => n.to_string(),
            TemplateArg::NonTypeAddress(addr) => format!("&{addr}"),
            TemplateArg::NonTypeNull(_) => "nullptr".to_string(),
            TemplateArg::TypePack(types) => types
                .iter()
                .map(|t| t.to_c_type())
                .collect::<Vec<_>>()
                .join(", "),
            TemplateArg::NonTypePack(vals) => vals
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        };
        result = replace_whole_word(&result, param, &value);
    }
    result
}
