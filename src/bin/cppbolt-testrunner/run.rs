//! Running the corpus tests and comparing their output.

use super::*;
use cppbolt::corpus::{compare_output, input_path, run_with_input};
use std::process::Command;

fn ensure_feature_dir(paths: &Paths, feature: &str) -> Result<(), String> {
    fs::create_dir_all(paths.work.join(feature)).map_err(|error| error.to_string())
}

fn run_command(
    label: &str,
    program: &Path,
    extra: &[&str],
    cpp: &Path,
    out: &Path,
) -> Result<std::process::Output, String> {
    if let Some(parent) = out.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut command = Command::new("timeout");
    command.arg("5s").arg(program);
    for arg in extra {
        command.arg(arg);
    }
    command
        .arg(cpp)
        .arg("-o")
        .arg(out)
        .output()
        .map_err(|error| format!("cannot run {label}: {error}"))
}

pub(super) fn check_positive(
    paths: &Paths,
    feature: &str,
    name: &str,
    cpp: &Path,
) -> Result<(), String> {
    ensure_feature_dir(paths, feature)?;

    if paths.compiler == Path::new("ref") {
        let native = paths.output(feature, name, "native");
        let compiled = run_command(
            "g++",
            Path::new("g++"),
            &["-std=c++17", "-O0"],
            cpp,
            &native,
        )?;
        if !compiled.status.success() {
            return Err("compilation error or timeout".to_string());
        }
        let output = run_with_input(&native, input_path(feature, name).as_deref())
            .map_err(|error| format!("cannot run native binary: {error}"))?;
        if feature == "runtime_trap" {
            return if output.status.success() {
                Err("expected native trap".to_string())
            } else {
                Ok(())
            };
        }
        return compare_output(&output, feature, name);
    }

    let wasm = paths.output(feature, name, "wasm");
    let compiled = run_command("compiler", &paths.compiler, &[], cpp, &wasm)?;
    if !compiled.status.success() {
        return Err("compilation error or timeout".to_string());
    }
    let output = run_wasm(&wasm, input_path(feature, name).as_deref())
        .map_err(|error| format!("cannot run node: {error}"))?;

    if feature == "runtime_trap" {
        let trapped = !output.status.success()
            || String::from_utf8_lossy(&output.stderr).contains("WASM_TRAP");
        return if trapped {
            Ok(())
        } else {
            Err("expected wasm trap".to_string())
        };
    }
    compare_output(&output, feature, name)
}

pub(super) fn check_negative(
    paths: &Paths,
    feature: &str,
    name: &str,
    cpp: &Path,
) -> Result<(), String> {
    ensure_feature_dir(paths, feature)?;
    let wasm = paths.output(feature, name, "wasm");
    let compiled = run_command("compiler", &paths.compiler, &[], cpp, &wasm)?;

    if compiled.status.success() {
        return Err("diagnostic mismatch or accepted".to_string());
    }
    let expected_path = Path::new("tests")
        .join(feature)
        .join(format!("{name}.expected_err"));
    let expected = fs::read_to_string(&expected_path)
        .map_err(|error| format!("cannot read {}: {error}", expected_path.display()))?;
    let expected = expected.lines().next().unwrap_or_default();
    let actual = String::from_utf8_lossy(&compiled.stderr);
    let actual = actual.lines().next().unwrap_or_default();
    if actual == expected {
        Ok(())
    } else {
        Err(format!("diagnostic mismatch: {actual}"))
    }
}

pub(super) fn run_wasm(wasm: &Path, input: Option<&Path>) -> std::io::Result<std::process::Output> {
    let mut command = Command::new("timeout");
    command
        .arg("5s")
        .arg("node")
        .arg("-e")
        .arg(NODE_RUNNER)
        .arg(wasm);
    if let Some(input) = input {
        command.arg(input);
    }
    command.output()
}
