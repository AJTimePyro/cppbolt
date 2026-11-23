use crate::common::run_cpp;

#[test]
fn test_nested_calls_in_argument_expressions() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int thrower() {
    throw 42;
    return 10;
}

void sink(int x) {
    std::printf("sink:%d ", x);
}

void middle() {
    Tracer t(55);
    sink(thrower());
}

int main() {
    try {
        middle();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:55 D:55 Caught:42");
    Ok(())
}

#[test]
fn test_member_virtual_and_function_pointer_calls_unwind() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

struct Worker {
    void work() { throw 11; }
};

struct Base {
    virtual void vwork() = 0;
};
struct Derived : Base {
    virtual void vwork() override { throw 22; }
};

void fn_throw() { throw 33; }

void test_member() { Tracer t(1); Worker w; w.work(); }
void test_virtual() { Tracer t(2); Derived d; Base* b = &d; b->vwork(); }
void test_indirect() { Tracer t(3); void (*f)() = fn_throw; f(); }

int main() {
    try { test_member(); } catch (int e) { std::printf("E:%d ", e); }
    try { test_virtual(); } catch (int e) { std::printf("E:%d ", e); }
    try { test_indirect(); } catch (int e) { std::printf("E:%d ", e); }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 D:1 E:11 C:2 D:2 E:22 C:3 D:3 E:33");
    Ok(())
}

#[test]
fn test_sizeof_throwing_function_call_does_not_execute_or_unwind() -> Result<(), String> {
    let source = r#"
int throwing_func() {
    throw 42;
    return 1;
}
int query_size() {
    return sizeof(throwing_func());
}
int main() {
    return query_size() == sizeof(int) ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0);
    Ok(())
}
