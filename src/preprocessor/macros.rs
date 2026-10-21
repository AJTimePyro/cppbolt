//! Object-like and function-like macro definition and expansion.

use super::Preprocessor;
use crate::diagnostics::CompileError;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub(super) enum MacroDef {
    Object(String),
    Function { params: Vec<String>, body: String },
}

/// The result of expanding one macro.
enum MacroExpansion {
    /// The name is not a macro and is kept as it is.
    NotAMacro,
    /// The expansion and the index just after it.
    Expanded(String, usize),
    /// The expansion, joined with the following source tokens.
    Joined(String),
}

/// One function-like macro definition.
struct FunctionMacro<'a> {
    /// The parameter names.
    pub(super) params: &'a [String],
    /// The replacement body.
    pub(super) body: &'a str,
    /// The macro name.
    pub(super) ident: &'a str,
}

/// The result of expanding one function-like macro.
enum FunctionExpansion {
    /// The macro was not called: the name is kept as it is.
    NotACall,
    /// The expansion and the index just after the closing parenthesis.
    Expanded(String, usize),
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

    /// Expands macros in one line, rescanning replacement lists.
    ///
    /// The `disabled` set implements the standard "painted blue" rule: a macro
    /// name is not expanded again while its own replacement list is being
    /// rescanned, so self-recursive and mutually recursive definitions
    /// terminate. The same set is threaded through argument pre-expansion and
    /// nested object/function macros.
    pub(super) fn expand_line(&self, line: &str) -> String {
        let mut disabled = HashSet::new();
        self.expand_with_hide_set(line, &mut disabled)
    }

    /// Expands the macros in `text`.
    ///
    /// `disabled` holds the macros that are currently being expanded, so a
    /// recursive macro stops instead of looping forever.
    fn expand_with_hide_set(&self, text: &str, disabled: &mut HashSet<String>) -> String {
        let chars: Vec<char> = text.chars().collect();
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
            if disabled.contains(&ident) {
                result.push_str(&ident);
                continue;
            }

            match self.expand_macro(definition, &ident, &chars, index, disabled) {
                MacroExpansion::Expanded(expanded, end_index) => {
                    result.push_str(&expanded);
                    index = end_index;
                }
                MacroExpansion::Joined(joined) => {
                    result.push_str(&joined);
                    return result;
                }
                MacroExpansion::NotAMacro => result.push_str(&ident),
            }
        }

