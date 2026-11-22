//! Integration tests verifying C++ exception unwinding, scope lifetimes, and stack integrity.

use cppbolt::{IncludeProvider, ResolvedInclude, compile_with_provider};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
struct RepoIncludes {
    stdlib: PathBuf,
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

fn run_cpp(source: &str) -> Result<(i32, String), String> {
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
        console.error("WASM_TRAP: " + e.message);
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
    Ok((code, stdout))
}

#[test]
fn test_primitive_exception_cross_function_boundary() -> Result<(), String> {
    let source = r#"
void inner() {
    throw 42;
}

int main() {
    try {
        inner();
    } catch (int value) {
        return value;
    }

    return 0;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 42, "inner() throwing 42 caught by main() must return exit code 42");
    Ok(())
}

#[test]
fn test_multiple_objects_reverse_destruction_order() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

void fail() {
    Tracer a(1);
    Tracer b(2);
    Tracer c(3);
    throw 99;
}

int main() {
    try {
        fail();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 C:3 D:3 D:2 D:1 Caught:99");
    Ok(())
}

#[test]
fn test_nested_scopes_with_exceptions_at_different_depths() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

void run() {
    Tracer t1(1);
    {
        Tracer t2(2);
        {
            Tracer t3(3);
            throw 42;
        }
    }
}

int main() {
    try {
        run();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 C:3 D:3 D:2 D:1 Caught:42");
    Ok(())
}

#[test]
fn test_catch_preserves_outer_local_objects() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int main() {
    Tracer outer(1);
    try {
        Tracer inner(2);
        throw 55;
    } catch (int e) {
        std::printf("Caught:%d ", e);
    }
    std::printf("After ");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 D:2 Caught:55 After D:1");
    Ok(())
}

#[test]
fn test_shadow_stack_integrity_after_catching_exception() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    int payload[8];
    Tracer(int i) : id(i) {
        for (int k = 0; k < 8; ++k) payload[k] = 100 + k;
    }
    ~Tracer() {
        std::printf("D:%d:%d ", id, payload[0]);
    }
};

void clobber() {
    int buf[16];
    for (int i = 0; i < 16; ++i) buf[i] = 9999;
}

int main() {
    Tracer t(1);
    try {
        throw 42;
    } catch (int e) {
        std::printf("Caught:%d ", e);
    }
    clobber();
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Caught:42 D:1:100");
    Ok(())
}

#[test]
fn test_nested_try_blocks_and_handler_precedence() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int main() {
    try {
        Tracer t1(1);
        try {
            Tracer t2(2);
            throw 42;
        } catch (int e) {
            std::printf("Inner:%d ", e);
        }
        std::printf("Between ");
    } catch (...) {
        std::printf("Outer ");
    }
    std::printf("End");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 D:2 Inner:42 Between D:1 End");
    Ok(())
}

#[test]
fn test_conditional_object_construction() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

void choose(bool flag) {
    Tracer a(1);
    if (flag) {
        Tracer b(2);
        throw 10;
    } else {
        Tracer c(3);
        throw 20;
    }
}

int main() {
    try { choose(true); } catch (int e) { std::printf("E:%d ", e); }
    try { choose(false); } catch (int e) { std::printf("E:%d ", e); }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 D:2 D:1 E:10 C:1 C:3 D:3 D:1 E:20");
    Ok(())
}

#[test]
fn test_objects_constructed_inside_loops() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

void loop_run() {
    Tracer outer(100);
    for (int i = 0; i < 3; ++i) {
        Tracer t(i);
        if (i == 2) throw 77;
    }
}

int main() {
    try {
        loop_run();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:100 C:0 D:0 C:1 D:1 C:2 D:2 D:100 Caught:77");
    Ok(())
}

#[test]
fn test_nested_calls_in_argument_expressions() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int thrower() {
    throw 42;
    return 10;
}

void sink(int x) {
    std::printf("sink:%d ", x);
}

void middle() {
    Tracer t(55);
    sink(thrower());
}

int main() {
    try {
        middle();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:55 D:55 Caught:42");
    Ok(())
}

#[test]
fn test_member_virtual_and_function_pointer_calls_unwind() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

struct Worker {
    void work() { throw 11; }
};

struct Base {
    virtual void vwork() = 0;
};
struct Derived : Base {
    virtual void vwork() override { throw 22; }
};

void fn_throw() { throw 33; }

void test_member() { Tracer t(1); Worker w; w.work(); }
void test_virtual() { Tracer t(2); Derived d; Base* b = &d; b->vwork(); }
void test_indirect() { Tracer t(3); void (*f)() = fn_throw; f(); }

int main() {
    try { test_member(); } catch (int e) { std::printf("E:%d ", e); }
    try { test_virtual(); } catch (int e) { std::printf("E:%d ", e); }
    try { test_indirect(); } catch (int e) { std::printf("E:%d ", e); }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 D:1 E:11 C:2 D:2 E:22 C:3 D:3 E:33");
    Ok(())
}

#[test]
fn test_constructor_failure_unwinds_completed_subobjects() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

struct Complex {
    Tracer t1;
    Tracer t2;
    Complex() : t1(1), t2(2) {
        std::printf("CtorFail ");
        throw 99;
    }
    ~Complex() {
        std::printf("DtorComplex ");
    }
};

int main() {
    try {
        Complex c;
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 CtorFail D:2 D:1 Caught:99");
    Ok(())
}

#[test]
fn test_destructor_throwing_during_unwinding_terminates() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct BadTracer {
    int id;
    BadTracer(int i) : id(i) {}
    ~BadTracer() {
        std::printf("BadDtor ");
        throw 999;
    }
};

void inner() {
    throw 42;
}

void middle() {
    BadTracer b(1);
    inner();
}

int main() {
    try {
        middle();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 134, "program must trap with abort/134 on throwing destructor during unwinding");
    assert!(stdout.contains("BadDtor"), "BadDtor must have been entered");
    assert!(!stdout.contains("Caught"), "handler must NOT catch or replace active exception");
    Ok(())
}

#[test]
fn test_repeated_exceptions_and_continued_execution() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

void step(int x) {
    Tracer t(x * 10);
    throw x;
}

int main() {
    Tracer outer(100);
    try {
        step(1);
    } catch (int e1) {
        std::printf("Caught:%d ", e1);
    }

    try {
        step(2);
    } catch (int e2) {
        std::printf("Caught:%d ", e2);
    }

    std::printf("End ");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:100 C:10 D:10 Caught:1 C:20 D:20 Caught:2 End D:100");
    Ok(())
}

#[test]
fn test_rethrow_outside_catch_terminates() -> Result<(), String> {
    let source = r#"
#include <cstdio>

void foo() {
    throw;
}

int main() {
    try {
        foo();
    } catch (...) {
        std::printf("Caught");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 134, "throw; outside catch must terminate with abort/unreachable");
    assert!(!stdout.contains("Caught"));
    Ok(())
}

#[test]
fn test_exceeding_max_cleanup_objects_errors() {
    let mut source = String::from("struct S { ~S() {} }; int main() {\n");
    for i in 0..65 {
        source.push_str(&format!("    S s{};\n", i));
    }
    source.push_str("    return 0;\n}\n");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    let res = compile_with_provider(&source, "<input>", 256, &includes);
    assert!(res.is_err(), "must fail compilation when exceeding 64 cleanup objects");
    let err = res.err().unwrap().to_string();
    assert!(err.contains("maximum of 64 cleanup objects"));
}
