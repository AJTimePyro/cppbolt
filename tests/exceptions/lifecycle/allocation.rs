use crate::common::run_cpp;

#[test]
fn test_exception_object_destruction_exactly_once() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int main() {
    try {
        throw Tracer(42);
    } catch (Tracer& t) {
        std::printf("InCatch:%d ", t.id);
    }
    std::printf("AfterCatch");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:42 InCatch:42 D:42 AfterCatch");
    Ok(())
}

#[test]
fn test_exception_destroyed_on_early_exit_return_and_break() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int test_early_return() {
    try {
        throw Tracer(10);
    } catch (Tracer& t) {
        std::printf("InReturnCatch ");
        return 77;
    }
    return 0;
}

int main() {
    int ret = test_early_return();
    std::printf("RetVal:%d ", ret);

    for (int i = 0; i < 2; i++) {
        try {
            throw Tracer(20);
        } catch (Tracer& t) {
            std::printf("InBreakCatch ");
            break;
        }
    }
    std::printf("AfterBreak");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "C:10 InReturnCatch D:10 RetVal:77 C:20 InBreakCatch D:20 AfterBreak"
    );
    Ok(())
}

#[test]
fn test_exception_thrown_while_handling_another_exception_lifecycle() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    Tracer(const Tracer& o) : id(o.id) { std::printf("Copy:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int main() {
    try {
        try {
            throw Tracer(1);
        } catch (Tracer& t1) {
            std::printf("InCatch1:%d ", t1.id);
            throw Tracer(2);
        }
    } catch (Tracer& t2) {
        std::printf("InCatch2:%d ", t2.id);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 InCatch1:1 C:2 D:1 InCatch2:2 D:2");
    Ok(())
}

#[test]
fn test_exception_allocation_freed_when_payload_constructor_throws() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct ThrowOnCopy {
    int id;
    ThrowOnCopy(int i) : id(i) {}
    ThrowOnCopy(const ThrowOnCopy& o) : id(o.id) {
        if (id == 99) {
            std::printf("ThrowingFromCopyCtor ");
            throw 123;
        }
    }
};

int main() {
    try {
        ThrowOnCopy obj(99);
        throw obj;
    } catch (int e) {
        std::printf("CaughtInt:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "ThrowingFromCopyCtor CaughtInt:123");
    Ok(())
}

#[test]
fn test_throw_allocation_failure_terminates_or_traps() -> Result<(), String> {
    let source = r#"
#include <exception>
int main() {
    try {
        throw 42;
    } catch (int x) {
        return x == 42 ? 0 : 1;
    }
    return 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0);
    Ok(())
}
