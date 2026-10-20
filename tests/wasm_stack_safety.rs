//! Node-hosted stack-safety check for the wasm build of the compiler.
//!
//! The test builds the wasm compiler into an isolated target directory and
//! runs `tests/wasm_stack_safety.js`, which compiles wide and deeply nested
//! inputs through the wasm ABI and fails on a trap, a wrong outcome, or a
//! compile slower than ten seconds.

use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn wasm_hosted_compiler_stays_within_stack_limits() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wasm = ensure_wasm_compiler(&root)?;
    let node = std::env::var("NODE").unwrap_or_else(|_| "node".to_string());
    let output = Command::new(node)
        .arg(root.join("tests/wasm_stack_safety.js"))
        .arg(&wasm)
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

fn ensure_wasm_compiler(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let isolated = root.join("target/abi-wasm");
    let wasm = isolated.join("wasm32-unknown-unknown/fast/cppwasm.wasm");
    if wasm.exists() {
        return Ok(wasm);
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .args([
            "build",
            "--profile",
            "fast",
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
