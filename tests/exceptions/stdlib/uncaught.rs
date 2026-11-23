use crate::common::run_cpp;

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
fn test_uncaught_exceptions_during_unwinding() -> Result<(), String> {
    let source = r#"
#include <cstdio>
#include <exception>

struct Tracer {
    ~Tracer() noexcept {
        std::printf("Uncaught:%d\n", std::uncaught_exceptions());
    }
};

void inner() {
    Tracer t;
    throw 1;
}

int main() {
    try {
        inner();
    } catch (...) {
        std::printf("Count:%d\n", std::uncaught_exceptions());
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Uncaught:1\nCount:0");
    Ok(())
}

#[test]
fn test_uncaught_exceptions_during_rethrow() -> Result<(), String> {
    let source = r#"
#include <cstdio>
#include <exception>

struct Tracer {
    int id;
    ~Tracer() noexcept {
        std::printf("Dtor%d:%d ", id, std::uncaught_exceptions());
    }
};

void g() {
    Tracer t{1};
    throw 42;
}

void f() {
    try {
        g();
    } catch (int) {
        std::printf("InCatch1:%d ", std::uncaught_exceptions());
        Tracer t{2};
        throw;
    }
}

int main() {
    try {
        f();
    } catch (int) {
        std::printf("InCatch2:%d ", std::uncaught_exceptions());
    }
    std::printf("Done:%d", std::uncaught_exceptions());
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "Dtor1:1 InCatch1:0 Dtor2:1 InCatch2:0 Done:0"
    );
    Ok(())
}
