//! Object-like and function-like macro definition and expansion.

use super::Preprocessor;
use crate::diagnostics::CompileError;

#[derive(Clone, Debug)]
pub(super) enum MacroDef {
    Object(String),
    Function { params: Vec<String>, body: String },
}

impl Preprocessor<'_> {
    pub(super) fn parse_define(
        &mut self,
        rest: &str,
        line_number: usize,
    ) -> Result<(), CompileError> {
        if rest.is_empty() {
            return Err(CompileError::new("Empty #define directive", line_number, 1));
        }

        if let Some(open_paren) = rest.find('(') {
            let name = &rest[..open_paren];
            if !name.contains(char::is_whitespace) && is_valid_ident(name) {
                let after_open = &rest[open_paren + 1..];
                if let Some(close_paren) = after_open.find(')') {
                    let params = after_open[..close_paren]
                        .split(',')
                        .map(str::trim)
                        .filter(|param| !param.is_empty())
                        .map(str::to_string)
                        .collect();
                    let body = after_open[close_paren + 1..].trim().to_string();
                    self.macros
                        .insert(name.to_string(), MacroDef::Function { params, body });
                    return Ok(());
                }
            }
        }

        let name = rest.split_whitespace().next().unwrap_or("");
        if !is_valid_ident(name) {
            return Err(CompileError::new(
                format!("Invalid macro name '{}'", name),
                line_number,
                1,
            ));
        }
        let body = rest[name.len()..].trim().to_string();
        self.macros.insert(name.to_string(), MacroDef::Object(body));
        Ok(())
    }

    pub(super) fn expand_line(&self, line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut result = String::new();
        let mut index = 0;

        while index < chars.len() {
            if chars[index] == '"' {
                index = copy_quoted(&chars, index, '"', &mut result);
                continue;
            }
            if chars[index] == '\'' {
                index = copy_quoted(&chars, index, '\'', &mut result);
                continue;
            }
            if chars[index] == '/' && chars.get(index + 1) == Some(&'/') {
                result.extend(&chars[index..]);
                break;
            }
            if !is_ident_start(chars[index]) {
                result.push(chars[index]);
                index += 1;
                continue;
            }

            let start = index;
            while index < chars.len() && is_ident_continue(chars[index]) {
                index += 1;
            }
            let ident: String = chars[start..index].iter().collect();
            let Some(definition) = self.macros.get(&ident) else {
                result.push_str(&ident);
                continue;
            };

            match definition {
                MacroDef::Object(body) => result.push_str(body),
                MacroDef::Function { params, body } => {
                    let mut call_start = index;
                    while call_start < chars.len() && chars[call_start].is_whitespace() {
                        call_start += 1;
                    }
                    let is_call = chars.get(call_start) == Some(&'(');
                    let extracted = if is_call {
                        extract_macro_args(&chars, call_start)
                    } else {
                        None
                    };
                    match extracted {
                        Some((args, end_index)) => {
                            let mut expanded = body.clone();
                            for (param, arg) in params.iter().zip(args.iter()) {
                                expanded = replace_param(&expanded, param, arg);
                            }
                            result.push_str(&expanded);
                            index = end_index;
                        }
                        None => result.push_str(&ident),
                    }
                }
            }
        }

        result
    }
}

fn copy_quoted(chars: &[char], start: usize, quote: char, out: &mut String) -> usize {
    out.push(chars[start]);
    let mut index = start + 1;
    while index < chars.len() {
        let current = chars[index];
        out.push(current);
        index += 1;
        if current == '\\' && index < chars.len() {
            out.push(chars[index]);
            index += 1;
        } else if current == quote {
            break;
        }
    }
    index
}

fn is_valid_ident(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) if is_ident_start(first) => chars.all(is_ident_continue),
        _ => false,
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn extract_macro_args(chars: &[char], open_paren_index: usize) -> Option<(Vec<String>, usize)> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 1usize;
    let mut index = open_paren_index + 1;

    while index < chars.len() {
        let c = chars[index];
        match c {
            '(' => {
                depth += 1;
                current.push(c);
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let trimmed = current.trim();
                    if !trimmed.is_empty() || !args.is_empty() {
                        args.push(trimmed.to_string());
                    }
                    return Some((args, index + 1));
                }
                current.push(c);
            }
            ',' if depth == 1 => {
                args.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(c),
        }
        index += 1;
    }

    None
}

fn replace_param(body: &str, param: &str, replacement: &str) -> String {
    let chars: Vec<char> = body.chars().collect();
    let mut result = String::new();
    let mut index = 0;

    while index < chars.len() {
        if !is_ident_start(chars[index]) {
            result.push(chars[index]);
            index += 1;
            continue;
        }
        let start = index;
        while index < chars.len() && is_ident_continue(chars[index]) {
            index += 1;
        }
        let ident: String = chars[start..index].iter().collect();
        if ident == param {
            result.push_str(replacement);
        } else {
            result.push_str(&ident);
        }
    }

    result
}
