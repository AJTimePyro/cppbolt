//! Support for the `defined` operator in `#if` expressions.

use std::collections::HashMap;

/// Rewrites `defined(NAME)` and `defined NAME` before macro expansion, as the
/// standard requires.
pub(super) fn replace_defined(
    argument: &str,
    macros: &HashMap<String, super::macros::MacroDef>,
) -> String {
    let chars: Vec<char> = argument.chars().collect();
    let mut out = String::new();
    let mut index = 0;
    while index < chars.len() {
        if !is_ident_start(chars[index]) {
            out.push(chars[index]);
            index += 1;
            continue;
        }
        let start = index;
        while index < chars.len() && is_ident_continue(chars[index]) {
            index += 1;
        }
        let ident: String = chars[start..index].iter().collect();
        if ident != "defined" {
            out.push_str(&ident);
            continue;
        }
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        let parenthesized = chars.get(index) == Some(&'(');
        if parenthesized {
            index += 1;
            while index < chars.len() && chars[index].is_whitespace() {
                index += 1;
            }
        }
        let name_start = index;
        while index < chars.len() && is_ident_continue(chars[index]) {
            index += 1;
        }
        let name: String = chars[name_start..index].iter().collect();
        if parenthesized {
            while index < chars.len() && chars[index].is_whitespace() {
                index += 1;
            }
            if chars.get(index) == Some(&')') {
                index += 1;
            }
        }
        out.push(if macros.contains_key(&name) { '1' } else { '0' });
    }
    out
}

pub(super) fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

pub(super) fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
