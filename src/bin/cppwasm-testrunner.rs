//! Corpus runner: compiles every test with the selected compiler, executes the
//! resulting module through the JavaScript host, and checks the expected output.

#![forbid(unsafe_code)]

#[path = "cppwasm-testrunner/checks.rs"]
mod checks;
#[path = "cppwasm-testrunner/run.rs"]
mod run;

#[path = "cppwasm-testrunner/golden.rs"]
mod golden;

#[path = "cppwasm-testrunner/oracle.rs"]
mod oracle;

use cppwasm::corpus::{self, FEATURES};
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

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
    failed_names: Vec<String>,
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

/// Runs the golden-manifest mode when the first argument selects it.
fn run_golden_mode(
    first: Option<&str>,
    args: &mut impl Iterator<Item = String>,
) -> Option<ExitCode> {
    if !matches!(first, Some("--golden") | Some("--check-golden")) {
        return None;
    }
    let mode = if first == Some("--golden") {
        golden::Mode::Write
    } else {
        golden::Mode::Check
    };
    let Some(manifest) = args.next() else {
        eprintln!("Usage: cppwasm-testrunner --golden|--check-golden <manifest> [compiler]");
        return Some(ExitCode::FAILURE);
    };
    let compiler = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/release/cppwasm-cli"));
    Some(golden::run(mode, Path::new(&manifest), compiler))
}

fn main() -> ExitCode {
    if checks::check_independent_manifest() > 0 {
        return ExitCode::FAILURE;
    }

    let mut args = std::env::args().skip(1);
    let first = args.next();
    if first.as_deref() == Some("--oracle") {
        return oracle::run_oracle(args.next().as_deref());
    }
    if let Some(code) = run_golden_mode(first.as_deref(), &mut args) {
        return code;
    }
    let compiler = first
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/release/cppwasm-cli"));
    let filter = args.next();

    let work = std::env::temp_dir().join(format!("cppwasm-corpus-{}", std::process::id()));
    if let Err(error) = fs::create_dir_all(&work) {
        eprintln!("cannot create {}: {error}", work.display());
        return ExitCode::FAILURE;
    }
    let paths = Paths { compiler, work };

    let reports = run_groups(&paths, filter.as_deref());
    print_reports(&reports);
    let totals = Totals::of(&reports);

    let (expected_failures, unexpected_failures) = count_failures(&reports, totals.failed);
    if expected_failures > 0 {
        println!("Expected failures (not in tests/PASSING.txt): {expected_failures}.");
    }

    let manifest_failures = check_manifest(&reports, paths.compiler == Path::new("ref"));
    let duplicate_failures = checks::check_variant_tests();
    let _ = fs::remove_dir_all(&paths.work);
    if manifest_failures == 0
        && duplicate_failures == 0
        && totals.executed == totals.on_disk
        && unexpected_failures == 0
    {
        ExitCode::SUCCESS
    } else {
        if unexpected_failures > 0 {
            eprintln!("{unexpected_failures} unexpected test failures");
        }
        if manifest_failures > 0 {
            eprintln!("{manifest_failures} PASSING manifest violations");
        }
        if duplicate_failures > 0 {
            eprintln!("{duplicate_failures} variant tests duplicate their main counterpart");
        }
        ExitCode::FAILURE
    }
}

/// Counts the failures that the pass manifest expects and the ones it does
/// not.
fn count_failures(reports: &[Report], failed: usize) -> (usize, usize) {
    let listed = listed_tests();
    let mut expected_failures = 0;
    for report in reports {
        for name in &report.failed_names {
            if !listed.contains(&(report.feature.clone(), name.clone())) {
                expected_failures += 1;
            }
        }
    }
    (expected_failures, failed.saturating_sub(expected_failures))
}

/// The totals of every group that was run.
struct Totals {
    on_disk: usize,
    executed: usize,
    passed: usize,
    failed: usize,
}

impl Totals {
    fn of(reports: &[Report]) -> Self {
        Self {
            on_disk: reports.iter().map(|report| report.on_disk).sum(),
            executed: reports.iter().map(|report| report.executed).sum(),
            passed: reports.iter().map(|report| report.passed).sum(),
            failed: reports.iter().map(|report| report.failed).sum(),
        }
    }
}

