use crate::common::run_cpp;

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
    assert!(
        res.is_err(),
        "assigning throwing function pointer to noexcept pointer must fail"
    );
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