        result
    }

    /// Expands the macro `ident` at the current position.
    fn expand_macro(
        &self,
        definition: &MacroDef,
        ident: &str,
        chars: &[char],
        index: usize,
        disabled: &mut HashSet<String>,
    ) -> MacroExpansion {
        match definition {
            MacroDef::Object(body) => {
                let expanded = self.expand_object_macro(body, ident, disabled);
                if let Some(joined) =
                    self.expand_joined(&expanded, &chars[index..], ident, disabled)
                {
                    return MacroExpansion::Joined(joined);
                }
                MacroExpansion::Expanded(expanded, index)
            }
            MacroDef::Function { params, body } => {
                let definition = FunctionMacro {
                    params,
                    body,
                    ident,
                };
                match self.expand_function_macro(&definition, chars, index, disabled) {
                    FunctionExpansion::Expanded(expanded, end_index) => {
                        if let Some(joined) =
                            self.expand_joined(&expanded, &chars[end_index..], ident, disabled)
                        {
                            return MacroExpansion::Joined(joined);
                        }
                        MacroExpansion::Expanded(expanded, end_index)
                    }
                    FunctionExpansion::NotACall => MacroExpansion::NotAMacro,
                }
            }
        }
    }

    /// Expands an object-like macro.
    fn expand_object_macro(
        &self,
        body: &str,
        ident: &str,
        disabled: &mut HashSet<String>,
    ) -> String {
        disabled.insert(ident.to_string());
        let expanded = self.expand_with_hide_set(body, disabled);
        disabled.remove(ident);
        expanded
    }

    /// Expands a function-like macro at the call site.
    fn expand_function_macro(
        &self,
        definition: &FunctionMacro<'_>,
        chars: &[char],
        index: usize,
        disabled: &mut HashSet<String>,
    ) -> FunctionExpansion {
        let FunctionMacro {
            params,
            body,
            ident,
        } = *definition;
        let mut call_start = index;
        while call_start < chars.len() && chars[call_start].is_whitespace() {
            call_start += 1;
        }
        if chars.get(call_start) != Some(&'(') {
            return FunctionExpansion::NotACall;
        }
        let Some((raw_args, end_index)) = extract_macro_args(chars, call_start) else {
            return FunctionExpansion::NotACall;
        };
        let expanded_args: Vec<String> = raw_args
            .iter()
            .map(|arg| self.expand_with_hide_set(arg, disabled))
            .collect();
        let substituted = substitute_body(body, params, &raw_args, &expanded_args);
        disabled.insert(ident.to_string());
        let expanded = self.expand_with_hide_set(&substituted, disabled);
        disabled.remove(ident);
        FunctionExpansion::Expanded(expanded, end_index)
    }

    /// Rescans an expansion together with the following source tokens when the
    /// expansion ends in a function-like macro name and the next non-space
    /// source character is `(`. This makes `ID(INC)(4)` expand `INC`, whose
    /// invocation continues in the original token stream.
    fn expand_joined(
        &self,
        expanded: &str,
        tail: &[char],
        exclude: &str,
        disabled: &mut HashSet<String>,
    ) -> Option<String> {
        let trimmed = expanded.trim_end();
        let start = trimmed
            .rfind(|c: char| !is_ident_continue(c))
            .map(|index| index + 1)
            .unwrap_or(0);
        let name = &trimmed[start..];
        if name.is_empty() || name == exclude || !is_valid_ident(name) {
            return None;
        }
        if !matches!(self.macros.get(name), Some(MacroDef::Function { .. })) {
            return None;
        }
        let mut next = 0;
        while next < tail.len() && tail[next].is_whitespace() {
            next += 1;
        }
        if tail.get(next) != Some(&'(') {
            return None;
        }
        let tail: String = tail.iter().collect();
        Some(self.expand_with_hide_set(&format!("{expanded}{tail}"), disabled))
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

fn substitute_body(
    body: &str,
    params: &[String],
    raw_args: &[String],
    expanded_args: &[String],
) -> String {
    let chars: Vec<char> = body.chars().collect();
    let mut result = String::new();
    let mut index = 0;

    while index < chars.len() {
        if chars[index] == '"' || chars[index] == '\'' {
            index = copy_quoted(&chars, index, chars[index], &mut result);
            continue;
        }
        if chars[index] == '#' {
            if chars.get(index + 1) == Some(&'#') {
                result.push_str("##");
                index += 2;
                continue;
            }
            let mut probe = index + 1;
            while probe < chars.len() && chars[probe].is_whitespace() {
                probe += 1;
            }
            let start = probe;
            while probe < chars.len() && is_ident_continue(chars[probe]) {
                probe += 1;
            }
            let ident: String = chars[start..probe].iter().collect();
            if let Some(position) = params.iter().position(|param| param == &ident) {
                result.push_str(&stringize(&raw_args[position]));
                index = probe;
                continue;
            }
            result.push('#');
            index += 1;
            continue;
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
        if let Some(position) = params.iter().position(|param| param == &ident) {
            result.push_str(&expanded_args[position]);
        } else {
            result.push_str(&ident);
        }
    }

    apply_token_paste(&result)
}

fn stringize(argument: &str) -> String {
    let mut out = String::from("\"");
    for c in argument.trim().chars() {
        if c == '\\' || c == '"' {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// Applies the `##` token-paste operator to a substituted replacement list.
fn apply_token_paste(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '"' || chars[index] == '\'' {
            let quote = chars[index];
            out.push(quote);
            index += 1;
            while index < chars.len() {
                let c = chars[index];
                out.push(c);
                index += 1;
                if c == '\\' && index < chars.len() {
                    out.push(chars[index]);
                    index += 1;
                } else if c == quote {
                    break;
                }
            }
            continue;
        }
        if chars[index] == '#' && chars.get(index + 1) == Some(&'#') {
            while out.ends_with(char::is_whitespace) {
                out.pop();
            }
            index += 2;
            while index < chars.len() && chars[index].is_whitespace() {
                index += 1;
            }
            continue;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expand(definitions: &[&str], line: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut preprocessor = Preprocessor::new();
        for definition in definitions {
            preprocessor.parse_define(definition, 1)?;
        }
        Ok(preprocessor.expand_line(line))
    }

    #[test]
    fn rescans_object_like_replacements() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(expand(&["A 1", "B A"], "return B-1;")?, "return 1-1;");
        Ok(())
    }

    #[test]
    fn self_and_mutual_recursion_terminate() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(expand(&["A A"], "A")?, "A");
        assert_eq!(expand(&["A B", "B A"], "A")?, "A");
        Ok(())
    }

    #[test]
    fn expands_function_arguments_and_rescans_body() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(expand(&["ID(x) x", "A 7"], "ID(A)")?, "7");
        Ok(())
    }

    #[test]
    fn rescans_expansion_with_following_invocation() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            expand(&["ID(x) x", "INC(x) ((x)+1)"], "ID(INC)(4)")?,
            "((4)+1)"
        );
        assert_eq!(expand(&["G F", "F(x) ((x)+1)"], "G(4)")?, "((4)+1)");
        Ok(())
    }

    #[test]
    fn stringizes_and_pastes_tokens() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(expand(&["STR(x) #x"], "STR(hello)")?, "\"hello\"");
        assert_eq!(expand(&["CAT(a,b) a ## b"], "CAT(foo,bar)")?, "foobar");
        Ok(())
    }
}
