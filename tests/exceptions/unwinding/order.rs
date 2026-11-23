use crate::common::run_cpp;

#[test]
fn test_multiple_objects_reverse_destruction_order() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

void fail() {
    Tracer a(1);
    Tracer b(2);
    Tracer c(3);
    throw 99;
}

int main() {
    try {
        fail();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 C:3 D:3 D:2 D:1 Caught:99");
    Ok(())
}

#[test]
fn test_conditional_object_construction() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

void choose(bool flag) {
    Tracer a(1);
    if (flag) {
        Tracer b(2);
        throw 10;
    } else {
        Tracer c(3);
        throw 20;
    }
}

int main() {
    try { choose(true); } catch (int e) { std::printf("E:%d ", e); }
    try { choose(false); } catch (int e) { std::printf("E:%d ", e); }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 D:2 D:1 E:10 C:1 C:3 D:3 D:1 E:20");
    Ok(())
}

#[test]
fn test_unwinding_destroys_temporaries_before_enclosing_locals() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct RecycledTemp {
    int id;
    RecycledTemp(int i) : id(i) { std::printf("C_Temp:%d ", id); }
    ~RecycledTemp() { std::printf("D_Temp:%d ", id); }
};

struct LocalVar {
    int id;
    LocalVar(int i) : id(i) { std::printf("C_Local:%d ", id); }
    ~LocalVar() { std::printf("D_Local:%d ", id); }
};

void consume(RecycledTemp, int) {}
int throwing() { throw 42; }

int main() {
    consume(RecycledTemp(1), 0);
    std::printf("---Try--- ");
    try {
        LocalVar v(2);
        consume(RecycledTemp(3), throwing());
    } catch (int) {
        std::printf("Caught ");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "C_Temp:1 D_Temp:1 ---Try--- C_Local:2 C_Temp:3 D_Temp:3 D_Local:2 Caught"
    );
    Ok(())
}

#[test]
fn test_expression_throw_triggers_function_unwinding_analysis() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct DtorGuard {
    int id;
    ~DtorGuard() {
        std::printf("Dtor:%d ", id);
    }
};

int compute(bool fail) {
    DtorGuard g{1};
    return fail ? throw 42 : 100;
}

int main() {
    try {
        compute(true);
    } catch (int val) {
        std::printf("Caught:%d", val);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Dtor:1 Caught:42");
    Ok(())
}
