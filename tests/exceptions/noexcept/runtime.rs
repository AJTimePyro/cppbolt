use crate::common::run_cpp;

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
    assert_eq!(
        code, 134,
        "noexcept function throwing must terminate with exit code 134"
    );
    assert!(
        !stdout.contains("Caught"),
        "exception must not be caught by caller"
    );
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
    assert!(
        stdout.contains("D:123"),
        "Tracer must be destroyed before termination"
    );
    assert!(!stdout.contains("Caught"), "caller must not catch");
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
fn test_noexcept_function_unwind_handler_emits_unreachable() -> Result<(), String> {
    let source = r#"
#include <cstdio>
#include <exception>

void custom_term() {
    std::printf("CustomTerminated ");
}

void throwing() { throw 1; }

void noexcept_fn() noexcept {
    throwing();
}

int main() {
    std::set_terminate(custom_term);
    try {
        noexcept_fn();
    } catch (...) {
        std::printf("Caught ");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 134);
    assert_eq!(stdout.trim(), "CustomTerminated");
    Ok(())
}
