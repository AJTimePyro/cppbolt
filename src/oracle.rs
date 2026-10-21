//! The native oracle gate: replays every positive test through the real
//! compilers and checks that their output matches the recorded expectation.
//!
//! The recorded `.expected` files are the reference of the corpus.  Without
//! this gate a wrong expectation would hide a behavior change, so the gate
//! runs both in the corpus runner and as a cargo test.

use crate::corpus::{compare_output, cpp_files, input_path, run_with_input};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The compilers the gate replays the corpus through.
pub const COMPILERS: &[&str] = &["g++", "clang++"];

/// The outcome of the oracle gate.
#[derive(Debug, Default)]
pub struct OracleReport {
    /// The compilers that ran the corpus.
    pub compilers: Vec<String>,
    /// The tests whose output matched the expectation.
    pub matched: usize,
    /// The tests whose output did not match the expectation.
    pub mismatched: usize,
    /// The tests that were skipped.
    pub skipped: usize,
    /// One entry per test that did not match.
    pub failures: Vec<String>,
}

/// The outcome of one test.
enum OracleResult {
    Passed,
    Skipped,
    Failed(String),
}

/// Replays the positive tests of the given groups through the real compilers.
pub fn run_oracle(groups: &[&str]) -> OracleReport {
    let mut report = OracleReport::default();
    for compiler in COMPILERS {
        if !tool_available(compiler) {
            continue;
        }
        report.compilers.push((*compiler).to_string());
    }
    if report.compilers.is_empty() {
        return report;
    }

    let work = std::env::temp_dir().join(format!("cppwasm-oracle-{}", std::process::id()));
    if fs::create_dir_all(&work).is_err() {
        report.mismatched += 1;
        report.failures.push(format!(
            "cannot create the work directory {}",
            work.display()
        ));
        return report;
    }

    for compiler in &report.compilers {
        for feature in groups {
            let directory = Path::new("tests").join(feature);
            let tests = match cpp_files(&directory) {
                Ok(tests) => tests,
                Err(error) => {
                    report.mismatched += 1;
                    report
                        .failures
                        .push(format!("cannot list {}: {error}", directory.display()));
                    continue;
                }
            };
            for cpp in tests {
                let name = cpp
                    .file_stem()
                    .and_then(OsStr::to_str)
                    .unwrap_or_default()
                    .to_string();
                match run_one(compiler, &work, feature, &name, &cpp) {
                    OracleResult::Passed => report.matched += 1,
                    OracleResult::Skipped => report.skipped += 1,
                    OracleResult::Failed(message) => {
                        report.mismatched += 1;
                        report
                            .failures
                            .push(format!("{compiler} {feature}/{name}: {message}"));
                    }
                }
            }
        }
    }

    let _ = fs::remove_dir_all(&work);
    report
}

/// Replays one test through one compiler.
fn run_one(compiler: &str, work: &Path, feature: &str, name: &str, cpp: &Path) -> OracleResult {
    let binary: PathBuf = work.join(format!("{compiler}_{feature}_{name}"));
    let compiled = Command::new("timeout")
        .arg("10s")
        .arg(compiler)
        .arg("-std=c++17")
        .arg("-O0")
        .arg(cpp)
        .arg("-o")
        .arg(&binary)
        .output();
    let compiled = match compiled {
        Ok(compiled) => compiled,
        Err(error) => return OracleResult::Failed(format!("cannot run {compiler}: {error}")),
    };
    if !compiled.status.success() {
        return OracleResult::Failed("compilation error or timeout".to_string());
    }

    let output = match run_with_input(&binary, &input_path(feature, name)) {
        Ok(output) => output,
        Err(error) => return OracleResult::Failed(format!("cannot run binary: {error}")),
    };
    let _ = fs::remove_file(&binary);

    if feature == "runtime_trap" {
        // Native compilers may legitimately not trap where the WebAssembly
        // runtime does, so a clean exit is a skip rather than a failure.
        return if output.status.success() {
            OracleResult::Skipped
        } else {
            OracleResult::Passed
        };
    }

    match compare_output(&output, feature, name) {
        Ok(()) => OracleResult::Passed,
        Err(message) => OracleResult::Failed(message),
    }
}

/// Returns true when a compiler is installed.
fn tool_available(compiler: &str) -> bool {
    Command::new(compiler)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
