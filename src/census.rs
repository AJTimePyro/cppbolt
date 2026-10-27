//! A census of the Rust sources: the size of every function, the number of
//! its parameters, and the size of every file.
//!
//! The scanner blanks the comments, the string literals, the raw strings,
//! the char literals, and the lifetimes before it looks for a function, so a
//! brace inside one of them never ends a function.  A function is counted
//! from the line of its `fn` keyword to the line of the closing brace of its
//! body.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One function of the census.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionRecord {
    /// Source file path.
    pub file: String,
    /// Function name.
    pub name: String,
    /// 1-based start line of `fn`.
    pub start_line: usize,
    /// 1-based end line of body closing brace.
    pub end_line: usize,
    /// Total line length.
    pub length: usize,
    /// Total parameter count including self.
    pub parameters: usize,
    /// Parameter count excluding receiver (`self`).
    pub parameters_without_receiver: usize,
}

/// The census of a source tree.
#[derive(Debug, Default)]
pub struct Census {
    /// All discovered functions.
    pub functions: Vec<FunctionRecord>,
    /// Line count for every scanned file.
    pub file_lines: BTreeMap<String, usize>,
}

impl Census {
    /// Counts functions longer than `limit` lines.
    pub fn over_length(&self, limit: usize) -> usize {
        self.functions
            .iter()
            .filter(|function| function.length > limit)
            .count()
    }

    /// Counts functions with more than `limit` parameters total.
    pub fn over_parameters(&self, limit: usize) -> usize {
        self.functions
            .iter()
            .filter(|function| function.parameters > limit)
            .count()
    }

    /// Counts functions with more than `limit` non-receiver parameters.
    pub fn over_parameters_without_receiver(&self, limit: usize) -> usize {
        self.functions
            .iter()
            .filter(|function| function.parameters_without_receiver > limit)
            .count()
    }

    /// Counts files longer than `limit` lines.
    pub fn files_over(&self, limit: usize) -> usize {
        self.file_lines
            .values()
            .filter(|lines| **lines > limit)
            .count()
    }

    /// Returns all functions longer than `limit` lines, sorted descending.
    pub fn longest(&self, limit: usize) -> Vec<&FunctionRecord> {
        let mut longest: Vec<&FunctionRecord> = self
            .functions
            .iter()
            .filter(|function| function.length > limit)
            .collect();
        longest.sort_by_key(|function| std::cmp::Reverse(function.length));
        longest
    }
}

/// Scans a source tree and returns its census.
pub fn census(root: &Path) -> std::io::Result<Census> {
    let mut result = Census::default();
    for path in source_files(&root.join("src"))? {
        let text = std::fs::read_to_string(&path)?;
        let file = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        result.file_lines.insert(file.clone(), text.lines().count());
        result.functions.extend(find_functions(&file, &text));
    }
    Ok(result)
}

