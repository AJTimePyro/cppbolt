use crate::common::run_cpp;

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
fn test_noexcept_expression_accounts_for_overloaded_operator() -> Result<(), String> {
    let source = r#"
struct S {
    int v;
    S operator+(const S& o) const { return *this; }
};

struct T {
    T operator+(const T& o) const noexcept { return *this; }
};

struct U {
    int operator[](int i) const { return i; }
};

S a;
S b;
T c;
T d;
U u;
static_assert(!noexcept(a + b), "operator+ is not noexcept");
static_assert(noexcept(c + d), "noexcept operator+ must stay noexcept");
static_assert(!noexcept(u[0]), "operator[] is not noexcept");
int main() { return 0; }
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0);
    Ok(())
}
