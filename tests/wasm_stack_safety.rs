//! Node-hosted stack-safety check for the wasm build of the compiler.
//!
//! The test builds the wasm compiler into an isolated target directory and
//! runs `tests/wasm_stack_safety.js`, which compiles wide and deeply nested
//! inputs through the wasm ABI and fails on a trap, a wrong outcome, or a
//! compile slower than ten seconds.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The nesting limits of the compiler, pinned as literals.
///
/// The same table is in `docs/known-limitations.md` and in `tests/nesting.rs`.
/// The limits are read from the compiler only to check that the two agree, so
/// a changed limit fails this test instead of silently moving the boundary.
const PARENTHESIS_LIMIT: usize = 1600;
const CALL_LIMIT: usize = 1500;
const STATEMENT_LIMIT: usize = 1600;

#[test]
fn the_pinned_limits_match_the_compiler() {
    let limits = cppbolt::nesting_limits();
    assert_eq!(limits.parentheses, PARENTHESIS_LIMIT);
    assert_eq!(limits.calls, CALL_LIMIT);
    assert_eq!(limits.statements, STATEMENT_LIMIT);
}

#[test]
fn wasm_hosted_compiler_stays_within_stack_limits() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wasm = ensure_wasm_compiler(&root)?;
    let node = std::env::var("NODE").unwrap_or_else(|_| "node".to_string());
    let limits = root.join("target/wasm-stack-limits.json");
    let limits = write_limits(&limits)?;
    let output = Command::new(node)
        .arg(root.join("tests/wasm_stack_safety.js"))
        .arg(&wasm)
        .arg(&limits)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(
            format!("wasm stack safety failed\nstdout:\n{stdout}\nstderr:\n{stderr}").into(),
        );
    }
    Ok(())
}

/// Writes the per-construct nesting limits as JSON for the JavaScript check.
fn write_limits(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let limits = cppbolt::nesting_limits();
    let json = format!(
        "{{\"parentheses\": {}, \"calls\": {}, \"statements\": {}}}",
        limits.parentheses, limits.calls, limits.statements
    );
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, json)?;
    Ok(path.to_path_buf())
}
fn ensure_wasm_compiler(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let isolated = root.join("target/abi-wasm");
    // The release build is the one hosts use, and the one the nesting
    // limits are measured on.
    let wasm = isolated.join("wasm32-unknown-unknown/release/cppbolt.wasm");
    if wasm.exists() {
        return Ok(wasm);
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
            "--lib",
        ])
        .current_dir(root)
        .env("CARGO_TARGET_DIR", &isolated)
        .status()?;
    if !status.success() {
        return Err(format!("cannot build wasm compiler: {status}").into());
    }
    if !wasm.exists() {
        return Err(format!("wasm compiler missing at {}", wasm.display()).into());
    }
    Ok(wasm)
}
