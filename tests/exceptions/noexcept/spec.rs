use crate::common::run_cpp;

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
