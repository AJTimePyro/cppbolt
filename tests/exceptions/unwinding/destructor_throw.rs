use crate::common::run_cpp;

#[test]
fn test_destructor_throwing_during_unwinding_terminates() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct BadTracer {
    int id;
    BadTracer(int i) : id(i) {}
    ~BadTracer() {
        std::printf("BadDtor ");
        throw 999;
    }
};

void inner() {
    throw 42;
}

void middle() {
    BadTracer b(1);
    inner();
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
    assert_eq!(
        code, 134,
        "program must trap with abort/134 on throwing destructor during unwinding"
    );
    assert!(stdout.contains("BadDtor"), "BadDtor must have been entered");
    assert!(
        !stdout.contains("Caught"),
        "handler must NOT catch or replace active exception"
    );
    Ok(())
}

#[test]
fn test_destructor_throwing_during_normal_exit_propagates() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Normal {
    int id;
    Normal(int i) : id(i) { std::printf("C_Normal:%d ", id); }
    ~Normal() { std::printf("D_Normal:%d ", id); }
};

struct ThrowingDtor {
    int val;
    ThrowingDtor(int v) : val(v) {}
    ~ThrowingDtor() noexcept(false) {
        std::printf("D_Throw:%d ", val);
        throw val;
    }
};

void test_fn() {
    Normal n(1);
    ThrowingDtor t(99);
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
    assert_eq!(stdout.trim(), "C_Normal:1 D_Throw:99 D_Normal:1 Caught:99");
    Ok(())
}

#[test]
fn test_destructor_throwing_while_unwinding_from_normal_destructor_throw_terminates()
-> Result<(), String> {
    let source = r#"
#include <cstdio>

struct ThrowingDtor {
    int val;
    ThrowingDtor(int v) : val(v) {}
    ~ThrowingDtor() noexcept(false) {
        std::printf("D_Throw:%d ", val);
        throw val;
    }
};

void test_fn() {
    ThrowingDtor first(1);
    ThrowingDtor second(2);
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
    assert_eq!(
        code, 134,
        "destructor throwing during unwinding from normal exit throw must terminate with abort/134"
    );
    assert!(stdout.contains("D_Throw:2"), "second dtor must have thrown");
    assert!(
        stdout.contains("D_Throw:1"),
        "first dtor must have been entered during unwinding and thrown"
    );
    assert!(
        !stdout.contains("Caught"),
        "active exception must terminate, not be caught"
    );
    Ok(())
}

#[test]
fn test_throwing_during_unwinding_invokes_custom_terminate_handler() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>
#include <cstdlib>

void my_terminate() {
    std::printf("CustomTerminateCalled\n");
    std::exit(42);
}

struct Bad {
    ~Bad() noexcept(false) { throw 9; }
};

void inner() { throw 1; }
void middle() { Bad b; inner(); }

int main() {
    std::set_terminate(my_terminate);
    try {
        middle();
    } catch (int) {
        std::printf("Caught");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(
        code, 42,
        "a throwing destructor during unwinding must invoke the custom terminate handler"
    );
    assert_eq!(stdout.trim(), "CustomTerminateCalled");
    Ok(())
}
