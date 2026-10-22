//! The browser flow: the WebAssembly build of the compiler compiles a
//! program, and the JavaScript host shim runs the result.
//!
//! The playground of `web/playground.html` uses exactly this sequence, so
//! the test keeps the flow of a browser host working.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The script that compiles a source and runs the result.
const BROWSER_FLOW: &str = r#"
const fs = require("fs");
const path = require("path");

async function main() {
    const root = process.argv[2];
    const compilerPath = process.argv[3];
    const sourcePath = process.argv[4];
    const source = fs.readFileSync(sourcePath, "utf8");

    const compiler = await WebAssembly.instantiate(fs.readFileSync(compilerPath), {});
    const api = compiler.instance.exports;

    const bytes = Buffer.from(source, "utf8");
    const input = api.alloc_buffer(bytes.length);
    new Uint8Array(api.memory.buffer).set(bytes, input);
    const result = api.compile_wasm(input, bytes.length);
    api.dealloc_buffer(input, bytes.length);
    if (!result) {
        const length = api.get_error_len();
        const pointer = api.get_error_ptr();
        console.error(Buffer.from(
            new Uint8Array(api.memory.buffer).slice(pointer, pointer + length)
        ).toString("utf8"));
        process.exit(2);
    }
    const length = api.get_result_len();
    const module = Buffer.from(new Uint8Array(api.memory.buffer).slice(result, result + length));
    api.free_result(result, length);

    const { createWasmImports } = require(path.join(root, "web/host.js"));
    const host = createWasmImports(null, "", (code) => {
        process.stdout.write(host.getStdout());
        process.exit(code);
    });
    const program = await WebAssembly.instantiate(module, host.imports);
    let code = 0;
    try {
        if (program.instance.exports.main) {
            code = program.instance.exports.main();
        } else if (program.instance.exports._start) {
            code = program.instance.exports._start();
        }
    } catch (error) {
        console.error("WASM_TRAP: " + error.message);
        process.exit(134);
    }
    process.stdout.write(host.getStdout());
    if (code) {
        process.exit(code);
    }
}

main().catch((error) => {
    console.error(error);
    process.exit(1);
});
"#;

/// The script that loads the `pkg` directory of the web frontend and runs
/// the program it compiles.
const PACKAGE_FLOW: &str = r#"
const fs = require("fs");
const path = require("path");

async function main() {
    const root = process.argv[2];
    const wasmPath = process.argv[3];

    const { init } = require(path.join(root, "web/pkg/cppwasm.js"));
    const bytes = fs.readFileSync(wasmPath);
    const compiler = await init(
        bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength)
    );
    const result = compiler.compile(
        '#include <cstdio>\nint main() { std::printf("from the package\\n"); return 0; }\n'
    );
    if (!result.ok) {
        console.error(result.diagnostic);
        process.exit(2);
    }
    const { createWasmImports } = require(path.join(root, "web/host.js"));
    const host = createWasmImports(null, "", (code) => {
        process.stdout.write(host.getStdout());
        process.exit(code);
    });
    const program = await WebAssembly.instantiate(result.bytes, host.imports);
    const code = program.instance.exports.main ? program.instance.exports.main() : 0;
    process.stdout.write(host.getStdout());
    if (code) {
        process.exit(code);
    }
}

main().catch((error) => {
    console.error(error);
    process.exit(1);
});
"#;

/// The program the flow compiles and runs.
const PROGRAM: &str = r#"#include <cstdio>
#include <string>

struct Point {
    int x;
    int y;
    Point(int a, int b) : x(a), y(b) {
        std::printf("Point(%d, %d)\n", x, y);
    }
};

int main() {
    Point p(3, 4);
    std::string text = "hello";
    std::printf("%s %d\n", text.c_str(), p.x + p.y);
    return 0;
}
"#;

/// Builds the WebAssembly build of the compiler and returns its path.
fn wasm_compiler(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let isolated = root.join("target/abi-wasm");
    let wasm = isolated.join("wasm32-unknown-unknown/release/cppwasm.wasm");
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
        return Err("cannot build the WebAssembly compiler".into());
    }
    if !wasm.exists() {
        return Err(format!("the WebAssembly compiler is missing at {}", wasm.display()).into());
    }
    Ok(wasm)
}

#[test]
fn a_browser_host_compiles_and_runs_a_program() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wasm = wasm_compiler(&root)?;
    let node = std::env::var("NODE").unwrap_or_else(|_| "node".to_string());
    let work = root.join("target/browser-flow");
    std::fs::create_dir_all(&work)?;
    let script = work.join("flow.js");
    let source = work.join("program.cpp");
    std::fs::write(&script, BROWSER_FLOW)?;
    std::fs::write(&source, PROGRAM)?;

    let output = Command::new(node)
        .arg(&script)
        .arg(&root)
        .arg(&wasm)
        .arg(&source)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "the browser flow failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(stdout, "Point(3, 4)\nhello 7\n");
    Ok(())
}

#[test]
fn the_package_of_the_web_frontend_compiles_a_program() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wasm = wasm_compiler(&root)?;
    let node = std::env::var("NODE").unwrap_or_else(|_| "node".to_string());
    let work = root.join("target/browser-flow");
    std::fs::create_dir_all(&work)?;
    let script = work.join("package.js");
    std::fs::write(&script, PACKAGE_FLOW)?;

    let output = Command::new(node)
        .arg(&script)
        .arg(&root)
        .arg(&wasm)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "the package flow failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(stdout, "from the package\n");
    Ok(())
}
