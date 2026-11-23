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
fn test_exceeding_max_cleanup_objects_errors() -> Result<(), String> {
    let mut source = String::from("struct S { ~S() {} }; int main() {\n");
    for i in 0..65 {
        source.push_str(&format!("    S s{};\n", i));
    }
    source.push_str("    return 0;\n}\n");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    match compile_with_provider(&source, "<input>", 256, &includes) {
        Err(err) => {
            let err_str = err.to_string();
            if err_str.contains("maximum of 64 cleanup objects") {
                Ok(())
            } else {
                Err(format!("unexpected error: {err_str}"))
            }
        }
        Ok(_) => Err("must fail compilation when exceeding 64 cleanup objects".to_string()),
    }
}

#[test]
fn test_destructor_throwing_during_normal_exit_propagates() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Normal {
    int id;
    Normal(int i) : id(i) { std::printf("C_Normal:%d ", id); }
    ~Normal() { std::printf("D_Normal:%d ", id); }
};

struct ThrowingDtor {
    int val;
    ThrowingDtor(int v) : val(v) {}
    ~ThrowingDtor() noexcept(false) {
        std::printf("D_Throw:%d ", val);
        throw val;
    }
};

void test_fn() {
    Normal n(1);
    ThrowingDtor t(99);
}

int main() {
    try {
        test_fn();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C_Normal:1 D_Throw:99 D_Normal:1 Caught:99");
    Ok(())
}

#[test]
fn test_destructor_throwing_while_unwinding_from_normal_destructor_throw_terminates() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct ThrowingDtor {
    int val;
    ThrowingDtor(int v) : val(v) {}
    ~ThrowingDtor() noexcept(false) {
        std::printf("D_Throw:%d ", val);
        throw val;
    }
};

void test_fn() {
    ThrowingDtor first(1);
    ThrowingDtor second(2);
}

int main() {
    try {
        test_fn();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 134, "destructor throwing during unwinding from normal exit throw must terminate with abort/134");
    assert!(stdout.contains("D_Throw:2"), "second dtor must have thrown");
    assert!(stdout.contains("D_Throw:1"), "first dtor must have been entered during unwinding and thrown");
    assert!(!stdout.contains("Caught"), "active exception must terminate, not be caught");
    Ok(())
}

#[test]
fn test_partially_constructed_array_unwinds_completed_elements() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int g_counter = 0;
struct Item {
    int id;
    Item() {
        id = ++g_counter;
        std::printf("C:%d ", id);
        if (id == 3) {
            throw 99;
        }
    }
    ~Item() {
        std::printf("D:%d ", id);
    }
};

void test_arr() {
    Item arr[4];
}

int main() {
    try {
        test_arr();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 C:3 D:2 D:1 Caught:99");
    Ok(())
}

#[test]
fn test_member_array_unwinds_on_constructor_failure() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int g_m_counter = 0;
struct MemberItem {
    int id;
    MemberItem() {
        id = ++g_m_counter;
        std::printf("C_M:%d ", id);
    }
    ~MemberItem() {
        std::printf("D_M:%d ", id);
    }
};

struct ThrowingMember {
    ThrowingMember() {
        std::printf("C_Throw ");
        throw 77;
    }
    ~ThrowingMember() {}
};

struct Container {
    MemberItem arr[3];
    ThrowingMember bad;
    Container() {}
};

int main() {
    try {
        Container c;
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C_M:1 C_M:2 C_M:3 C_Throw D_M:3 D_M:2 D_M:1 Caught:77");
    Ok(())
}

#[test]
fn test_nested_catch_preserves_outer_rethrow() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    try {
        try {
            throw 10;
        } catch (int e1) {
            std::printf("OuterCatch:%d ", e1);
            try {
                throw 20;
            } catch (int e2) {
                std::printf("InnerCatch:%d ", e2);
            }
            throw;
        }
    } catch (int rethrown) {
        std::printf("CaughtRethrown:%d", rethrown);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "OuterCatch:10 InnerCatch:20 CaughtRethrown:10");
    Ok(())
}

#[test]
fn test_nested_catch_all_preserves_outer_rethrow() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    try {
        try {
            throw 100;
        } catch (...) {
            std::printf("OuterCatchAll ");
            try {
                throw 200;
            } catch (...) {
                std::printf("InnerCatchAll ");
            }
            throw;
        }
    } catch (int e) {
        std::printf("CaughtFinal:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "OuterCatchAll InnerCatchAll CaughtFinal:100");
    Ok(())
}

#[test]
fn test_break_and_continue_unwinds_exact_scopes() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int main() {
    for (int i = 0; i < 3; i++) {
        Tracer t(i);
        if (i == 0) {
            continue;
        }
        if (i == 1) {
            break;
        }
    }
    std::printf("AfterLoop");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:0 D:0 C:1 D:1 AfterLoop");
    Ok(())
}

#[test]
fn test_exception_caught_inside_loop_preserves_loop_variables() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int main() {
    for (int i = 0; i < 2; i++) {
        Tracer outer(i + 10);
        try {
            Tracer inner(i + 20);
            throw i;
        } catch (int e) {
            std::printf("Caught:%d ", e);
        }
        std::printf("LoopEnd:%d ", i);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:10 C:20 D:20 Caught:0 LoopEnd:0 D:10 C:11 C:21 D:21 Caught:1 LoopEnd:1 D:11");
    Ok(())
}

#[test]
fn test_nested_catch_depth_5_levels_and_rethrow() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    try {
        try {
            throw 10;
        } catch (...) {
            std::printf("L1 ");
            try {
                throw 20;
            } catch (...) {
                std::printf("L2 ");
                try {
                    throw 30;
                } catch (...) {
                    std::printf("L3 ");
                    try {
                        throw 40;
                    } catch (...) {
                        std::printf("L4 ");
                        try {
                            throw 50;
                        } catch (...) {
                            std::printf("L5 ");
                        }
                        std::printf("A4 ");
                    }
                    std::printf("A3 ");
                }
                std::printf("A2 ");
                throw; // Rethrows 20!
            }
        }
    } catch (int e) {
        std::printf("CaughtFinal:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "L1 L2 L3 L4 L5 A4 A3 A2 CaughtFinal:20");
    Ok(())
}

#[test]
fn test_mixed_catch_int_and_catch_all_nesting_rethrow() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    try {
        try {
            throw 111;
        } catch (int e1) {
            std::printf("L1:%d ", e1);
            try {
                throw 222;
            } catch (...) {
                std::printf("L2 ");
                try {
                    throw 333;
                } catch (int e3) {
                    std::printf("L3:%d ", e3);
                    try {
                        throw 444;
                    } catch (...) {
                        std::printf("L4 ");
                        try {
                            throw 555;
                        } catch (int e5) {
                            std::printf("L5:%d ", e5);
                        }
                        std::printf("A4 ");
                    }
                    std::printf("A3 ");
                }
                std::printf("A2 ");
                throw; // Rethrows 222 (from catch-all L2)
            }
        }
    } catch (int final_e) {
        std::printf("CaughtFinal:%d", final_e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "L1:111 L2 L3:333 L4 L5:555 A4 A3 A2 CaughtFinal:222");
    Ok(())
}

#[test]
fn test_temporary_cleanup_when_later_argument_throws() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Temp {
    int id;
    Temp(int i) : id(i) { std::printf("C_Temp:%d ", id); }
    ~Temp() { std::printf("D_Temp:%d ", id); }
};

int bar() {
    std::printf("BarThrow ");
    throw 42;
}

void foo(Temp t, int b) {
    std::printf("Foo ");
}

int main() {
    try {
        foo(Temp(1), bar());
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C_Temp:1 BarThrow D_Temp:1 Caught:42");
    Ok(())
}

#[test]
fn test_temporary_cleanup_when_called_function_throws() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Temp {
    int id;
    Temp(int i) : id(i) { std::printf("C_Temp:%d ", id); }
    ~Temp() { std::printf("D_Temp:%d ", id); }
};

void foo(const Temp& t) {
    std::printf("FooThrow ");
    throw 77;
}

int main() {
    try {
        foo(Temp(2));
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C_Temp:2 FooThrow D_Temp:2 Caught:77");
    Ok(())
}

#[test]
fn test_multiple_temporary_arguments_reverse_destruction() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Temp {
    int id;
    Temp(int i) : id(i) { std::printf("C:%d ", id); }
    ~Temp() { std::printf("D:%d ", id); }
};

int bar() {
    std::printf("BarThrow ");
    throw 99;
}

void foo(Temp a, Temp b, Temp c, int d) {}

int main() {
    try {
        foo(Temp(10), Temp(20), Temp(30), bar());
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:10 C:20 C:30 BarThrow D:30 D:20 D:10 Caught:99");
    Ok(())
}

#[test]
fn test_nested_expression_temporaries_cleanup() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Temp {
    int id;
    Temp(int i) : id(i) { std::printf("C_Temp:%d ", id); }
    ~Temp() { std::printf("D_Temp:%d ", id); }
};

Temp make_temp(int id) {
    std::printf("Make:%d ", id);
    return Temp(id);
}

int throw_nested() {
    std::printf("ThrowNested ");
    throw 88;
}

void consume(Temp a, Temp b) {}

int main() {
    try {
        consume(make_temp(100), make_temp(throw_nested()));
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Make:100 C_Temp:100 ThrowNested D_Temp:100 Caught:88");
    Ok(())
}

#[test]
fn test_reference_binding_temporary_lifetime_extension() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Temp {
    int id;
    Temp(int i) : id(i) { std::printf("C_Temp:%d ", id); }
    ~Temp() { std::printf("D_Temp:%d ", id); }
};

void test_ref_normal() {
    std::printf("Start ");
    const Temp& r = Temp(50);
    std::printf("RefBound ");
    std::printf("EndFn ");
}

void test_ref_throw() {
    const Temp& r = Temp(60);
    std::printf("RefActive ");
    throw 99;
}

int main() {
    test_ref_normal();
    std::printf("AfterNormal ");
    try {
        test_ref_throw();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Start C_Temp:50 RefBound EndFn D_Temp:50 AfterNormal C_Temp:60 RefActive D_Temp:60 Caught:99");
    Ok(())
}

#[test]
fn test_constructor_failure_cleans_completed_arguments_and_subobjects() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Temp {
    int id;
    Temp(int i) : id(i) { std::printf("C_Temp:%d ", id); }
    ~Temp() { std::printf("D_Temp:%d ", id); }
};

struct Sub {
    int id;
    Sub(int i) : id(i) { std::printf("C_Sub:%d ", id); }
    ~Sub() { std::printf("D_Sub:%d ", id); }
};

struct FailingClass {
    Sub s;
    FailingClass(Temp t) : s(1) {
        std::printf("CtorThrow ");
        throw 123;
    }
    ~FailingClass() {
        std::printf("D_Failing ");
    }
};

void run_test() {
    FailingClass obj(Temp(2));
}

int main() {
    try {
        run_test();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C_Temp:2 C_Sub:1 CtorThrow D_Sub:1 D_Temp:2 Caught:123");
    Ok(())
}

#[test]
fn test_byval_argument_preserves_subexpression_temporaries() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct SubTemp {
    int id;
    SubTemp(int i) : id(i) { std::printf("C_Sub:%d ", id); }
    ~SubTemp() { std::printf("D_Sub:%d ", id); }
};

struct ArgWidget {
    int id;
    ArgWidget(int i) : id(i) { std::printf("C_Wid:%d ", id); }
    ~ArgWidget() { std::printf("D_Wid:%d ", id); }
};

ArgWidget make_widget(const SubTemp& s) {
    std::printf("MakeWid ");
    return ArgWidget(s.id);
}

void consume_normal(ArgWidget w) {
    std::printf("ConsumeNormal ");
}

void consume_throw(ArgWidget w) {
    std::printf("ConsumeThrow ");
    throw 456;
}

int main() {
    std::printf("---Normal--- ");
    consume_normal(make_widget(SubTemp(10)));
    std::printf("---AfterNormal--- ");
    try {
        consume_throw(make_widget(SubTemp(20)));
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "---Normal--- C_Sub:10 MakeWid C_Wid:10 ConsumeNormal D_Wid:10 D_Sub:10 ---AfterNormal--- C_Sub:20 MakeWid C_Wid:20 ConsumeThrow D_Wid:20 D_Sub:20 Caught:456"
    );
    Ok(())
}

#[test]
fn test_byval_parameter_construction_failure_cleans_earlier_parameters() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct ParamA {
    int id;
    ParamA(int i) : id(i) { std::printf("C_A:%d ", id); }
    ~ParamA() { std::printf("D_A:%d ", id); }
};

struct ParamB {
    int id;
    ParamB(int i) : id(i) {
        std::printf("C_B:%d ", id);
        if (id == 99) {
            std::printf("Throw_B ");
            throw 777;
        }
    }
    ~ParamB() { std::printf("D_B:%d ", id); }
};

void target(ParamA a, ParamB b) {
    std::printf("InTarget ");
}

int main() {
    try {
        target(ParamA(1), ParamB(99));
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C_A:1 C_B:99 Throw_B D_A:1 Caught:777");
    Ok(())
}

#[test]
fn test_byval_parameter_callee_throw_cleans_parameters() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Param {
    int id;
    Param(int i) : id(i) { std::printf("C:%d ", id); }
    Param(const Param& o) : id(o.id) { std::printf("Copy:%d ", id); }
    ~Param() { std::printf("D:%d ", id); }
};

void target(Param a, Param b) {
    std::printf("InTarget ");
    throw 888;
}

int main() {
    Param p1(1);
    Param p2(2);
    try {
        target(p1, p2);
    } catch (int e) {
        std::printf("Caught:%d ", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 Copy:1 Copy:2 InTarget D:2 D:1 Caught:888 D:2 D:1");
    Ok(())
}

#[test]
fn test_ternary_conditional_temporaries_normal_exit() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Widget {
    int id;
    Widget(int i) : id(i) { std::printf("C_W:%d ", id); }
    ~Widget() { std::printf("D_W:%d ", id); }
};

int consume(const Widget& w) {
    return w.id;
}

int main() {
    int r1 = 1 ? consume(Widget(1)) : consume(Widget(2));
    std::printf("r1=%d ", r1);
    int r2 = 0 ? consume(Widget(3)) : consume(Widget(4));
    std::printf("r2=%d", r2);
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C_W:1 D_W:1 r1=1 C_W:4 D_W:4 r2=4");
    Ok(())
}

#[test]
fn test_short_circuit_temporaries_with_exceptions() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Widget {
    int id;
    Widget(int i) : id(i) {
        std::printf("C_W:%d ", id);
        if (id == 99) {
            std::printf("Throw_W ");
            throw 999;
        }
    }
    ~Widget() { std::printf("D_W:%d ", id); }
};

int consume(const Widget& w) {
    return w.id;
}

int main() {
    try {
        // false && ... should not evaluate second operand
        int x = (0 && consume(Widget(1))) ? 10 : (consume(Widget(2)) + consume(Widget(99)));
        std::printf("x=%d ", x);
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C_W:2 C_W:99 Throw_W D_W:2 Caught:999");
    Ok(())
}

#[test]
fn test_reference_binding_ternary_lifetime_extension() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Widget {
    int id;
    Widget(int i) : id(i) { std::printf("C_W:%d ", id); }
    ~Widget() { std::printf("D_W:%d ", id); }
};

int main() {
    std::printf("Start ");
    {
        // When cond is true, Widget(1) is constructed, Widget(2) is not.
        // Widget(1) must stay alive until end of scope.
        const Widget& w1 = 1 ? Widget(1) : Widget(2);
        std::printf("Scope1Active ");
    }
    std::printf("Scope1End ");
    {
        // When cond is false, Widget(4) is constructed, Widget(3) is not.
        // Widget(4) must stay alive until end of scope.
        const Widget& w2 = 0 ? Widget(3) : Widget(4);
        std::printf("Scope2Active ");
    }
    std::printf("Scope2End");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "Start C_W:1 Scope1Active D_W:1 Scope1End C_W:4 Scope2Active D_W:4 Scope2End"
    );
    Ok(())
}

#[test]
fn test_multiple_catch_clauses_int_then_catch_all() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    try {
        std::printf("ThrowingInt ");
        throw 42;
    } catch (int e) {
        std::printf("CaughtInt:%d ", e);
    } catch (...) {
        std::printf("CaughtAll ");
    }
    std::printf("Done");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "ThrowingInt CaughtInt:42 Done");
    Ok(())
}

#[test]
fn test_multiple_catch_clauses_rethrow_to_outer_catch() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    try {
        try {
            throw 99;
        } catch (int e) {
            std::printf("InnerInt:%d ", e);
            throw;
        } catch (...) {
            std::printf("InnerAll ");
        }
    } catch (int e2) {
        std::printf("OuterInt:%d", e2);
    } catch (...) {
        std::printf("OuterAll");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "InnerInt:99 OuterInt:99");
    Ok(())
}

#[test]
fn test_throw_and_catch_pointer_payload() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    try {
        const char* msg = "hello exception";
        throw msg;
    } catch (const char* e) {
        std::printf("Caught:%s", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Caught:hello exception");
    Ok(())
}

#[test]
fn test_throw_and_catch_double() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    try {
        throw 3.14159;
    } catch (double d) {
        std::printf("CaughtDouble:%d", (int)(d * 100.0));
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtDouble:314");
    Ok(())
}

#[test]
fn test_throw_void_fails_compilation() -> Result<(), String> {
    let source = "void foo() {} int main() { throw foo(); }";
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    match compile_with_provider(source, "<input>", 256, &includes) {
        Err(err) => {
            let err_str = err.to_string();
            if err_str.contains("cannot throw void expression") {
                Ok(())
            } else {
                Err(format!("unexpected error: {err_str}"))
            }
        }
        Ok(_) => Err("must fail compilation when throwing void".to_string()),
    }
}

#[test]
fn test_catch_all_not_last_handler_errors() -> Result<(), String> {
    let source = r#"
int main() {
    try {
        throw 1;
    } catch (...) {
    } catch (int e) {
    }
    return 0;
}
"#;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    match compile_with_provider(source, "<input>", 256, &includes) {
        Err(err) => {
            let err_str = err.to_string();
            if err_str.contains("catch-all handler (...) must be the last handler") {
                Ok(())
            } else {
                Err(format!("unexpected error: {err_str}"))
            }
        }
        Ok(_) => Err("must fail compilation when catch-all is not last".to_string()),
    }
}

#[test]
fn test_throw_and_catch_class_by_value() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct MyError {
    int code;
    MyError(int c) : code(c) {}
};

int main() {
    try {
        throw MyError(42);
    } catch (MyError err) {
        std::printf("CaughtByValue:%d", err.code);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtByValue:42");
    Ok(())
}

#[test]
fn test_throw_and_catch_class_by_const_reference() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct MyError {
    int code;
    MyError(int c) : code(c) {}
};

int main() {
    try {
        throw MyError(77);
    } catch (const MyError& err) {
        std::printf("CaughtByConstRef:%d", err.code);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtByConstRef:77");
    Ok(())
}

#[test]
fn test_throw_and_catch_class_by_non_const_reference_mutation() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct MyError {
    int code;
    MyError(int c) : code(c) {}
};

int main() {
    try {
        try {
            throw MyError(50);
        } catch (MyError& err) {
            std::printf("InnerRef:%d ", err.code);
            err.code += 25;
            throw;
        }
    } catch (const MyError& err2) {
        std::printf("OuterRefMutated:%d", err2.code);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "InnerRef:50 OuterRefMutated:75");
    Ok(())
}

#[test]
fn test_derived_exception_caught_by_base_reference_single_inheritance() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Base {
    int b;
    Base(int val) : b(val) {}
};

struct Derived : public Base {
    int d;
    Derived(int b_val, int d_val) : Base(b_val), d(d_val) {}
};

int main() {
    try {
        throw Derived(101, 202);
    } catch (Base& b) {
        std::printf("CaughtBaseRef:%d", b.b);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtBaseRef:101");
    Ok(())
}

#[test]
fn test_derived_exception_caught_by_secondary_base_multiple_inheritance() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Base1 {
    int b1;
    Base1(int x) : b1(x) {}
};

struct Base2 {
    int b2;
    Base2(int y) : b2(y) {}
};

struct Derived : public Base1, public Base2 {
    int d;
    Derived(int x, int y, int z) : Base1(x), Base2(y), d(z) {}
};

int main() {
    try {
        throw Derived(11, 22, 33);
    } catch (Base2& b2) {
        std::printf("CaughtBase2Ref:%d", b2.b2);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtBase2Ref:22");
    Ok(())
}

#[test]
fn test_exact_type_handler_preferred_before_base() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Base {
    int b;
    Base(int val) : b(val) {}
};

struct Derived : public Base {
    int d;
    Derived(int b_val, int d_val) : Base(b_val), d(d_val) {}
};

int main() {
    try {
        throw Derived(10, 20);
    } catch (Derived& d) {
        std::printf("CaughtExact:%d", d.d);
    } catch (Base& b) {
        std::printf("CaughtBase:%d", b.b);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtExact:20");
    Ok(())
}

#[test]
fn test_unmatched_exception_propagates_to_outer_try() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct ErrorA {
    int valA;
    ErrorA(int v) : valA(v) {}
};

struct ErrorB {
    int valB;
    ErrorB(int v) : valB(v) {}
};

int main() {
    try {
        try {
            throw ErrorA(999);
        } catch (ErrorB& b) {
            std::printf("ShouldNotMatchB ");
        }
    } catch (ErrorA& a) {
        std::printf("CaughtOuterA:%d", a.valA);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtOuterA:999");
    Ok(())
}

#[test]
fn test_exception_object_destruction_exactly_once() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int main() {
    try {
        throw Tracer(42);
    } catch (Tracer& t) {
        std::printf("InCatch:%d ", t.id);
    }
    std::printf("AfterCatch");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:42 InCatch:42 D:42 AfterCatch");
    Ok(())
}

#[test]
fn test_rethrow_preserves_exception_and_destroys_at_final_catch() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int main() {
    try {
        try {
            throw Tracer(88);
        } catch (Tracer& t) {
            std::printf("InnerCatch:%d ", t.id);
            throw;
        }
    } catch (Tracer& t2) {
        std::printf("OuterCatch:%d ", t2.id);
    }
    std::printf("Finished");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:88 InnerCatch:88 OuterCatch:88 D:88 Finished");
    Ok(())
}

#[test]
fn test_exception_thrown_in_catch_block_propagates() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    try {
        try {
            throw 10;
        } catch (int e1) {
            std::printf("Catch1:%d ", e1);
            throw 20;
        }
    } catch (int e2) {
        std::printf("Catch2:%d", e2);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Catch1:10 Catch2:20");
    Ok(())
}

#[test]
fn test_negative_cannot_throw_abstract_class() -> Result<(), String> {
    let source = r#"
struct Abstract {
    virtual void pure() = 0;
};

int main() {
    Abstract a;
    throw a;
}
"#;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    match compile_with_provider(source, "<input>", 256, &includes) {
        Err(err) => {
            let err_str = err.to_string();
            if err_str.contains("abstract class") {
                Ok(())
            } else {
                Err(format!("unexpected error: {err_str}"))
            }
        }
        Ok(_) => Err("must fail compilation when throwing abstract class".to_string()),
    }
}

#[test]
fn test_negative_cannot_catch_abstract_class_by_value() -> Result<(), String> {
    let source = r#"
struct Abstract {
    virtual void pure() = 0;
};

int main() {
    try {
        throw 1;
    } catch (Abstract a) {
    }
}
"#;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    match compile_with_provider(source, "<input>", 256, &includes) {
        Err(err) => {
            let err_str = err.to_string();
            if err_str.contains("cannot catch abstract class") {
                Ok(())
            } else {
                Err(format!("unexpected error: {err_str}"))
            }
        }
        Ok(_) => Err("must fail compilation when catching abstract class by value".to_string()),
    }
}

#[test]
fn test_negative_cannot_catch_rvalue_reference() -> Result<(), String> {
    let source = r#"
int main() {
    try {
        throw 1;
    } catch (int&& e) {
    }
}
"#;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    match compile_with_provider(source, "<input>", 256, &includes) {
        Err(err) => {
            let err_str = err.to_string();
            if err_str.contains("rvalue reference") {
                Ok(())
            } else {
                Err(format!("unexpected error: {err_str}"))
            }
        }
        Ok(_) => Err("must fail compilation when catching rvalue reference".to_string()),
    }
}

#[test]
fn test_catch_by_value_destroys_parameter_at_catch_exit() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct TracerVal {
    int id;
    TracerVal(int i) : id(i) { std::printf("C:%d ", id); }
    TracerVal(const TracerVal& o) : id(o.id) { std::printf("Copy:%d ", id); }
    ~TracerVal() { std::printf("D:%d ", id); }
};

int main() {
    try {
        throw TracerVal(10);
    } catch (TracerVal v) {
        std::printf("InCatch:%d ", v.id);
    }
    std::printf("AfterCatch");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:10 Copy:10 InCatch:10 D:10 D:10 AfterCatch");
    Ok(())
}

#[test]
fn test_pointer_const_qualification_conversion() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Widget {
    int val;
    Widget(int v) : val(v) {}
};

Widget g_w(99);

int main() {
    // 1. const Widget* should NOT be caught by non-const Widget*
    try {
        const Widget* pw = &g_w;
        throw pw;
    } catch (Widget* w) {
        std::printf("ErrorShouldNotCatch ");
    } catch (const Widget* cw) {
        std::printf("CaughtConstWidget:%d ", cw->val);
    }

    // 2. non-const Widget* CAN be caught by const Widget*
    try {
        Widget* pw = &g_w;
        throw pw;
    } catch (const Widget* cw) {
        std::printf("CaughtNonConstByConst:%d ", cw->val);
    }

    // 3. const char* should NOT be caught by void*
    try {
        const char* msg = "test";
        throw msg;
    } catch (void* v) {
        std::printf("ErrorVoidPtr ");
    } catch (const void* cv) {
        std::printf("CaughtConstVoid");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtConstWidget:99 CaughtNonConstByConst:99 CaughtConstVoid");
    Ok(())
}

#[test]
fn test_exception_destroyed_on_early_exit_return_and_break() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int test_early_return() {
    try {
        throw Tracer(10);
    } catch (Tracer& t) {
        std::printf("InReturnCatch ");
        return 77;
    }
    return 0;
}

int main() {
    int ret = test_early_return();
    std::printf("RetVal:%d ", ret);

    for (int i = 0; i < 2; i++) {
        try {
            throw Tracer(20);
        } catch (Tracer& t) {
            std::printf("InBreakCatch ");
            break;
        }
    }
    std::printf("AfterBreak");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:10 InReturnCatch D:10 RetVal:77 C:20 InBreakCatch D:20 AfterBreak");
    Ok(())
}

#[test]
fn test_exception_thrown_while_handling_another_exception_lifecycle() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    Tracer(const Tracer& o) : id(o.id) { std::printf("Copy:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int main() {
    try {
        try {
            throw Tracer(1);
        } catch (Tracer& t1) {
            std::printf("InCatch1:%d ", t1.id);
            throw Tracer(2);
        }
    } catch (Tracer& t2) {
        std::printf("InCatch2:%d ", t2.id);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 InCatch1:1 C:2 D:1 InCatch2:2 D:2");
    Ok(())
}

#[test]
fn test_exception_allocation_freed_when_payload_constructor_throws() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct ThrowOnCopy {
    int id;
    ThrowOnCopy(int i) : id(i) {}
    ThrowOnCopy(const ThrowOnCopy& o) : id(o.id) {
        if (id == 99) {
            std::printf("ThrowingFromCopyCtor ");
            throw 123;
        }
    }
};

int main() {
    try {
        ThrowOnCopy obj(99);
        throw obj;
    } catch (int e) {
        std::printf("CaughtInt:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "ThrowingFromCopyCtor CaughtInt:123");
    Ok(())
}

#[test]
fn test_pointer_exception_conversions_and_null_pointer() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Base1 { int b1; Base1() : b1(100) {} };
struct Base2 { int b2; Base2() : b2(200) {} };
struct DerivedMI : Base1, Base2 { int d; DerivedMI() : d(300) {} };

int main() {
    int val = 42;
    int* p = &val;

    // 1. int* should NOT match double* (no type collision)
    try {
        throw p;
    } catch (double* pd) {
        std::printf("ErrorDouble ");
    } catch (int* pi) {
        std::printf("IntPtr:%d ", *pi);
    }

    // 2. Multiple inheritance pointer adjustment for Base2*
    DerivedMI obj;
    try {
        throw &obj;
    } catch (Base2* pb2) {
        std::printf("MI_Base2:%d ", pb2->b2);
    }

    // 3. Null pointer matching Base2* remains null (0)
    try {
        DerivedMI* null_d = nullptr;
        throw null_d;
    } catch (Base2* pb2) {
        std::printf("NullMI:%s ", pb2 == nullptr ? "null" : "non-null");
    }

    // 4. nullptr exception caught by pointer handler
    try {
        throw nullptr;
    } catch (int* pi) {
        std::printf("NullptrCaught:%s", pi == nullptr ? "null" : "non-null");
    }

    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "IntPtr:42 MI_Base2:200 NullMI:null NullptrCaught:null");
    Ok(())
}

#[test]
fn test_catch_handler_selection_source_order_and_multiple_inheritance() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct B1 { int b1; B1(int v) : b1(v) {} };
struct B2 { int b2; B2(int v) : b2(v) {} };
struct D : B1, B2 {
    int d;
    D() : B1(10), B2(20), d(30) {}
};

int main() {
    // 1. Base before Derived handler: base handler is selected per C++17 source order
    try {
        D obj;
        throw obj;
    } catch (B1& b) {
        std::printf("Base1First:%d ", b.b1);
    } catch (D& d) {
        std::printf("ErrorDerived ");
    }

    // 2. Derived before Base handler: derived handler is selected
    try {
        D obj;
        throw obj;
    } catch (D& d) {
        std::printf("DerivedFirst:%d ", d.d);
    } catch (B1& b) {
        std::printf("ErrorBase ");
    }

    // 3. Multiple inheritance: B2 before B1 selects B2 and adjusts subobject offset
    try {
        D obj;
        throw obj;
    } catch (B2& b) {
        std::printf("B2First:%d ", b.b2);
    } catch (B1& b) {
        std::printf("ErrorB1 ");
    }

    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Base1First:10 DerivedFirst:30 B2First:20");
    Ok(())
}

#[test]
fn test_exception_object_move_slicing_and_unnamed_catch() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct MoveTracer {
    int id;
    MoveTracer(int i) : id(i) { std::printf("C:%d ", id); }
    MoveTracer(const MoveTracer& o) : id(o.id) { std::printf("Copy:%d ", id); }
    MoveTracer(MoveTracer&& o) : id(o.id) { std::printf("Move:%d ", id); }
    ~MoveTracer() { std::printf("D:%d ", id); }
};

struct SBase {
    int b;
    SBase(int v) : b(v) { std::printf("SBaseC:%d ", b); }
    SBase(const SBase& o) : b(o.b) { std::printf("SBaseCopy:%d ", b); }
    virtual ~SBase() { std::printf("SBaseD:%d ", b); }
};
struct SDerived : SBase {
    int d;
    SDerived(int bv, int dv) : SBase(bv), d(dv) { std::printf("SDerivedC:%d ", d); }
    SDerived(const SDerived& o) : SBase(o), d(o.d) { std::printf("SDerivedCopy:%d ", d); }
    ~SDerived() override { std::printf("SDerivedD:%d ", d); }
};

int main() {
    // 1. Move constructor preferred when throwing local named variable
    try {
        MoveTracer m(42);
        throw m;
    } catch (MoveTracer& t) {
        std::printf("CaughtMove:%d ", t.id);
    }

    // 2. Unnamed catch-by-value constructs and destroys temporary parameter
    try {
        throw MoveTracer(99);
    } catch (MoveTracer) {
        std::printf("InCatchUnnamed ");
    }

    // 3. Catch derived by value as base slices and runs both destructors
    try {
        SDerived obj(10, 20);
        throw obj;
    } catch (SBase b) {
        std::printf("Sliced:%d ", b.b);
    }

    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "C:42 Move:42 D:42 CaughtMove:42 D:42 C:99 Copy:99 InCatchUnnamed D:99 D:99 SBaseC:10 SDerivedC:20 SBaseCopy:10 SDerivedCopy:20 SDerivedD:20 SBaseD:10 SBaseCopy:10 Sliced:10 SBaseD:10 SDerivedD:20 SBaseD:10"
    );
    Ok(())
}

#[test]
fn test_pointer_indirection_depth_type_id_distinct() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    int val = 42;
    int* p1 = &val;
    int** p2 = &p1;
    int*** p3 = &p2;

    // int*** should NOT be caught by const int**
    try {
        throw p3;
    } catch (const int** p) {
        std::printf("CollisionError ");
    } catch (int*** p) {
        std::printf("CorrectIntTriplePtr:%d", ***p);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectIntTriplePtr:42");
    Ok(())
}

#[test]
fn test_multilevel_pointer_qualification_conversions() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    int val = 42;
    int* p1 = &val;
    int** p2 = &p1;

    // 1. int** caught by const int* const* (valid in C++17)
    try {
        throw p2;
    } catch (const int* const* p) {
        std::printf("CaughtConstIntPtrConstPtr:%d ", **p);
    } catch (...) {
        std::printf("FailedPCP ");
    }

    // 2. int** caught by int* const* (valid in C++17)
    try {
        throw p2;
    } catch (int* const* p) {
        std::printf("CaughtIntPtrConstPtr:%d ", **p);
    } catch (...) {
        std::printf("FailedIPCP ");
    }

    // 3. int** should NOT be caught by const int** (invalid in C++)
    try {
        throw p2;
    } catch (const int** p) {
        std::printf("InvalidMatch ");
    } catch (...) {
        std::printf("CorrectlyRejectedConstIntPtrPtr ");
    }

    // 4. const int** caught by const int* const* (valid in C++17)
    const int cval = 100;
    const int* cp1 = &cval;
    const int** cp2 = &cp1;
    try {
        throw cp2;
    } catch (const int* const* p) {
        std::printf("CaughtCP2ByConstIntPtrConstPtr:%d", **p);
    } catch (...) {
        std::printf("FailedCP2 ");
    }

    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "CaughtConstIntPtrConstPtr:42 CaughtIntPtrConstPtr:42 CorrectlyRejectedConstIntPtrPtr CaughtCP2ByConstIntPtrConstPtr:100"
    );
    Ok(())
}

#[test]
fn test_dce_eliminates_unreferenced_exception_class_destructors() -> Result<(), String> {
    let source = r#"
#include <cstdio>

// An undefined function declaration. If DeadException's destructor is kept by DCE,
// compilation will fail trying to resolve this undefined function.
void undefined_function_that_does_not_exist();

struct DeadException {
    ~DeadException() {
        undefined_function_that_does_not_exist();
    }
};

struct LiveException {
    int code;
    LiveException(int c) : code(c) {}
    ~LiveException() {
        std::printf("LiveExceptionDtor:%d ", code);
    }
};

int main() {
    try {
        throw LiveException(77);
    } catch (LiveException e) {
        std::printf("CaughtLive:%d ", e.code);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "CaughtLive:77 LiveExceptionDtor:77 LiveExceptionDtor:77"
    );
    Ok(())
}

#[test]
fn test_noexcept_function_throwing_terminates() -> Result<(), String> {
    let source = r#"
#include <cstdio>

void bad() noexcept {
    throw 42;
}

int main() {
    try {
        bad();
    } catch (...) {
        std::printf("Caught");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 134, "noexcept function throwing must terminate with exit code 134");
    assert!(!stdout.contains("Caught"), "exception must not be caught by caller");
    Ok(())
}

#[test]
fn test_noexcept_function_internal_catch_succeeds() -> Result<(), String> {
    let source = r#"
#include <cstdio>

void safe() noexcept {
    try {
        throw 1;
    } catch (int e) {
        std::printf("InternalCatch:%d ", e);
    }
}

int main() {
    safe();
    std::printf("Done");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "InternalCatch:1 Done");
    Ok(())
}

#[test]
fn test_noexcept_false_function_propagates() -> Result<(), String> {
    let source = r#"
#include <cstdio>

void prop() noexcept(false) {
    throw 99;
}

int main() {
    try {
        prop();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Caught:99");
    Ok(())
}

#[test]
fn test_noexcept_function_cleans_locals_before_terminating() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

void bad_with_tracer() noexcept {
    Tracer t(123);
    throw 5;
}

int main() {
    try {
        bad_with_tracer();
    } catch (...) {
        std::printf("Caught");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 134, "must terminate with exit code 134");
    assert!(stdout.contains("C:123"), "Tracer must be constructed");
    assert!(stdout.contains("D:123"), "Tracer must be destroyed before termination");
    assert!(!stdout.contains("Caught"), "caller must not catch");
    Ok(())
}

#[test]
fn test_noexcept_constant_expression_true_and_false() -> Result<(), String> {
    let source = r#"
#include <cstdio>

void f_true() noexcept(1 + 1 == 2) {
    throw 1;
}

void f_false() noexcept(1 + 1 == 3) {
    throw 2;
}

int main() {
    try {
        f_false();
    } catch (int e) {
        std::printf("CaughtFalse:%d ", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtFalse:2");

    let source_term = r#"
#include <cstdio>

void f_true() noexcept(1 + 1 == 2) {
    throw 1;
}

int main() {
    try {
        f_true();
    } catch (...) {
        std::printf("Caught");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source_term)?;
    assert_eq!(code, 134, "noexcept(1 + 1 == 2) must terminate on throw");
    assert!(!stdout.contains("Caught"));
    Ok(())
}

#[test]
fn test_noexcept_sizeof_expression() -> Result<(), String> {
    let source = r#"
#include <cstdio>

void f_sizeof() noexcept(sizeof(int) > 100) {
    throw 7;
}

int main() {
    try {
        f_sizeof();
    } catch (int e) {
        std::printf("CaughtSizeof:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtSizeof:7");
    Ok(())
}

#[test]
fn test_noexcept_template_dependent_expression() -> Result<(), String> {
    let source = r#"
#include <cstdio>

template <bool B>
void check() noexcept(B) {
    throw 42;
}

int main() {
    try {
        check<false>();
    } catch (int e) {
        std::printf("CaughtTmplFalse:%d ", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtTmplFalse:42");

    let source_term = r#"
#include <cstdio>

template <bool B>
void check() noexcept(B) {
    throw 42;
}

int main() {
    try {
        check<true>();
    } catch (...) {
        std::printf("Caught");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source_term)?;
    assert_eq!(code, 134, "check<true>() must terminate on throw");
    assert!(!stdout.contains("Caught"));
    Ok(())
}

#[test]
fn test_noexcept_function_pointer_address_and_assignment() -> Result<(), String> {
    let source = r#"
#include <cstdio>

void f() noexcept {
    std::printf("CalledF ");
}

int main() {
    void (*p)() noexcept = &f;
    void (*q)() = p;
    void (*r)() = &f;
    p();
    q();
    r();
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CalledF CalledF CalledF");
    Ok(())
}

#[test]
fn test_noexcept_function_pointer_reverse_fails() -> Result<(), String> {
    let source = r#"
void g() {}

int main() {
    void (*r)() noexcept = &g;
    return 0;
}
"#;
    let res = run_cpp(source);
    assert!(res.is_err(), "assigning throwing function pointer to noexcept pointer must fail");
    Ok(())
}

#[test]
fn test_noexcept_function_reference_conversion() -> Result<(), String> {
    let source = r#"
#include <cstdio>

void f() noexcept {
    std::printf("RefCall ");
}

int main() {
    void (&rf)() = f;
    rf();
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "RefCall");
    Ok(())
}

#[test]
fn test_noexcept_operator_primitive_operations() -> Result<(), String> {
    let source = r#"
static_assert(noexcept(1 + 2), "1 + 2 does not throw");
static_assert(noexcept(true && false), "logical ops do not throw");
int main() {
    return 0;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0);
    Ok(())
}

#[test]
fn test_noexcept_operator_function_calls() -> Result<(), String> {
    let source = r#"
void non_throwing() noexcept;
void throwing();
static_assert(noexcept(non_throwing()), "non_throwing() is noexcept");
static_assert(!noexcept(throwing()), "throwing() is not noexcept");
static_assert(!noexcept(throw 1), "throw 1 is potentially throwing");
int main() {
    return 0;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0);
    Ok(())
}

#[test]
fn test_noexcept_operator_nested() -> Result<(), String> {
    let source = r#"
void throwing();
static_assert(noexcept(noexcept(throwing())), "noexcept operator itself never throws");
int main() {
    return 0;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0);
    Ok(())
}

#[test]
fn test_noexcept_operator_unevaluated_operand() -> Result<(), String> {
    let source = r#"
int counter = 0;
int inc() {
    counter++;
    return 42;
}

int main() {
    bool b = noexcept(inc());
    if (counter != 0) return 1;
    if (b) return 2;
    return 0;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0);
    Ok(())
}

#[test]
fn test_destructor_implicitly_noexcept_terminates_on_throw() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct ThrowingDtor {
    int val;
    ThrowingDtor(int v) : val(v) {}
    ~ThrowingDtor() {
        std::printf("EnteringDtor:%d ", val);
        throw val;
    }
};

void test_fn() {
    ThrowingDtor t(42);
}

int main() {
    try {
        test_fn();
    } catch (...) {
        std::printf("Caught ");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 134, "unannotated throwing destructor must terminate");
    assert!(stdout.contains("EnteringDtor:42"));
    assert!(!stdout.contains("Caught"));
    Ok(())
}

#[test]
fn test_destructor_explicit_noexcept_false_propagates() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct ThrowingDtor {
    int val;
    ThrowingDtor(int v) : val(v) {}
    ~ThrowingDtor() noexcept(false) {
        std::printf("EnteringDtor:%d ", val);
        throw val;
    }
};

void test_fn() {
    ThrowingDtor t(42);
}

int main() {
    try {
        test_fn();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "EnteringDtor:42 Caught:42");
    Ok(())
}

#[test]
fn test_noexcept_operator_on_destructor() -> Result<(), String> {
    let source = r#"
struct Tracer {
    ~Tracer() {}
};
struct Thrower {
    ~Thrower() noexcept(false) {}
};

int main() {
    Tracer t;
    Thrower th;
    bool b1 = noexcept(t.~Tracer());
    bool b2 = noexcept(th.~Thrower());
    if (!b1) return 1;
    if (b2) return 2;
    return 0;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0);
    Ok(())
}

#[test]
fn test_non_template_noexcept_specification_evaluation() -> Result<(), String> {
    let source = r#"
void throwing_func() { throw 42; }
void non_throwing() noexcept {}

void f_throw() noexcept(noexcept(throwing_func())) {}
void f_nothrow() noexcept(noexcept(non_throwing())) {}

struct S {
    void m_throw() noexcept(noexcept(throwing_func())) {}
    void m_nothrow() noexcept(noexcept(non_throwing())) {}
};

int main() {
    if (noexcept(f_throw())) return 1;
    if (!noexcept(f_nothrow())) return 2;
    S s;
    if (noexcept(s.m_throw())) return 3;
    if (!noexcept(s.m_nothrow())) return 4;
    return 0;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0);
    Ok(())
}

#[test]
fn test_noexcept_multilevel_function_pointer_conversion_rejected() {
    let source = r#"
void f() noexcept {}
using FnNoexcept = void (*)() noexcept;
using Fn = void (*)();
int main() {
    FnNoexcept p = f;
    Fn* pp = &p;
    return 0;
}
"#;
    let res = run_cpp(source);
    assert!(res.is_err());
}

#[test]
fn test_std_exception_base_what() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>

int main() {
    try {
        throw std::exception();
    } catch (const std::exception& e) {
        std::printf("Caught:%s", e.what());
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Caught:std::exception");
    Ok(())
}

#[test]
fn test_std_bad_exception_polymorphic_catch() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>

void do_throw() {
    throw std::bad_exception();
}

int main() {
    try {
        do_throw();
    } catch (const std::exception& e) {
        std::printf("Polymorphic:%s", e.what());
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Polymorphic:std::bad_exception");
    Ok(())
}

#[test]
fn test_std_terminate_custom_handler() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>
#include <cstdlib>

void my_terminate() {
    std::printf("CustomTerminateCalled\n");
    std::exit(42);
}

int main() {
    std::set_terminate(my_terminate);
    std::terminate();
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 42);
    assert_eq!(stdout.trim(), "CustomTerminateCalled");
    Ok(())
}

#[test]
fn test_std_uncaught_exceptions_query() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>

int main() {
    int u = std::uncaught_exceptions();
    bool b = std::uncaught_exception();
    std::printf("Uncaught:%d,%d", u, (int)b);
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Uncaught:0,0");
    Ok(())
}

#[test]
fn test_std_exception_copy_and_slicing() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>

int main() {
    std::exception e1;
    std::exception e2 = e1;
    std::exception e3;
    e3 = e2;

    try {
        throw std::bad_exception();
    } catch (std::exception sliced) {
        // Slicing occurs when caught by value
        std::printf("Sliced:%s ", sliced.what());
    }

    try {
        throw std::bad_exception();
    } catch (const std::exception& ref) {
        // Virtual dispatch preserved when caught by reference
        std::printf("Ref:%s", ref.what());
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Sliced:std::exception Ref:std::bad_exception");
    Ok(())
}

#[test]
fn test_std_exception_derived_custom_virtual_dtor() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>

static int dtor_count = 0;

class MyCustomException : public std::exception {
public:
    MyCustomException() noexcept {}
    virtual ~MyCustomException() noexcept {
        dtor_count++;
    }
    virtual const char* what() const noexcept {
        return "custom exception message";
    }
};

void throw_custom() {
    throw MyCustomException();
}

int main() {
    try {
        throw_custom();
    } catch (const std::exception& e) {
        std::printf("CaughtCustom:%s ", e.what());
    }
    std::printf("DtorCount:%d", dtor_count);
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtCustom:custom exception message DtorCount:1");
    Ok(())
}

#[test]
fn test_stdexcept_hierarchy_and_what() -> Result<(), String> {
    let source = r#"
#include <stdexcept>
#include <cstdio>
#include <string>

void test_classes() {
    // 1. logic_error
    try {
        throw std::logic_error("logic error msg");
    } catch (const std::logic_error& e) {
        std::printf("1:%s ", e.what());
    }

    // 2. domain_error
    try {
        throw std::domain_error("domain error msg");
    } catch (const std::domain_error& e) {
        std::printf("2:%s ", e.what());
    }

    // 3. invalid_argument
    try {
        throw std::invalid_argument("invalid argument msg");
    } catch (const std::invalid_argument& e) {
        std::printf("3:%s ", e.what());
    }

    // 4. length_error
    try {
        throw std::length_error("length error msg");
    } catch (const std::length_error& e) {
        std::printf("4:%s ", e.what());
    }

    // 5. out_of_range
    try {
        throw std::out_of_range("out of range msg");
    } catch (const std::out_of_range& e) {
        std::printf("5:%s ", e.what());
    }

    // 6. runtime_error
    try {
        throw std::runtime_error("runtime error msg");
    } catch (const std::runtime_error& e) {
        std::printf("6:%s ", e.what());
    }

    // 7. range_error
    try {
        throw std::range_error("range error msg");
    } catch (const std::range_error& e) {
        std::printf("7:%s ", e.what());
    }

    // 8. overflow_error
    try {
        throw std::overflow_error("overflow error msg");
    } catch (const std::overflow_error& e) {
        std::printf("8:%s ", e.what());
    }

    // 9. underflow_error
    try {
        throw std::underflow_error("underflow error msg");
    } catch (const std::underflow_error& e) {
        std::printf("9:%s ", e.what());
    }
}

int main() {
    test_classes();
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "1:logic error msg 2:domain error msg 3:invalid argument msg 4:length error msg 5:out of range msg 6:runtime error msg 7:range error msg 8:overflow error msg 9:underflow error msg"
    );
    Ok(())
}

#[test]
fn test_stdexcept_polymorphic_catch_base_classes() -> Result<(), String> {
    let source = r#"
#include <stdexcept>
#include <cstdio>
#include <string>

int main() {
    // Catch derived logic_error through std::exception
    try {
        throw std::out_of_range("out of range through std::exception");
    } catch (const std::exception& e) {
        std::printf("ExCatch:%s ", e.what());
    }

    // Catch derived logic_error through std::logic_error
    try {
        throw std::invalid_argument("invalid argument through std::logic_error");
    } catch (const std::logic_error& e) {
        std::printf("LogicCatch:%s ", e.what());
    }

    // Catch derived runtime_error through std::exception
    try {
        throw std::overflow_error("overflow error through std::exception");
    } catch (const std::exception& e) {
        std::printf("ExCatch2:%s ", e.what());
    }

    // Catch derived runtime_error through std::runtime_error
    try {
        throw std::underflow_error("underflow error through std::runtime_error");
    } catch (const std::runtime_error& e) {
        std::printf("RuntimeCatch:%s", e.what());
    }

    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "ExCatch:out of range through std::exception LogicCatch:invalid argument through std::logic_error ExCatch2:overflow error through std::exception RuntimeCatch:underflow error through std::runtime_error"
    );
    Ok(())
}

#[test]
fn test_stdexcept_string_constructor_and_copy() -> Result<(), String> {
    let source = r#"
#include <stdexcept>
#include <cstdio>
#include <string>

int main() {
    std::string base_msg = "constructed from std::string";
    try {
        std::runtime_error original(base_msg);
        std::runtime_error copy = original;
        throw copy;
    } catch (const std::runtime_error& caught) {
        std::printf("CaughtCopy:%s", caught.what());
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtCopy:constructed from std::string");
    Ok(())
}

#[test]
fn test_stdexcept_realistic_scenarios() -> Result<(), String> {
    let source = r#"
#include <stdexcept>
#include <cstdio>
#include <string>

int safe_divide(int a, int b) {
    if (b == 0) {
        throw std::invalid_argument("division by zero");
    }
    return a / b;
}

int get_element(int* arr, int size, int index) {
    if (index < 0 || index >= size) {
        throw std::out_of_range("array index out of bounds");
    }
    return arr[index];
}

void process_transaction(int amount) {
    if (amount > 1000000) {
        throw std::runtime_error("transaction limit exceeded");
    }
}

int main() {
    try {
        safe_divide(10, 0);
    } catch (const std::invalid_argument& e) {
        std::printf("CaughtDivZero:%s ", e.what());
    }

    int nums[3] = {10, 20, 30};
    try {
        get_element(nums, 3, 5);
    } catch (const std::out_of_range& e) {
        std::printf("CaughtBounds:%s ", e.what());
    }

    try {
        process_transaction(2000000);
    } catch (const std::runtime_error& e) {
        std::printf("CaughtTx:%s", e.what());
    }

    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "CaughtDivZero:division by zero CaughtBounds:array index out of bounds CaughtTx:transaction limit exceeded"
    );
    Ok(())
}

#[test]
fn test_stdexcept_nested_throw_in_catch() -> Result<(), String> {
    let source = r#"
#include <stdexcept>
#include <cstdio>

int main() {
    try {
        try {
            throw std::runtime_error("first runtime error");
        } catch (const std::exception& e1) {
            std::printf("FirstCaught:%s ", e1.what());
            throw std::logic_error("second logic error");
        }
    } catch (const std::exception& e2) {
        std::printf("SecondCaught:%s", e2.what());
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "FirstCaught:first runtime error SecondCaught:second logic error"
    );
    Ok(())
}

fn assert_compile_error(source: &str, needle: &str) -> Result<(), String> {
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

#[test]
fn test_negative_throw_deleted_copy_ctor() -> Result<(), String> {
    let source = r#"
struct NoCopy {
    NoCopy() {}
    NoCopy(const NoCopy&) = delete;
    NoCopy(NoCopy&&) = delete;
};
int main() {
    NoCopy a;
    try { throw a; } catch (...) {}
    return 0;
}
"#;
    assert_compile_error(source, "copy constructor")
}

#[test]
fn test_negative_catch_private_copy_ctor() -> Result<(), String> {
    let source = r#"
struct NoCopy {
    NoCopy() {}
private:
    NoCopy(const NoCopy&) {}
};
int main() {
    try { throw 1; } catch (NoCopy) {}
    return 0;
}
"#;
    assert_compile_error(source, "copy constructor")
}

#[test]
fn test_outer_local_variable_throw_copies_instead_of_moving() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct S {
    int v;
    S(int x) : v(x) {}
    S(const S& o) : v(o.v) { std::printf("Copy "); }
    S(S&& o) : v(o.v) { std::printf("Move "); }
    ~S() {}
};

int main() {
    S s(1);
    try {
        throw s;
    } catch (...) {
        std::printf("Caught ");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert!(
        stdout.contains("Copy"),
        "an outer local must be copied on throw: {stdout}"
    );
    assert!(
        !stdout.contains("Move"),
        "an outer local must not be moved on throw: {stdout}"
    );
    Ok(())
}

#[test]
fn test_private_base_catch_is_rejected() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Base { int tag; };
struct Derived : private Base { Derived() { tag = 7; } };

int main() {
    Derived d;
    try {
        throw d;
    } catch (Base&) {
        std::printf("WrongPrivate ");
    } catch (...) {
        std::printf("CorrectFallback");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectFallback");
    Ok(())
}

#[test]
fn test_ambiguous_base_catch_is_rejected() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Base { int tag; };
struct Left : Base {};
struct Right : Base {};
struct Bottom : Left, Right {};

int main() {
    Bottom b;
    try {
        throw b;
    } catch (Base&) {
        std::printf("WrongAmbiguous ");
    } catch (...) {
        std::printf("CorrectFallback");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectFallback");
    Ok(())
}

#[test]
fn test_enum_exception_type_id_is_distinct() -> Result<(), String> {
    let source = r#"
#include <cstdio>

enum class Color { Red, Green };
enum class Fruit { Apple };

int main() {
    try {
        throw Color::Red;
    } catch (int) {
        std::printf("WrongInt ");
    } catch (Fruit) {
        std::printf("WrongFruit ");
    } catch (Color c) {
        std::printf("CorrectColor:%d", (int)c);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectColor:0");
    Ok(())
}

#[test]
fn test_function_pointer_exception_type_ids_are_distinct() -> Result<(), String> {
    let source = r#"
#include <cstdio>

using VA = void(*)();
using IA = int(*)(int);
void foo() {}

int main() {
    VA p = &foo;
    try {
        throw p;
    } catch (IA) {
        std::printf("WrongSignature ");
    } catch (VA) {
        std::printf("CorrectVA");
    } catch (...) {
        std::printf("Fallback");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectVA");
    Ok(())
}

#[test]
fn test_member_pointer_exception_type_id_is_distinct() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct S { int x; };
using MP = int S::*;
using FP = void(*)();
void foo() {}

int main() {
    MP p = &S::x;
    try {
        throw p;
    } catch (FP) {
        std::printf("WrongFP ");
    } catch (MP) {
        std::printf("CorrectMP");
    } catch (...) {
        std::printf("Fallback");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectMP");
    Ok(())
}

#[test]
fn test_nullptr_matches_member_pointer_handler() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct S { int x; };
using MP = int S::*;

int main() {
    try {
        throw nullptr;
    } catch (int) {
        std::printf("WrongInt ");
    } catch (MP) {
        std::printf("CorrectNullMP");
    } catch (...) {
        std::printf("Fallback");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectNullMP");
    Ok(())
}

#[test]
fn test_noexcept_violation_invokes_custom_terminate_handler() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>
#include <cstdlib>

void my_terminate() {
    std::printf("CustomTerminateCalled\n");
    std::exit(42);
}

void bad() noexcept {
    throw 7;
}

int main() {
    std::set_terminate(my_terminate);
    bad();
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(
        code, 42,
        "noexcept violation must invoke the custom terminate handler"
    );
    assert_eq!(stdout.trim(), "CustomTerminateCalled");
    Ok(())
}

#[test]
fn test_throwing_during_unwinding_invokes_custom_terminate_handler() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>
#include <cstdlib>

void my_terminate() {
    std::printf("CustomTerminateCalled\n");
    std::exit(42);
}

struct Bad {
    ~Bad() noexcept(false) { throw 9; }
};

void inner() { throw 1; }
void middle() { Bad b; inner(); }

int main() {
    std::set_terminate(my_terminate);
    try {
        middle();
    } catch (int) {
        std::printf("Caught");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(
        code, 42,
        "a throwing destructor during unwinding must invoke the custom terminate handler"
    );
    assert_eq!(stdout.trim(), "CustomTerminateCalled");
    Ok(())
}

#[test]
fn test_uncaught_exception_from_main_invokes_custom_terminate_handler() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>
#include <cstdlib>

void my_terminate() {
    std::printf("CustomTerminateCalled\n");
    std::exit(42);
}

int main() {
    std::set_terminate(my_terminate);
    throw 5;
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(
        code, 42,
        "an uncaught exception escaping main must invoke std::terminate"
    );
    assert_eq!(stdout.trim(), "CustomTerminateCalled");
    Ok(())
}

#[test]
fn test_rethrow_without_active_exception_invokes_custom_terminate_handler() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>
#include <cstdlib>

void my_terminate() {
    std::printf("CustomTerminateCalled\n");
    std::exit(42);
}

void foo() { throw; }

int main() {
    std::set_terminate(my_terminate);
    foo();
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(
        code, 42,
        "throw; without an active exception must invoke the custom terminate handler"
    );
    assert_eq!(stdout.trim(), "CustomTerminateCalled");
    Ok(())
}
