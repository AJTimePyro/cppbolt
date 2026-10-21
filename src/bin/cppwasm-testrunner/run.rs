//! Running the corpus tests and comparing their output.

use super::*;
use std::io::Write;
use std::process::{Command, Stdio};

pub(super) fn cpp_files(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension() == Some(OsStr::new("cpp")) {
            files.push(path);
        }
    }
    Ok(files)
}

pub(super) fn check_positive(
    paths: &Paths,
    feature: &str,
    name: &str,
    cpp: &Path,
) -> Result<(), String> {
    let feature_dir = paths.work.join(feature);
    fs::create_dir_all(&feature_dir).map_err(|error| error.to_string())?;

    if paths.compiler == Path::new("ref") {
        let native = paths.output(feature, name, "native");
        let compiled = Command::new("timeout")
            .arg("5s")
            .arg("g++")
            .arg("-std=c++17")
            .arg("-O0")
            .arg(cpp)
            .arg("-o")
            .arg(&native)
            .output()
            .map_err(|error| format!("cannot run g++: {error}"))?;
        if !compiled.status.success() {
            return Err("compilation error or timeout".to_string());
        }
        let output = run_with_input(&native, &input_path(feature, name))
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
    let compiled = Command::new("timeout")
        .arg("5s")
        .arg(&paths.compiler)
        .arg(cpp)
        .arg("-o")
        .arg(&wasm)
        .output()
        .map_err(|error| format!("cannot run compiler: {error}"))?;
    if !compiled.status.success() {
        return Err("compilation error or timeout".to_string());
    }
    let output = run_wasm(&wasm, &input_path(feature, name))
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
    let feature_dir = paths.work.join(feature);
    fs::create_dir_all(&feature_dir).map_err(|error| error.to_string())?;
    let wasm = paths.output(feature, name, "wasm");
    let compiled = Command::new("timeout")
        .arg("5s")
        .arg(&paths.compiler)
        .arg(cpp)
        .arg("-o")
        .arg(&wasm)
        .output()
        .map_err(|error| format!("cannot run compiler: {error}"))?;

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

pub(super) fn input_path(feature: &str, name: &str) -> Option<PathBuf> {
    let path = Path::new("tests")
        .join(feature)
        .join(format!("{name}.input"));
    path.exists().then_some(path)
}

pub(super) fn run_wasm(
    wasm: &Path,
    input: &Option<PathBuf>,
) -> std::io::Result<std::process::Output> {
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

pub(super) fn run_with_input(
    binary: &Path,
    input: &Option<PathBuf>,
) -> std::io::Result<std::process::Output> {
    let mut command = Command::new("timeout");
    command.arg("5s").arg(binary);
    match input {
        Some(path) => {
            let text = fs::read(path)?;
            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(&text)?;
            }
            child.wait_with_output()
        }
        None => command.output(),
    }
}

pub(super) fn compare_output(
    output: &std::process::Output,
    feature: &str,
    name: &str,
) -> Result<(), String> {
    if !output.status.success() {
        return Err(format!("runtime exit {}", output.status));
    }
    let expected_path = Path::new("tests")
        .join(feature)
        .join(format!("{name}.expected"));
    let expected = fs::read(&expected_path)
        .map_err(|error| format!("cannot read {}: {error}", expected_path.display()))?;
    let mut actual = output.stdout.clone();
    actual.extend_from_slice(&output.stderr);
    if actual == expected {
        Ok(())
    } else {
        Err("output mismatch".to_string())
    }
}
