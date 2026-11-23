use crate::common::run_cpp;

#[test]
fn test_throw_and_catch_class_by_value() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct MyError {
    int code;
    MyError(int c) : code(c) {}
};

int main() {
    try {
        throw MyError(42);
    } catch (MyError err) {
        std::printf("CaughtByValue:%d", err.code);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtByValue:42");
    Ok(())
}

#[test]
fn test_throw_and_catch_class_by_const_reference() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct MyError {
    int code;
    MyError(int c) : code(c) {}
};

int main() {
    try {
        throw MyError(77);
    } catch (const MyError& err) {
        std::printf("CaughtByConstRef:%d", err.code);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtByConstRef:77");
    Ok(())
}

#[test]
fn test_throw_and_catch_class_by_non_const_reference_mutation() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct MyError {
    int code;
    MyError(int c) : code(c) {}
};

int main() {
    try {
        try {
            throw MyError(50);
        } catch (MyError& err) {
            std::printf("InnerRef:%d ", err.code);
            err.code += 25;
            throw;
        }
    } catch (const MyError& err2) {
        std::printf("OuterRefMutated:%d", err2.code);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "InnerRef:50 OuterRefMutated:75");
    Ok(())
}

#[test]
fn test_derived_exception_caught_by_base_reference_single_inheritance() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Base {
    int b;
    Base(int val) : b(val) {}
};

struct Derived : public Base {
    int d;
    Derived(int b_val, int d_val) : Base(b_val), d(d_val) {}
};

int main() {
    try {
        throw Derived(101, 202);
    } catch (Base& b) {
        std::printf("CaughtBaseRef:%d", b.b);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtBaseRef:101");
    Ok(())
}

#[test]
fn test_derived_exception_caught_by_secondary_base_multiple_inheritance() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Base1 {
    int b1;
    Base1(int x) : b1(x) {}
};

struct Base2 {
    int b2;
    Base2(int y) : b2(y) {}
};

struct Derived : public Base1, public Base2 {
    int d;
    Derived(int x, int y, int z) : Base1(x), Base2(y), d(z) {}
};

int main() {
    try {
        throw Derived(11, 22, 33);
    } catch (Base2& b2) {
        std::printf("CaughtBase2Ref:%d", b2.b2);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtBase2Ref:22");
    Ok(())
}

#[test]
fn test_exact_type_handler_preferred_before_base() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Base {
    int b;
    Base(int val) : b(val) {}
};

struct Derived : public Base {
    int d;
    Derived(int b_val, int d_val) : Base(b_val), d(d_val) {}
};

int main() {
    try {
        throw Derived(10, 20);
    } catch (Derived& d) {
        std::printf("CaughtExact:%d", d.d);
    } catch (Base& b) {
        std::printf("CaughtBase:%d", b.b);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtExact:20");
    Ok(())
}

#[test]
fn test_catch_by_value_destroys_parameter_at_catch_exit() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct TracerVal {
    int id;
    TracerVal(int i) : id(i) { std::printf("C:%d ", id); }
    TracerVal(const TracerVal& o) : id(o.id) { std::printf("Copy:%d ", id); }
    ~TracerVal() { std::printf("D:%d ", id); }
};

int main() {
    try {
        throw TracerVal(10);
    } catch (TracerVal v) {
        std::printf("InCatch:%d ", v.id);
    }
    std::printf("AfterCatch");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "C:10 Copy:10 InCatch:10 D:10 D:10 AfterCatch"
    );
    Ok(())
}

#[test]
fn test_catch_handler_selection_source_order_and_multiple_inheritance() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct B1 { int b1; B1(int v) : b1(v) {} };
struct B2 { int b2; B2(int v) : b2(v) {} };
struct D : B1, B2 {
    int d;
    D() : B1(10), B2(20), d(30) {}
};

int main() {
    // 1. Base before Derived handler: base handler is selected per C++17 source order
    try {
        D obj;
        throw obj;
    } catch (B1& b) {
        std::printf("Base1First:%d ", b.b1);
    } catch (D& d) {
        std::printf("ErrorDerived ");
    }

    // 2. Derived before Base handler: derived handler is selected
    try {
        D obj;
        throw obj;
    } catch (D& d) {
        std::printf("DerivedFirst:%d ", d.d);
    } catch (B1& b) {
        std::printf("ErrorBase ");
    }

    // 3. Multiple inheritance: B2 before B1 selects B2 and adjusts subobject offset
    try {
        D obj;
        throw obj;
    } catch (B2& b) {
        std::printf("B2First:%d ", b.b2);
    } catch (B1& b) {
        std::printf("ErrorB1 ");
    }

    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Base1First:10 DerivedFirst:30 B2First:20");
    Ok(())
}

#[test]
fn test_private_base_catch_is_rejected() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Base { int tag; };
struct Derived : private Base { Derived() { tag = 7; } };

int main() {
    Derived d;
    try {
        throw d;
    } catch (Base&) {
        std::printf("WrongPrivate ");
    } catch (...) {
        std::printf("CorrectFallback");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectFallback");
    Ok(())
}

#[test]
fn test_ambiguous_base_catch_is_rejected() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Base { int tag; };
struct Left : Base {};
struct Right : Base {};
struct Bottom : Left, Right {};

int main() {
    Bottom b;
    try {
        throw b;
    } catch (Base&) {
        std::printf("WrongAmbiguous ");
    } catch (...) {
        std::printf("CorrectFallback");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectFallback");
    Ok(())
}
