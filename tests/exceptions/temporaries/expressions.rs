use crate::common::run_cpp;

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
    assert_eq!(
        stdout.trim(),
        "C:10 C:20 C:30 BarThrow D:30 D:20 D:10 Caught:99"
    );
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
    assert_eq!(
        stdout.trim(),
        "Make:100 C_Temp:100 ThrowNested D_Temp:100 Caught:88"
    );
    Ok(())
}

#[test]
fn test_ternary_conditional_temporaries_normal_exit() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Widget {
    int id;
    Widget(int i) : id(i) { std::printf("C_W:%d ", id); }
    ~Widget() { std::printf("D_W:%d ", id); }
};

int consume(const Widget& w) {
    return w.id;
}

int main() {
    int r1 = 1 ? consume(Widget(1)) : consume(Widget(2));
    std::printf("r1=%d ", r1);
    int r2 = 0 ? consume(Widget(3)) : consume(Widget(4));
    std::printf("r2=%d", r2);
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C_W:1 D_W:1 r1=1 C_W:4 D_W:4 r2=4");
    Ok(())
}

#[test]
fn test_short_circuit_temporaries_with_exceptions() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Widget {
    int id;
    Widget(int i) : id(i) {
        std::printf("C_W:%d ", id);
        if (id == 99) {
            std::printf("Throw_W ");
            throw 999;
        }
    }
    ~Widget() { std::printf("D_W:%d ", id); }
};

int consume(const Widget& w) {
    return w.id;
}

int main() {
    try {
        // false && ... should not evaluate second operand
        int x = (0 && consume(Widget(1))) ? 10 : (consume(Widget(2)) + consume(Widget(99)));
        std::printf("x=%d ", x);
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C_W:2 C_W:99 Throw_W D_W:2 Caught:999");
    Ok(())
}

#[test]
fn test_exception_thrown_in_conditional_expression_preserves_destructor() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct ExprExn {
    int val;
    ExprExn(int v) : val(v) {}
    ~ExprExn() { std::printf("DtorExn:%d ", val); }
};

int compute(bool fail) {
    return fail ? throw ExprExn(7) : 42;
}

int main() {
    try {
        compute(true);
    } catch (ExprExn& e) {
        std::printf("Caught:%d ", e.val);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Caught:7 DtorExn:7");
    Ok(())
}
