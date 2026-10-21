//! Every diagnostic in the corpus must name a real source location.
//!
//! A diagnostic that reports line 1, column 1 is only correct when the
//! failing construct really is the first token of the file, so the corpus
//! keeps an explicit list of the files where that is the case.  Every other
//! diagnostic must point at a line that exists in the file it names, which
//! rules out a fabricated location.

use cppwasm::{CompileError, IncludeProvider, ResolvedInclude, compile_with_provider};
use std::fs;
use std::path::{Path, PathBuf};

/// The corpus files whose diagnostic really is at line 1, column 1.
///
/// Each entry names a file whose first token is the construct that failed:
/// a `#error` directive, a stray `#endif`, or the `#include` line of a header
/// that takes part in an include cycle.
const FIRST_POSITION_FILES: &[&str] = &[
    "tests/negative/error_directive.cpp",
    "tests/negative/include_cycle_b.h",
    "tests/negative/stray_endif.cpp",
];

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
/// A file without a directory is one of the embedded headers, which live in
/// `stdlib/`.
fn reported_path(file: &str) -> PathBuf {
    if file.contains('/') {
        PathBuf::from(file)
    } else {
        Path::new("stdlib").join(file)
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
            if !FIRST_POSITION_FILES.contains(&error.file.as_str()) {
                failures.push(format!(
                    "{name}: the diagnostic of {} reports 1:1, but the file is not one of \
                     the files whose error really is at the first position",
                    error.file
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