/// Every `.rs` file under a directory, sorted.
fn source_files(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![directory.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Blanks the comments, the string literals, the raw strings, the char
/// literals, and the lifetimes.
///
/// The result has the same length as the input, so every offset still points
/// at the same character.
fn strip_code(text: &str) -> Vec<char> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = chars.clone();
    let mut index = 0;
    while index < chars.len() {
        let current = chars[index];
        if current == '/' && chars.get(index + 1) == Some(&'/') {
            index = blank_line_comment(&chars, &mut out, index);
        } else if current == '/' && chars.get(index + 1) == Some(&'*') {
            index = blank_block_comment(&chars, &mut out, index);
        } else if current == 'r' && matches!(chars.get(index + 1), Some('"') | Some('#')) {
            index = blank_raw_string(&chars, &mut out, index);
        } else if current == '"' {
            index = blank_string(&chars, &mut out, index);
        } else if current == '\'' {
            // A char literal is an escape or one character between quotes; a
            // lifetime is a quote that is never closed.
            index = match char_literal_length(&chars, index) {
                Some(length) => {
                    for position in out.iter_mut().skip(index).take(length) {
                        *position = ' ';
                    }
                    index + length
                }
                None => index + 1,
            };
        } else {
            index += 1;
        }
    }
    out
}

/// Blanks a line comment and returns the index after it.
fn blank_line_comment(chars: &[char], out: &mut [char], start: usize) -> usize {
    let mut index = start;
    while index < chars.len() && chars[index] != '\n' {
        out[index] = ' ';
        index += 1;
    }
    index
}

/// Blanks a block comment, nested ones included, and returns the index after
/// it.
fn blank_block_comment(chars: &[char], out: &mut [char], start: usize) -> usize {
    let mut index = start;
    let mut depth = 0;
    while index < chars.len() {
        if chars[index] == '/' && chars.get(index + 1) == Some(&'*') {
            depth += 1;
            out[index] = ' ';
            out[index + 1] = ' ';
            index += 2;
        } else if chars[index] == '*' && chars.get(index + 1) == Some(&'/') {
            depth -= 1;
            out[index] = ' ';
            out[index + 1] = ' ';
            index += 2;
            if depth == 0 {
                break;
            }
        } else {
            if chars[index] != '\n' {
                out[index] = ' ';
            }
            index += 1;
        }
    }
    index
}

/// Blanks a raw string and returns the index after it.
///
/// A raw string that is not terminated ends the file.
fn blank_raw_string(chars: &[char], out: &mut [char], start: usize) -> usize {
    let mut cursor = start + 1;
    let mut hashes = 0;
    while chars.get(cursor) == Some(&'#') {
        hashes += 1;
        cursor += 1;
    }
    if chars.get(cursor) != Some(&'"') {
        return start + 1;
    }
    let terminator: String = std::iter::once('"')
        .chain(std::iter::repeat_n('#', hashes))
        .collect();
    let rest: String = chars[cursor + 1..].iter().collect();
    let end = match rest.find(&terminator) {
        Some(offset) => cursor + 1 + offset + terminator.len(),
        None => chars.len(),
    };
    for position in out.iter_mut().skip(start).take(end - start) {
        if *position != '\n' {
            *position = ' ';
        }
    }
    end
}

/// Blanks a string literal and returns the index after it.
fn blank_string(chars: &[char], out: &mut [char], start: usize) -> usize {
    let mut index = start + 1;
    while index < chars.len() {
        if chars[index] == '\\' {
            out[index] = ' ';
            if index + 1 < chars.len() && chars[index + 1] != '\n' {
                out[index + 1] = ' ';
            }
            index += 2;
            continue;
        }
        if chars[index] == '"' {
            out[index] = ' ';
            return index + 1;
        }
        if chars[index] != '\n' {
            out[index] = ' ';
        }
        index += 1;
    }
    index
}

/// The length of the char literal that starts at `index`, if there is one.
///
/// A char literal is a quote, an escaped character or a plain one, and a
/// closing quote.  A quote that is never closed is a lifetime.
fn char_literal_length(chars: &[char], index: usize) -> Option<usize> {
    let second = *chars.get(index + 1)?;
    if second == '\\' {
        chars.get(index + 2)?;
        if chars.get(index + 3) != Some(&'\'') {
            return None;
        }
        return Some(4);
    }
    if second == '\'' {
        return None;
    }
    if chars.get(index + 2) != Some(&'\'') {
        return None;
    }
    Some(3)
}

/// The index of the bracket that closes the one at `start`, if there is one.
fn match_pair(code: &[char], start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0;
    for (offset, character) in code[start..].iter().enumerate() {
        if *character == open {
            depth += 1;
        } else if *character == close {
            depth -= 1;
            if depth == 0 {
                return Some(start + offset);
            }
        }
    }
    None
}

/// Splits a parameter list on the commas that are not nested.
fn split_parameters(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for character in text.chars() {
        match character {
            '(' | '[' | '{' | '<' => {
                depth += 1;
                current.push(character);
            }
            ')' | ']' | '}' | '>' => {
                depth -= 1;
                current.push(character);
            }
            ',' if depth == 0 => {
                parts.push(current.clone());
                current.clear();
            }
            _ => current.push(character),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current);
    }
    parts
}

/// Returns true when a parameter is the receiver of the function.
fn is_receiver(parameter: &str) -> bool {
    let trimmed = parameter.trim();
    let name = trimmed.split(':').next().unwrap_or(trimmed).trim();
    matches!(name, "self" | "mut self" | "&self" | "&mut self") || trimmed.starts_with("self")
}

/// Finds every function of a source file.
fn find_functions(file: &str, text: &str) -> Vec<FunctionRecord> {
    let code = strip_code(text);
    let mut records = Vec::new();
    let mut index = 0;
    while index + 1 < code.len() {
        if code[index] != 'f' || code[index + 1] != 'n' {
            index += 1;
            continue;
        }
        if index > 0 && is_word_character(code[index - 1]) {
            index += 1;
            continue;
        }
        if code.get(index + 2).is_some_and(|c| is_word_character(*c)) {
            index += 1;
            continue;
        }
        let Some(record) = function_at(&code, file, index) else {
            index += 1;
            continue;
        };
        records.push(record);
        // The search continues after the keyword, so that a function inside
        // the body of another one is found as well.
        index += 2;
    }
    records
}

/// Returns true when a character is part of an identifier.
fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// Reads the function that starts at the `fn` keyword at `start`.
fn function_at(code: &[char], file: &str, start: usize) -> Option<FunctionRecord> {
    let mut cursor = start + 2;
    while matches!(code.get(cursor), Some(' ') | Some('\t') | Some('\n')) {
        cursor += 1;
    }
    let mut name = String::new();
    while let Some(character) = code.get(cursor) {
        if character.is_alphanumeric() || *character == '_' {
            name.push(*character);
            cursor += 1;
        } else {
            break;
        }
    }
    let mut angle = 0i32;
    while cursor < code.len() {
        match code[cursor] {
            '<' => angle += 1,
            '>' => angle -= 1,
            '(' if angle <= 0 => break,
            ';' | '{' if angle <= 0 => return None,
            _ => {}
        }
        cursor += 1;
    }
    if code.get(cursor) != Some(&'(') {
        return None;
    }
    let parameters_open = cursor;
    let parameters_close = match_pair(code, parameters_open, '(', ')')?;
    let parameters: String = code[parameters_open + 1..parameters_close].iter().collect();
    let mut body = parameters_close + 1;
    while body < code.len() && code[body] != '{' {
        if code[body] == ';' {
            return None;
        }
        body += 1;
    }
    if body >= code.len() {
        return None;
    }
    let body_end = match_pair(code, body, '{', '}')?;
    let parts: Vec<String> = split_parameters(&parameters)
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect();
    let has_receiver = parts.iter().any(|part| is_receiver(part));
    let start_line = line_of(code, start);
    let end_line = line_of(code, body_end);
    Some(FunctionRecord {
        file: file.to_string(),
        name: if name.is_empty() {
            "?".to_string()
        } else {
            name
        },
        start_line,
        end_line,
        length: end_line - start_line + 1,
        parameters: parts.len(),
        parameters_without_receiver: parts.len() - usize::from(has_receiver),
    })
}

/// The line of a character of the stripped code.
fn line_of(code: &[char], index: usize) -> usize {
    let mut line = 1;
    for character in &code[..index] {
        if *character == '\n' {
            line += 1;
        }
    }
    line
}
