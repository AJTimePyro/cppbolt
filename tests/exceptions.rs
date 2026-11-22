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
    ~ThrowingDtor() {
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
    ~ThrowingDtor() {
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

