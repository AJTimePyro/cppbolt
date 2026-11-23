use crate::common::run_cpp;

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
    assert_eq!(
        code, 42,
        "inner() throwing 42 caught by main() must return exit code 42"
    );
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
    assert_eq!(
        stdout.trim(),
        "C:100 C:10 D:10 Caught:1 C:20 D:20 Caught:2 End D:100"
    );
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
