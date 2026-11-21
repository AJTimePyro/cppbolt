//! Every diagnostic in the corpus must name a real source location.
//!
//! A diagnostic that reports line 1, column 1 is only correct when the
//! failing construct really is the first token of the file, so the corpus
//! keeps an explicit list of the diagnostics where that is the case.  Every
//! other diagnostic must point at a line that exists in the file it names,
//! which rules out a fabricated location.

use cppbolt::{CompileError, IncludeProvider, ResolvedInclude, compile_with_provider};
use std::fs;
use std::path::{Path, PathBuf};

/// The corpus diagnostics that really are at line 1, column 1.
///
/// Every entry names the file and the exact text of the diagnostic, so an
/// exemption cannot hide another error of the same file:
///
/// - `tests/negative/error_directive.cpp`: the `#error` directive is the
///   first token of the file.
/// - `tests/negative/include_cycle_b.h`: the `#include` line of the header
///   that closes the cycle is the first line of the file.
/// - `tests/negative/stray_endif.cpp`: the stray `#endif` is the first token
///   of the file.
/// - `tests/negative/namespace_alias_later_variable.cpp`: the failing `using`
///   declaration is the first token of the file. Its target names an
///   undeclared identifier; expression identifiers carry no token position, so
///   the diagnostic anchors to the declaration that failed to resolve.
const FIRST_POSITION_DIAGNOSTICS: &[(&str, &str)] = &[
    ("tests/negative/error_directive.cpp", "#error stop here"),
    (
        "tests/negative/include_cycle_b.h",
        "include cycle detected: 'tests/negative/include_cycle_a.h'",
    ),
    ("tests/negative/stray_endif.cpp", "#endif without #if"),
    (
        "tests/negative/namespace_alias_later_variable.cpp",
        "Undeclared variable or identifier 'later_var'",
    ),
];

/// Returns true when a diagnostic is one of the exempted ones.
fn is_exempted(file: &str, message: &str) -> bool {
    FIRST_POSITION_DIAGNOSTICS
        .iter()
        .any(|(exempted_file, exempted_message)| {
            *exempted_file == file && *exempted_message == message
        })
}

/// Reads includes from the filesystem, like the command-line compiler does.
#[derive(Clone)]
struct RepoIncludes;

impl IncludeProvider for RepoIncludes {
    fn read_include(&self, name: &str) -> Option<String> {
        fs::read_to_string(name).ok()
    }

    fn resolve_include(
        &self,
        name: &str,
        including_file: Option<&str>,
        quoted: bool,
    ) -> Option<ResolvedInclude> {
        let mut candidates = Vec::new();
        if quoted
            && let Some(including) = including_file
            && let Some(parent) = Path::new(including).parent()
            && !parent.as_os_str().is_empty()
        {
            candidates.push(parent.join(name));
        }
        candidates.push(PathBuf::from(name));
        for path in candidates {
            if let Ok(content) = fs::read_to_string(&path) {
                return Some(ResolvedInclude {
                    name: path.to_string_lossy().into_owned(),
                    content,
                });
            }
        }
        None
    }
}

/// Collects every C++ source file of the corpus, relative to the repository.
fn corpus_sources() -> Vec<PathBuf> {
    let mut sources = Vec::new();
    collect_sources(Path::new("tests"), &mut sources);
    sources.sort();
    sources
}

/// Collects the C++ sources of one directory.
fn collect_sources(directory: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_sources(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "cpp") {
            out.push(path);
        }
    }
}

/// Compiles one source on a thread with a large stack.
///
/// The compiler uses one recursion limit for every host, so the check runs on
/// the same stack size as the command-line compiler.
fn compile(path: &Path) -> Result<Result<Vec<u8>, CompileError>, String> {
    let source = fs::read_to_string(path).unwrap_or_default();
    let name = path.to_string_lossy().into_owned();
    let handle = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || compile_with_provider(&source, &name, 256, &RepoIncludes))
        .map_err(|error| format!("cannot start the compiler thread: {error}"))?;
    Ok(handle
        .join()
        .unwrap_or_else(|_| Err(CompileError::new("compiler thread panicked", 1, 1))))
}

/// Returns the file a diagnostic names.
///
/// Embedded headers live in `stdlib/`.
fn reported_path(file: &str) -> PathBuf {
    let stdlib_candidate = Path::new("stdlib").join(file);
    if stdlib_candidate.exists() {
        stdlib_candidate
    } else {
        PathBuf::from(file)
    }
}

/// Returns the number of lines of a file.
fn line_count(path: &Path) -> usize {
    fs::read_to_string(path).unwrap_or_default().lines().count()
}

#[test]
fn every_corpus_diagnostic_names_a_real_position() -> Result<(), Box<dyn std::error::Error>> {
    let sources = corpus_sources();
    assert!(!sources.is_empty(), "the corpus is empty");

    let mut failures = Vec::new();
    for path in &sources {
        let result = compile(path)?;
        let Err(error) = result else {
            continue;
        };
        let name = path.to_string_lossy().into_owned();
        if error.file.is_empty() {
            failures.push(format!("{name}: the diagnostic names no file"));
            continue;
        }
        if error.line == 1 && error.col == 1 {
            if !is_exempted(&error.file, &error.message) {
                failures.push(format!(
                    "{name}: the diagnostic of {} reports 1:1, but it is not one of the \
                     exempted diagnostics: {:?}",
                    error.file, error.message
                ));
            }
            continue;
        }
        let reported = reported_path(&error.file);
        if !reported.exists() {
            failures.push(format!(
                "{name}: the diagnostic names the missing file {}",
                error.file
            ));
            continue;
        }
        let lines = line_count(&reported);
        if error.line == 0 || error.line > lines {
            failures.push(format!(
                "{name}: the diagnostic reports line {} of {}, which has {lines} lines",
                error.line, error.file
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "corpus diagnostics with a fabricated location:\n{}",
        failures.join("\n")
    );
    Ok(())
}

/// Compiles an inline source and returns the diagnostic's position fields.
fn diagnostic_position(source: &str, name: &str) -> Option<(String, usize, usize)> {
    let provider = RepoIncludes;
    compile_with_provider(source, name, 256, &provider)
        .err()
        .map(|error| (error.file, error.line, error.col))
}

#[test]
fn namespace_alias_errors_report_the_alias_declaration_position() {
    let source = "struct NoType {};\n\
                  template <class T> struct Maybe { typedef T type; };\n\
                  template <> struct Maybe<NoType> {};\n\
                  using Invalid = Maybe<NoType>::type;\n\
                  int main() {}\n";
    assert_eq!(
        diagnostic_position(source, "alias.cpp"),
        Some(("alias.cpp".to_string(), 4, 1))
    );
}

#[test]
fn static_assert_failure_reports_the_condition_position() {
    let source = "constexpr int ok = 5;\n\
                  static_assert(ok == 6, \"mismatch\");\n\
                  int main() {}\n";
    // `ok` in `static_assert(ok == 6, ...)` begins at column 15.
    assert_eq!(
        diagnostic_position(source, "assert.cpp"),
        Some(("assert.cpp".to_string(), 2, 15))
    );
}
