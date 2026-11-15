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
            TemplateArg::TypePack(types) => {
                let joined = types.iter().map(|t| t.to_c_type()).collect::<Vec<_>>().join(", ");
                result = substitute_pack_string(result, param, &joined);
            }
            TemplateArg::NonTypePack(vals) => {
                let joined = vals.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ");
                result = substitute_pack_string(result, param, &joined);
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

fn substitute_pack_string(result: String, param: &str, joined: &str) -> String {
    let with_ellipsis = format!("{param}...");
    let with_space = format!("{param} ...");
    if result.contains(&with_ellipsis) {
        result.replace(&with_ellipsis, joined)
    } else if result.contains(&with_space) {
        result.replace(&with_space, joined)
    } else {
        replace_whole_word(&result, param, joined)
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
                    parts.push(trimmed.to_string());
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        parts.push(trimmed.to_string());
    }
    parts.join(", ")
}

fn replace_whole_word(text: &str, word: &str, replacement: &str) -> String {
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
