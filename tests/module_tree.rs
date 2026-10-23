//! Fails when a source file under `src/` is not reachable from the module
//! tree.
//!
//! Unreachable files are dead weight: they are never compiled, never linted
//! and never exercised, so they can drift away from the code they were
//! extracted from and stop building without anyone noticing.
//!
//! The check walks the module tree from the crate roots, resolving every
//! `mod name;` declaration the way rustc does (including `#[path = "..."]`),
//! and then requires that the set of visited files is exactly the set of
//! `.rs` files under `src/`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The package root, i.e. the directory that holds `src/` and `tests/`.
fn package_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Replaces comments and string literals with spaces so that a `mod` inside
/// a comment or a string is not mistaken for a declaration.
fn strip_comments_and_strings(text: &str) -> Vec<char> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<char> = chars.clone();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                out[i] = ' ';
                i += 1;
            }
        } else if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            let mut depth = 0;
            while i < chars.len() {
                if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                    depth += 1;
                    out[i] = ' ';
                    out[i + 1] = ' ';
                    i += 2;
                } else if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    depth -= 1;
                    out[i] = ' ';
                    out[i + 1] = ' ';
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    if chars[i] != '\n' {
                        out[i] = ' ';
                    }
                    i += 1;
                }
            }
        } else if chars[i] == '"' {
            out[i] = ' ';
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    out[i] = ' ';
                    out[i + 1] = ' ';
                    i += 2;
                    continue;
                }
                if chars[i] == '"' {
                    out[i] = ' ';
                    i += 1;
                    break;
                }
                if chars[i] != '\n' {
                    out[i] = ' ';
                }
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    out
}

/// One `mod name;` declaration and the `#[path]` attribute that may precede
/// it.
struct Declaration {
    name: String,
    path_attribute: Option<String>,
}

/// Collects the `mod name;` declarations of one file.
///
/// A declaration is the keyword `mod` followed by a name and a `;`.  Inline
/// module bodies (`mod name { ... }`) are skipped: they do not name a file.
fn module_declarations(text: &str) -> Vec<Declaration> {
    let bytes = strip_comments_and_strings(text);
    let original: Vec<char> = text.chars().collect();
    let mut declarations = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let is_start = i == 0 || !(bytes[i - 1].is_alphanumeric() || bytes[i - 1] == '_');
        if !(is_start && bytes[i..].starts_with(&['m', 'o', 'd'])) {
            i += 1;
            continue;
        }
        let after = i + 3;
        // `mod` must be a keyword of its own: `module` is an identifier.
        if after >= bytes.len() || bytes[after].is_alphanumeric() || bytes[after] == '_' {
            i = after;
            continue;
        }
        let path_attribute = preceding_path_attribute(&original, i);
        let mut j = after;
        while j < bytes.len() && bytes[j].is_whitespace() {
            j += 1;
        }
        let name_start = j;
        while j < bytes.len() && (bytes[j].is_alphanumeric() || bytes[j] == '_') {
            j += 1;
        }
        if j == name_start {
            i = j.max(after);
            continue;
        }
        let name: String = bytes[name_start..j].iter().collect();
        // The declaration ends at the first `;` before any inline body.
        let mut k = j;
        let mut next_index = None;
        while k < bytes.len() {
            if bytes[k] == ';' {
                declarations.push(Declaration {
                    name,
                    path_attribute,
                });
                next_index = Some(k + 1);
                break;
            }
            if bytes[k] == '{' || bytes[k] == '}' {
                // An inline module body: not a file declaration.
                break;
            }
            k += 1;
        }
        match next_index {
            Some(next) => i = next,
            None => {
                if k >= bytes.len() {
                    break;
                }
                i = j;
            }
        }
    }
    declarations
}

/// Returns the file named by a `#[path = "..."]` attribute that precedes the
/// `mod` keyword at `index`, if there is one.
fn preceding_path_attribute(bytes: &[char], index: usize) -> Option<String> {
    let mut i = index;
    while i > 0 && bytes[i - 1].is_whitespace() {
        i -= 1;
    }
    if i == 0 || bytes[i - 1] != ']' {
        return None;
    }
    let close = i - 1;
    let mut start = close;
    while start > 0 && bytes[start - 1] != '[' {
        start -= 1;
    }
    if start == 0 {
        return None;
    }
    let attribute: String = bytes[start..=close].iter().collect();
    if !attribute.contains("path") {
        return None;
    }
    let quote_start = attribute.find('"')?;
    let quote_end = attribute[quote_start + 1..].find('"')? + quote_start + 1;
    Some(attribute[quote_start + 1..quote_end].to_string())
}

/// Resolves `mod name;` declared inside `current` the way rustc does.
fn resolve_module(current: &Path, declaration: &Declaration) -> Option<PathBuf> {
    let dir = current.parent()?;
    if let Some(path) = &declaration.path_attribute {
        let candidate = dir.join(path);
        if candidate.is_file() {
            return Some(candidate);
        }
        return None;
    }
    let flat = dir.join(format!("{}.rs", declaration.name));
    if flat.is_file() {
        return Some(flat);
    }
    let nested = dir.join(&declaration.name).join("mod.rs");
    if nested.is_file() {
        return Some(nested);
    }
    None
}

/// The crate roots: the library, the binary and every extra binary.
fn roots(src: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut roots = Vec::new();
    for name in ["lib.rs", "main.rs"] {
        let path = src.join(name);
        if path.is_file() {
            roots.push(path);
        }
    }
    let bin = src.join("bin");
    if bin.is_dir() {
        for entry in std::fs::read_dir(&bin)? {
            let path = entry?.path();
            if path.extension().is_some_and(|ext| ext == "rs") {
                roots.push(path);
            }
        }
    }
    Ok(roots)
}

/// Every `.rs` file under `src/`.
fn source_files(src: &Path) -> std::io::Result<BTreeSet<PathBuf>> {
    let mut all: BTreeSet<PathBuf> = BTreeSet::new();
    let mut stack = vec![src.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                all.insert(path);
            }
        }
    }
    Ok(all)
}

#[test]
fn every_source_file_is_reachable_from_the_module_tree() -> Result<(), String> {
    let root = package_root();
    let src = root.join("src");

    let mut visited: BTreeSet<PathBuf> = BTreeSet::new();
    let mut pending: Vec<PathBuf> = roots(&src).map_err(|error| error.to_string())?;
    let mut missing: Vec<String> = Vec::new();

    while let Some(current) = pending.pop() {
        if !visited.insert(current.clone()) {
            continue;
        }
        let text = std::fs::read_to_string(&current)
            .map_err(|error| format!("read {}: {error}", current.display()))?;
        for declaration in module_declarations(&text) {
            match resolve_module(&current, &declaration) {
                Some(path) => pending.push(path),
                None => missing.push(format!(
                    "{}: `mod {};` does not resolve to a file",
                    current.display(),
                    declaration.name
                )),
            }
        }
    }

    assert!(
        missing.is_empty(),
        "unresolved module declarations:\n{}",
        missing.join("\n")
    );

    // Every `.rs` file under `src/` must have been reached.
    let all = source_files(&src).map_err(|error| error.to_string())?;
    let orphans: Vec<&PathBuf> = all.difference(&visited).collect();
    assert!(
        orphans.is_empty(),
        "source files that no `mod` declaration reaches (delete them or wire \
         them into the module tree):\n{}",
        orphans
            .iter()
            .map(|path| format!("  {}", path.display()))
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(())
}
