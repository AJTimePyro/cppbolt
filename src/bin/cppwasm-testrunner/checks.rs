//! Variant and independent corpus checks used by the runner's regression gate.

use super::cpp_files;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

/// Rejects a `*_variant` test whose source matches its main counterpart once
/// comments and whitespace are removed, and warns when only identifiers and
/// literals differ.
pub(super) fn check_variant_tests() -> usize {
    const PAIRS: &[(&str, &str)] = &[
        ("c_basics_variant", "c_basics"),
        ("classes_oop_variant", "classes_oop"),
        ("memory_model_variant", "memory_model"),
        ("mini_stl_variant", "mini_stl"),
    ];
    let mut failures = 0;
    for (variant, main) in PAIRS {
        let directory = Path::new("tests").join(variant);
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
            let variant_source = fs::read_to_string(&path).unwrap_or_default();
            let main_source = fs::read_to_string(&main_path).unwrap_or_default();
            match classify_pair(&main_source, &variant_source) {
                PairClass::Identical => {
                    eprintln!("[FAIL] duplicate variant test: {variant}/{name}");
                    failures += 1;
                }
                PairClass::IdentifierOrLiteralOnly => {
                    eprintln!(
                        "warning: variant test {variant}/{name} differs only in identifiers or literals"
                    );
                }
                PairClass::Distinct => {}
            }
        }
    }
    failures
}

#[derive(Debug, PartialEq, Eq)]
enum PairClass {
    Identical,
    IdentifierOrLiteralOnly,
    Distinct,
}

fn classify_pair(main: &str, variant: &str) -> PairClass {
    if normalized_source(main) == normalized_source(variant) {
        PairClass::Identical
    } else if skeleton_source(main) == skeleton_source(variant) {
        PairClass::IdentifierOrLiteralOnly
    } else {
        PairClass::Distinct
    }
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

#[cfg(test)]
mod tests {
    use super::{PairClass, classify_pair};

    #[test]
    fn identical_sources_fail_the_variant_check() {
        assert_eq!(
            classify_pair("int main() { return 0; }", "int main() { return 0; }"),
            PairClass::Identical
        );
    }

    #[test]
    fn whitespace_and_comment_only_changes_fail_the_variant_check() {
        let main = "int main() {\n  // explain\n  return 0;\n}\n";
        let variant = "int main(){/* same */return 0;}";
        assert_eq!(classify_pair(main, variant), PairClass::Identical);
    }

    #[test]
    fn identifier_and_literal_only_changes_warn() {
        let main = "int main() { int x = 1; return x; }";
        let variant = "int main() { int value = 2; return value; }";
        assert_eq!(
            classify_pair(main, variant),
            PairClass::IdentifierOrLiteralOnly
        );
    }

    #[test]
    fn structural_changes_pass_silently() {
        let main = "int main() { int x = 1; return x; }";
        let variant = "int main() { int x = 1; x += 1; return x; }";
        assert_eq!(classify_pair(main, variant), PairClass::Distinct);
    }
}
