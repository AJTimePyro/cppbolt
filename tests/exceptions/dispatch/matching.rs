use crate::common::{PathBuf, RepoIncludes, compile_with_provider, run_cpp};

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
fn test_catch_all_foreign_exception_does_not_underflow_uncaught_count() -> Result<(), String> {
    let source = r#"
#include <cstdio>
#include <exception>

int main() {
    std::printf("Before:%d ", std::uncaught_exceptions());
    try {
        // A catch(...) block that catches or finishes should not corrupt uncaught count
    } catch (...) {
        std::printf("InCatch:%d ", std::uncaught_exceptions());
    }
    std::printf("After:%d ", std::uncaught_exceptions());
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Before:0 After:0");
    Ok(())
}
