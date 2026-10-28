//! Variant and independent corpus checks used by the runner's regression gate.

use super::golden::sha256_hex;
use cppwasm::corpus::{cpp_files, test_name};
use std::collections::BTreeMap;
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
            Err(error) => {
                eprintln!("[FAIL] cannot list {}: {error}", directory.display());
                failures += 1;
                continue;
            }
        };
        for path in files {
            let Some(name) = test_name(&directory, &path) else {
                eprintln!("[FAIL] non-UTF-8 variant file name: {}", path.display());
                failures += 1;
                continue;
            };
            let main_path = Path::new("tests").join(main).join(format!("{name}.cpp"));
            if !main_path.exists() {
                continue;
            }
            let variant_source = match fs::read_to_string(&path) {
                Ok(text) => text,
                Err(error) => {
                    eprintln!("[FAIL] cannot read {}: {error}", path.display());
                    failures += 1;
                    continue;
                }
            };
            let main_source = match fs::read_to_string(&main_path) {
                Ok(text) => text,
                Err(error) => {
                    eprintln!("[FAIL] cannot read {}: {error}", main_path.display());
                    failures += 1;
                    continue;
                }
            };
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

/// Verifies that every file in `tests/independent` still matches the committed
/// hash list, and that no unlisted file has appeared.
pub(super) fn check_independent_manifest() -> usize {
    let root = Path::new("tests/independent");
    let manifest_path = root.join("MANIFEST.sha256");
    let text = match fs::read_to_string(&manifest_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("cannot read {}: {error}", manifest_path.display());
            return 1;
        }
    };

    let (expected, mut failures) = parse_independent_manifest(&text);

    let mut actual = BTreeMap::new();
    if let Err(error) = collect_independent_files(root, root, &mut actual) {
        eprintln!(
            "[FAIL] independent manifest: cannot inventory {}: {error}",
            root.display()
        );
        return failures + 1;
    }

    for (path, hash) in &expected {
        match actual.get(path) {
            Some(actual_hash) if actual_hash == hash => {}
            Some(_) => {
                eprintln!("[FAIL] independent file changed: tests/independent/{path}");
                failures += 1;
            }
            None => {
                eprintln!("[FAIL] independent file missing: tests/independent/{path}");
                failures += 1;
            }
        }
    }
    for path in actual.keys() {
        if !expected.contains_key(path) {
            eprintln!("[FAIL] independent file is not in the manifest: tests/independent/{path}");
            failures += 1;
        }
    }

    if failures == 0 {
        let (noun, verb) = if actual.len() == 1 {
            ("file", "matches")
        } else {
            ("files", "match")
        };
        println!(
            "Independent manifest: {} {noun} {verb} {}.",
            actual.len(),
            manifest_path.display()
        );
    }
    failures
}

/// Parses the committed manifest of the independent test files.
fn parse_independent_manifest(text: &str) -> (BTreeMap<String, String>, usize) {
    let mut failures = 0;
    let mut expected = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() != 2 {
            eprintln!("[FAIL] independent manifest: malformed entry: {line}");
            failures += 1;
            continue;
        }
        let path = fields[1]
            .strip_prefix("./")
            .unwrap_or(fields[1])
            .replace('\\', "/");
        if path == "MANIFEST.sha256" {
            eprintln!("[FAIL] independent manifest: the manifest cannot list itself");
            failures += 1;
            continue;
        }
        if path.starts_with('/') || path.split('/').any(|part| part == "..") {
            eprintln!("[FAIL] independent manifest: path escapes the directory: {path}");
            failures += 1;
            continue;
        }
        if expected
            .insert(path.clone(), fields[0].to_string())
            .is_some()
        {
            eprintln!("[FAIL] independent manifest: duplicate entry: {path}");
            failures += 1;
        }
    }

    (expected, failures)
}

fn collect_independent_files(
    root: &Path,
    directory: &Path,
    files: &mut BTreeMap<String, String>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_independent_files(root, &path, files)?;
        } else if path.file_name() != Some(OsStr::new("MANIFEST.sha256")) {
            let relative = match path.strip_prefix(root) {
                Ok(relative) => relative.to_string_lossy().replace('\\', "/"),
                Err(error) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("{}: {error}", path.display()),
                    ));
                }
            };
            files.insert(relative, sha256_hex(&fs::read(&path)?));
        }
    }
    Ok(())
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
