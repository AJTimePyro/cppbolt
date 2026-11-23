use crate::common::run_cpp;

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
