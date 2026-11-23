use crate::common::{assert_compile_error, run_cpp};

#[test]
fn test_constructor_failure_unwinds_completed_subobjects() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

struct Complex {
    Tracer t1;
    Tracer t2;
    Complex() : t1(1), t2(2) {
        std::printf("CtorFail ");
        throw 99;
    }
    ~Complex() {
        std::printf("DtorComplex ");
    }
};

int main() {
    try {
        Complex c;
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 CtorFail D:2 D:1 Caught:99");
    Ok(())
}

#[test]
fn test_constructor_failure_cleans_completed_arguments_and_subobjects() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Temp {
    int id;
    Temp(int i) : id(i) { std::printf("C_Temp:%d ", id); }
    ~Temp() { std::printf("D_Temp:%d ", id); }
};

struct Sub {
    int id;
    Sub(int i) : id(i) { std::printf("C_Sub:%d ", id); }
    ~Sub() { std::printf("D_Sub:%d ", id); }
};

struct FailingClass {
    Sub s;
    FailingClass(Temp t) : s(1) {
        std::printf("CtorThrow ");
        throw 123;
    }
    ~FailingClass() {
        std::printf("D_Failing ");
    }
};

void run_test() {
    FailingClass obj(Temp(2));
}

int main() {
    try {
        run_test();
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
        "C_Temp:2 C_Sub:1 CtorThrow D_Sub:1 D_Temp:2 Caught:123"
    );
    Ok(())
}

#[test]
fn test_constructor_function_try_block_rethrows() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct A {
    A() try {
        std::printf("Ctor ");
        throw 42;
    } catch (...) {
        std::printf("Handler ");
    }
};

int main() {
    try {
        A a;
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Ctor Handler Caught:42");
    Ok(())
}

#[test]
fn test_constructor_function_try_block_with_member_initializers() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct B {
    int x;
    B() try : x(42) {
        std::printf("Ctor:%d ", x);
    } catch (...) {
        std::printf("Handler ");
    }
};

int main() {
    B b;
    std::printf("Done:%d", b.x);
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Ctor:42 Done:42");
    Ok(())
}

#[test]
fn test_constructor_function_try_block_member_throw() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Member {
    int id;
    Member(int id) : id(id) {
        std::printf("MInit:%d ", id);
        if (id == 2) throw 99;
    }
    ~Member() {
        std::printf("MDtor:%d ", id);
    }
};

struct Base {
    Base() { std::printf("BInit "); }
    ~Base() { std::printf("BDtor "); }
};

struct Target : Base {
    Member m1;
    Member m2;
    Member m3;
    Target() try : Base(), m1(1), m2(2), m3(3) {
        std::printf("TargetBody ");
    } catch (int e) {
        std::printf("TargetHandler:%d ", e);
    }
};

int main() {
    try {
        Target t;
    } catch (int e) {
        std::printf("MainCaught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "BInit MInit:1 MInit:2 MDtor:1 BDtor TargetHandler:99 MainCaught:99"
    );
    Ok(())
}

#[test]
fn test_constructor_function_try_block_body_throw_destroys_subobjects() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Member {
    int id;
    Member(int id) : id(id) { std::printf("MInit:%d ", id); }
    ~Member() { std::printf("MDtor:%d ", id); }
};

struct Target {
    Member m1;
    Member m2;
    Target() try : m1(1), m2(2) {
        std::printf("TargetBody ");
        throw 77;
    } catch (int e) {
        std::printf("Handler:%d ", e);
    }
};

int main() {
    try {
        Target t;
    } catch (int e) {
        std::printf("MainCaught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "MInit:1 MInit:2 TargetBody MDtor:2 MDtor:1 Handler:77 MainCaught:77"
    );
    Ok(())
}

#[test]
fn test_constructor_function_try_block_rejects_return() -> Result<(), String> {
    let source = r#"
struct A {
    int x;
    A() try : x(1) {
    } catch (...) {
        return;
    }
};
int main() { return 0; }
"#;
    assert_compile_error(
        source,
        "return statements cannot appear in a function-try-block handler",
    )
}
