//! The native oracle gate as a cargo test.
//!
//! The recorded `.expected` files are the reference of the corpus.  The
//! corpus runner replays them through the real compilers as well, but a
//! wrong expectation must not be able to hide there, so the gate also runs
//! as a test.  A machine without a native compiler cannot run the gate, and
//! the test says so instead of passing silently.

use cppbolt::corpus::FEATURES;
use cppbolt::oracle;

#[test]
fn every_expected_file_matches_the_native_compilers() -> Result<(), Box<dyn std::error::Error>> {
    let report = oracle::run_oracle(FEATURES);
    if report.compilers.is_empty() {
        let message = "oracle skipped: neither g++ nor clang++ is available";
        println!("{message}");
        return Err(message.into());
    }
    println!(
        "oracle: {} matched, {} mismatched, {} skipped across {} compiler(s)",
        report.matched,
        report.mismatched,
        report.skipped,
        report.compilers.len()
    );
    if !report.failures.is_empty() {
        return Err(format!(
            "the native compilers disagree with the recorded expectations:\n{}",
            report.failures.join("\n")
        )
        .into());
    }
    Ok(())
}
