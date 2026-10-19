//! Node-hosted check that the wasm C ABI releases every buffer it hands out.
//!
//! The test builds the wasm compiler into an isolated target directory so it
//! cannot deadlock on the outer `cargo test` build lock, then runs the Node
//! lifecycle script against the freshly built module.

use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn abi_lifecycle_keeps_memory_bounded() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wasm = ensure_wasm_compiler(&root)?;
    let node = std::env::var("NODE").unwrap_or_else(|_| "node".to_string());
    let output = Command::new(node)
        .arg(root.join("tests/ffi_lifecycle.js"))
        .arg(&wasm)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("ffi lifecycle failed\nstdout:\n{stdout}\nstderr:\n{stderr}").into());
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
