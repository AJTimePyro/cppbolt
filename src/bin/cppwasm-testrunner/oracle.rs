//! Real-compiler oracle gate: replays every positive test through `g++` and
//! `clang++` and checks that their native output matches the recorded
//! `.expected` file. Compilers that are not installed are skipped.

use super::FEATURES;
use super::run::{compare_output, cpp_files, input_path, run_with_input};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

pub(super) fn run_oracle(filter: Option<&str>) -> ExitCode {
    let compilers: Vec<&str> = ["g++", "clang++"]
        .into_iter()
        .filter(|compiler| tool_available(compiler))
        .collect();
    if compilers.is_empty() {
        eprintln!("oracle gate skipped: neither g++ nor clang++ is available");
        return ExitCode::SUCCESS;
    }

    let groups: Vec<&str> = match filter {
        Some("all") | None => FEATURES.to_vec(),
        Some("negative") => Vec::new(),
        Some(feature) => vec![feature],
    };

    let work = std::env::temp_dir().join(format!("cppwasm-oracle-{}", std::process::id()));
    if let Err(error) = fs::create_dir_all(&work) {
        eprintln!("cannot create {}: {error}", work.display());
        return ExitCode::FAILURE;
    }

    let mut matched = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for compiler in &compilers {
        for feature in &groups {
            let directory = Path::new("tests").join(feature);
            let tests = match cpp_files(&directory) {
                Ok(tests) => tests,
                Err(error) => {
                    eprintln!("[FAIL] cannot list {}: {error}", directory.display());
                    failed += 1;
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
                    OracleResult::Passed => matched += 1,
                    OracleResult::Skipped => skipped += 1,
                    OracleResult::Failed(message) => {
                        eprintln!("[FAIL] {compiler} {feature}/{name}: {message}");
                        failed += 1;
                    }
                }
            }
        }
    }

    let _ = fs::remove_dir_all(&work);
    println!(
        "\nOracle: {matched} matched, {failed} mismatched, {skipped} skipped across {} compiler(s).",
        compilers.len()
    );
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

enum OracleResult {
    Passed,
    Skipped,
    Failed(String),
}

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

fn tool_available(compiler: &str) -> bool {
    Command::new(compiler)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
