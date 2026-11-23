pub use cppbolt::compile_with_provider;
use cppbolt::{IncludeProvider, ResolvedInclude};
use std::fs;
use std::path::Path;
pub use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
pub struct RepoIncludes {
    pub stdlib: PathBuf,
}

impl IncludeProvider for RepoIncludes {
    fn read_include(&self, name: &str) -> Option<String> {
        fs::read_to_string(name).ok()
    }

    fn resolve_include(
        &self,
        name: &str,
        including_file: Option<&str>,
        quoted: bool,
    ) -> Option<ResolvedInclude> {
        let mut candidates = Vec::new();
        if quoted
            && let Some(including) = including_file
            && let Some(parent) = Path::new(including).parent()
            && !parent.as_os_str().is_empty()
        {
            candidates.push(parent.join(name));
        }
        candidates.push(self.stdlib.join(name));
        candidates.push(PathBuf::from(name));
        for path in candidates {
            if let Ok(content) = fs::read_to_string(&path) {
                return Some(ResolvedInclude {
                    name: path.to_string_lossy().into_owned(),
                    content,
                });
            }
        }
        None
    }
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn run_cpp(source: &str) -> Result<(i32, String), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    let wasm = compile_with_provider(source, "<input>", 256, &includes)
        .map_err(|e| format!("Compilation error: {e}"))?;

    let id = std::process::id();
    let c = COUNTER.fetch_add(1, Ordering::Relaxed);
    let wasm_path = std::env::temp_dir().join(format!("exn_test_{id}_{c}.wasm"));
    fs::write(&wasm_path, &wasm).map_err(|e| e.to_string())?;

    let host_js = root.join("web").join("host.js");
    let script = format!(
        r#"
const fs = require("fs");
const {{ createWasmHost }} = require({:?});
const bytes = fs.readFileSync({:?});
const host = createWasmHost({{
    stdin: "",
    onExit: (code) => {{
        process.stdout.write(host.getStdout());
        process.exit(code);
    }},
}});
WebAssembly.instantiate(bytes, host.imports).then(({{ instance }}) => {{
    let ret = 0;
    try {{
        ret = instance.exports.main ? instance.exports.main() : instance.exports._start();
    }} catch (e) {{
        process.stdout.write(host.getStdout());
        console.error("WASM_TRAP: " + (e.stack || e));
        process.exit(134);
    }}
    process.stdout.write(host.getStdout());
    process.exit(ret);
}}).catch((e) => {{
    console.error("WASM_TRAP: " + e.message);
    process.exit(134);
}});
"#,
        host_js.to_string_lossy(),
        wasm_path.to_string_lossy()
    );

    let node = std::env::var("NODE").unwrap_or_else(|_| "node".to_string());
    let output = Command::new(&node)
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to spawn {node}: {e}"))?;

    let _ = fs::remove_file(&wasm_path);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let code = output.status.code().unwrap_or(-1);
    if code != 0 && !output.stderr.is_empty() {
        eprintln!("NODE_STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok((code, stdout))
}

pub fn assert_compile_error(source: &str, needle: &str) -> Result<(), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    match compile_with_provider(source, "<input>", 256, &includes) {
        Err(err) => {
            let err_str = err.to_string();
            if err_str.contains(needle) {
                Ok(())
            } else {
                Err(format!("unexpected error: {err_str}"))
            }
        }
        Ok(_) => Err(format!("must fail compilation with '{needle}'")),
    }
}
