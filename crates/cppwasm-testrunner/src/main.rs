//! Corpus runner: compiles every test with the selected compiler, executes the
//! resulting module through the JavaScript host, and checks the expected output.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

const FEATURES: &[&str] = &[
    "c_basics",
    "c_basics_heldout",
    "classes_oop",
    "classes_oop_heldout",
    "memory_model",
    "memory_model_heldout",
    "runtime_trap",
    "templates_operators",
    "mini_stl",
    "mini_stl_heldout",
];

const NODE_RUNNER: &str = r#"
const fs=require("fs"), path=require("path");
const {createWasmImports}=require(path.resolve(".","web/host.js"));
const bytes=fs.readFileSync(process.argv[1]);
const input=process.argv[2]&&fs.existsSync(process.argv[2])?fs.readFileSync(process.argv[2],"utf8"):"";
const host=createWasmImports(null,input,code=>{process.stdout.write(host.getStdout());process.exit(code)});
WebAssembly.instantiate(bytes,host.imports).then(({instance})=>{
  let ret=0;
  try { ret=instance.exports.main?instance.exports.main():instance.exports._start(); }
  catch(e){ console.error("WASM_TRAP: "+e.message); process.exit(134); }
  process.stdout.write(host.getStdout());
  if(ret) process.exit(ret);
}).catch(e=>{ console.error("WASM_TRAP: "+e.message); process.exit(134); });
"#;

struct Report {
    feature: String,
    passed: usize,
    failed: usize,
    executed: usize,
    on_disk: usize,
    failures: Vec<String>,
    passed_names: Vec<String>,
}

struct Paths {
    compiler: PathBuf,
    work: PathBuf,
}

impl Paths {
    fn output(&self, feature: &str, test: &str, extension: &str) -> PathBuf {
        self.work.join(feature).join(format!("{test}.{extension}"))
    }
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let compiler = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/release/cppwasm-cli"));
    let filter = args.next();

    let work = std::env::temp_dir().join(format!("cppwasm-corpus-{}", std::process::id()));
    if let Err(error) = fs::create_dir_all(&work) {
        eprintln!("cannot create {}: {error}", work.display());
        return ExitCode::FAILURE;
    }
    let paths = Paths { compiler, work };

    let mut reports = Vec::new();
    let groups: Vec<&str> = match filter.as_deref() {
        Some("all") | None => FEATURES.to_vec(),
        Some("negative") => Vec::new(),
        Some(feature) => vec![feature],
    };
    for feature in groups {
        reports.push(run_feature(&paths, feature, false));
    }
    let reference_mode = paths.compiler == Path::new("ref");
    if !reference_mode && matches!(filter.as_deref(), Some("all") | Some("negative") | None) {
        reports.push(run_feature(&paths, "negative", true));
    }

    for report in &reports {
        println!("\nFeature Group: [{}]", report.feature);
        for failure in &report.failures {
            println!("  [FAIL] {failure}");
        }
        println!(
            "Summary for {}: on-disk {}, executed {}, passed {}, failed {}.",
            report.feature, report.on_disk, report.executed, report.passed, report.failed
        );
    }

    let on_disk: usize = reports.iter().map(|report| report.on_disk).sum();
    let executed: usize = reports.iter().map(|report| report.executed).sum();
    let passed: usize = reports.iter().map(|report| report.passed).sum();
    let failed: usize = reports.iter().map(|report| report.failed).sum();
    println!(
        "\nOverall: on-disk {on_disk}, executed {executed}, passed {passed}, failed {failed}."
    );

    let manifest_failures = check_manifest(&reports, reference_mode);
    let _ = fs::remove_dir_all(&paths.work);
    if manifest_failures == 0 && executed == on_disk && failed == 0 {
        ExitCode::SUCCESS
    } else {
        if manifest_failures > 0 {
            eprintln!("{manifest_failures} PASSING manifest entries did not pass");
        }
        ExitCode::FAILURE
    }
}

fn run_feature(paths: &Paths, feature: &str, negative: bool) -> Report {
    let directory = Path::new("tests").join(feature);
    let mut tests = match cpp_files(&directory) {
        Ok(tests) => tests,
        Err(error) => {
            return Report {
                feature: feature.to_string(),
                passed: 0,
                failed: 1,
                executed: 1,
                on_disk: 0,
                failures: vec![format!("cannot list {}: {error}", directory.display())],
                passed_names: Vec::new(),
            };
        }
    };
    tests.sort();
    let on_disk = tests.len();
    let mut report = Report {
        feature: feature.to_string(),
        passed: 0,
        failed: 0,
        executed: 0,
        on_disk,
        failures: Vec::new(),
        passed_names: Vec::new(),
    };

    for cpp in tests {
        let name = cpp
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_string();
        report.executed += 1;
        let result = if negative {
            check_negative(paths, feature, &name, &cpp)
        } else {
            check_positive(paths, feature, &name, &cpp)
        };
        match result {
            Ok(()) => {
                report.passed += 1;
                report.passed_names.push(name);
            }
            Err(message) => {
                report.failed += 1;
                report.failures.push(format!("{name} {message}"));
            }
        }
    }
    report
}

fn cpp_files(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension() == Some(OsStr::new("cpp")) {
            files.push(path);
        }
    }
    Ok(files)
}

fn check_positive(paths: &Paths, feature: &str, name: &str, cpp: &Path) -> Result<(), String> {
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

fn check_negative(paths: &Paths, feature: &str, name: &str, cpp: &Path) -> Result<(), String> {
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

fn input_path(feature: &str, name: &str) -> Option<PathBuf> {
    let path = Path::new("tests")
        .join(feature)
        .join(format!("{name}.input"));
    path.exists().then_some(path)
}

fn run_wasm(wasm: &Path, input: &Option<PathBuf>) -> std::io::Result<std::process::Output> {
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

fn run_with_input(binary: &Path, input: &Option<PathBuf>) -> std::io::Result<std::process::Output> {
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

fn compare_output(output: &std::process::Output, feature: &str, name: &str) -> Result<(), String> {
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

fn check_manifest(reports: &[Report], reference_mode: bool) -> usize {
    let manifest = match fs::read_to_string("tests/PASSING.txt") {
        Ok(manifest) => manifest,
        Err(error) => {
            eprintln!("cannot read tests/PASSING.txt: {error}");
            return 1;
        }
    };
    let passed: BTreeSet<(&str, &str)> = reports
        .iter()
        .flat_map(|report| {
            report
                .passed_names
                .iter()
                .map(|name| (report.feature.as_str(), name.as_str()))
        })
        .collect();

    let mut missing = 0;
    for line in manifest.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((feature, name)) = line.split_once('/') else {
            eprintln!("malformed PASSING entry: {line}");
            missing += 1;
            continue;
        };
        if reference_mode && feature == "negative" {
            continue;
        }
        if !passed.contains(&(feature, name)) {
            eprintln!("PASSING entry did not pass: {line}");
            missing += 1;
        }
    }
    missing
}
