use crate::common::run_cpp;

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
    assert_eq!(
        code, 134,
        "throw; outside catch must terminate with abort/unreachable"
    );
    assert!(!stdout.contains("Caught"));
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
    assert_eq!(
        stdout.trim(),
        "OuterCatch:10 InnerCatch:20 CaughtRethrown:10"
    );
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
    assert_eq!(
        stdout.trim(),
        "C:88 InnerCatch:88 OuterCatch:88 D:88 Finished"
    );
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
