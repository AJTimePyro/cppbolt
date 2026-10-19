//! Held-out corpus checks used by the runner's regression gate.

use super::cpp_files;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

/// Rejects a `*_heldout` test whose source matches its main counterpart once
/// comments and whitespace are removed, and warns when only identifiers and
/// literals differ.
pub(super) fn check_heldout_tests() -> usize {
    const PAIRS: &[(&str, &str)] = &[
        ("c_basics_heldout", "c_basics"),
        ("classes_oop_heldout", "classes_oop"),
        ("memory_model_heldout", "memory_model"),
        ("mini_stl_heldout", "mini_stl"),
    ];
    let mut failures = 0;
    for (held, main) in PAIRS {
        let directory = Path::new("tests").join(held);
        let files = match cpp_files(&directory) {
            Ok(files) => files,
            Err(_) => continue,
        };
        for path in files {
            let Some(name) = path.file_stem().and_then(OsStr::to_str) else {
                continue;
            };
            let main_path = Path::new("tests").join(main).join(format!("{name}.cpp"));
            if !main_path.exists() {
                continue;
            }
            let held_source = fs::read_to_string(&path).unwrap_or_default();
            let main_source = fs::read_to_string(&main_path).unwrap_or_default();
            if normalized_source(&held_source) == normalized_source(&main_source) {
                eprintln!("[FAIL] duplicate held-out test: {held}/{name}");
                failures += 1;
            } else if skeleton_source(&held_source) == skeleton_source(&main_source) {
                eprintln!(
                    "warning: held-out test {held}/{name} differs only in identifiers or literals"
                );
            }
        }
    }
    failures
}

/// Removes comments, then all whitespace, so formatting differences vanish.
fn normalized_source(text: &str) -> String {
    strip_comments(text)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

/// Removes comments and replaces identifiers and literals with placeholders,
/// leaving punctuation and control flow as the comparison skeleton.
fn skeleton_source(text: &str) -> String {
    let stripped = strip_comments(text);
    let bytes = stripped.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_alphanumeric() || c == b'_' {
            out.push('x');
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
        } else if c == b'"' || c == b'\'' {
            out.push('s');
            let quote = c;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                } else if bytes[i] == quote {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
        } else if c.is_ascii_whitespace() {
            i += 1;
        } else {
            out.push(c as char);
            i += 1;
        }
    }
    out
}

/// Returns `text` with `//` and `/* */` comments removed, preserving strings.
fn strip_comments(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    #[derive(PartialEq)]
    enum State {
        Code,
        Line,
        Block,
        Text,
    }
    let mut state = State::Code;
    while i < bytes.len() {
        let c = bytes[i];
        match state {
            State::Code => {
                if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    state = State::Line;
                    i += 2;
                } else if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    state = State::Block;
                    i += 2;
                } else {
                    if c == b'"' || c == b'\'' {
                        state = State::Text;
                    }
                    out.push(c as char);
                    i += 1;
                }
            }
            State::Line => {
                if c == b'\n' {
                    state = State::Code;
                    out.push('\n');
                }
                i += 1;
            }
            State::Block => {
                if c == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    state = State::Code;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            State::Text => {
                out.push(c as char);
                if c == b'\\' && i + 1 < bytes.len() {
                    out.push(bytes[i + 1] as char);
                    i += 2;
                } else {
                    if c == b'"' || c == b'\'' {
                        state = State::Code;
                    }
                    i += 1;
                }
            }
        }
    }
    out
}
