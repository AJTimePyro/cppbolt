//! Real-compiler oracle gate: replays every positive test through `g++` and
//! `clang++` and checks that their native output matches the recorded
//! `.expected` file. Compilers that are not installed are skipped.

use cppbolt::oracle;
use std::process::ExitCode;

pub(super) fn run_oracle(filter: Option<&str>) -> ExitCode {
    let groups = super::selected_groups(filter);

    let report = oracle::run_oracle(&groups);
    if report.compilers.is_empty() {
        eprintln!("oracle gate skipped: neither g++ nor clang++ is available");
        return ExitCode::SUCCESS;
    }
    for failure in &report.failures {
        eprintln!("[FAIL] {failure}");
    }
    println!(
        "\nOracle: {} matched, {} mismatched, {} skipped across {} compiler(s).",
        report.matched,
        report.mismatched,
        report.skipped,
        report.compilers.len()
    );
    if report.mismatched == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