/// Runs the selected feature groups.
fn run_groups(paths: &Paths, filter: Option<&str>) -> Vec<Report> {
    let mut reports = Vec::new();
    let groups: Vec<&str> = match filter {
        Some("all") | None => FEATURES.to_vec(),
        Some("negative") => Vec::new(),
        Some(feature) => vec![feature],
    };
    for feature in groups {
        reports.push(run_feature(paths, feature, false));
    }
    let reference_mode = paths.compiler == Path::new("ref");
    if !reference_mode && matches!(filter, Some("all") | Some("negative") | None) {
        reports.push(run_feature(paths, "negative", true));
    }
    reports
}

/// Prints the per-group summaries and the overall accounting.
fn print_reports(reports: &[Report]) {
    for report in reports {
        println!("\nFeature Group: [{}]", report.feature);
        for failure in &report.failures {
            println!("  [FAIL] {failure}");
        }
        println!(
            "Summary for {}: on-disk {}, executed {}, passed {}, failed {}.",
            report.feature, report.on_disk, report.executed, report.passed, report.failed
        );
    }

    let totals = Totals::of(reports);
    println!(
        "\nOverall: on-disk {}, executed {}, passed {}, failed {}.",
        totals.on_disk, totals.executed, totals.passed, totals.failed
    );
}

fn run_feature(paths: &Paths, feature: &str, negative: bool) -> Report {
    let directory = Path::new("tests").join(feature);
    let mut tests = match corpus::cpp_files(&directory) {
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
                failed_names: Vec::new(),
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
        failed_names: Vec::new(),
    };

    for cpp in tests {
        let name = cpp
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_string();
        report.executed += 1;
        let result = if negative {
            run::check_negative(paths, feature, &name, &cpp)
        } else {
            run::check_positive(paths, feature, &name, &cpp)
        };
        match result {
            Ok(()) => {
                report.passed += 1;
                report.passed_names.push(name);
            }
            Err(message) => {
                report.failed += 1;
                report.failures.push(format!("{name} {message}"));
                report.failed_names.push(name);
            }
        }
    }
    report
}

fn listed_tests() -> BTreeSet<(String, String)> {
    let Ok(manifest) = fs::read_to_string("tests/PASSING.txt") else {
        return BTreeSet::new();
    };
    manifest
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let (feature, name) = line.split_once('/')?;
            Some((feature.to_string(), name.to_string()))
        })
        .collect()
}

fn check_manifest(reports: &[Report], reference_mode: bool) -> usize {
    let manifest = match fs::read_to_string("tests/PASSING.txt") {
        Ok(manifest) => manifest,
        Err(error) => {
            eprintln!("cannot read tests/PASSING.txt: {error}");
            return 1;
        }
    };
    let ran_features: BTreeSet<&str> = reports
        .iter()
        .map(|report| report.feature.as_str())
        .collect();
    let passed: BTreeSet<(&str, &str)> = reports
        .iter()
        .flat_map(|report| {
            report
                .passed_names
                .iter()
                .map(|name| (report.feature.as_str(), name.as_str()))
        })
        .collect();

    let mut failures = 0;
    let mut listed: BTreeSet<(&str, &str)> = BTreeSet::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((feature, name)) = line.split_once('/') else {
            eprintln!("malformed PASSING entry: {line}");
            failures += 1;
            continue;
        };
        listed.insert((feature, name));
        if !ran_features.contains(feature) {
            continue;
        }
        if reference_mode && feature == "negative" {
            continue;
        }
        if !passed.contains(&(feature, name)) {
            eprintln!("PASSING entry did not pass: {line}");
            failures += 1;
        }
    }

    // A test that passes but is absent from the manifest is invisible to the
    // baseline. Only inspect groups that this invocation actually ran.
    for (feature, name) in &passed {
        if !listed.contains(&(*feature, *name)) {
            eprintln!("passing test missing from tests/PASSING.txt: {feature}/{name}");
            failures += 1;
        }
    }
    failures
}
