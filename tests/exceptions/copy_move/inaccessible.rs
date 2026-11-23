use crate::common::{assert_compile_error, run_cpp};

#[test]
fn test_throw_private_move_ctor_local_variable_fails() -> Result<(), String> {
    let source = r#"
struct PrivMove {
    int v;
    PrivMove(int x) : v(x) {}
    PrivMove(const PrivMove& o) : v(o.v) {}
private:
    PrivMove(PrivMove&&);
};

void f() {
    PrivMove local_obj(123);
    throw local_obj;
}

int main() { return 0; }
"#;
    assert_compile_error(
        source,
        "move constructor for exception object of type 'PrivMove' is inaccessible",
    )
}

#[test]
fn test_throw_deleted_copy_ctor_non_move_eligible_lvalue_fails() -> Result<(), String> {
    let source = r#"
struct MoveOnly {
    int v;
    MoveOnly(int x) : v(x) {}
    MoveOnly(const MoveOnly&) = delete;
    MoveOnly(MoveOnly&& o) : v(o.v) {}
};

MoveOnly global_m(1);

void f(MoveOnly& ref) {
    throw ref;
}

int main() { return 0; }
"#;
    assert_compile_error(
        source,
        "copy constructor for exception object of type 'MoveOnly' is deleted",
    )
}

#[test]
fn test_throw_deleted_move_ctor_local_variable_fails() -> Result<(), String> {
    let source = r#"
struct DelMove {
    int v;
    DelMove(int x) : v(x) {}
    DelMove(const DelMove& o) : v(o.v) {}
    DelMove(DelMove&&) = delete;
};

void f() {
    DelMove m(1);
    throw m;
}

int main() { return 0; }
"#;
    assert_compile_error(
        source,
        "move constructor for exception object of type 'DelMove' is deleted",
    )
}

#[test]
fn test_explicit_copy_constructor_rejected_on_throw_and_catch_by_value() -> Result<(), String> {
    let source_throw = r#"
struct ExpCopy {
    int v;
    ExpCopy(int x) : v(x) {}
    explicit ExpCopy(const ExpCopy& o) : v(o.v) {}
};

ExpCopy global_e(1);

void f() {
    throw global_e;
}

int main() { return 0; }
"#;
    let _ = assert_compile_error(source_throw, "explicit copy constructor");

    let source_catch = r#"
struct ExpCopy {
    int v;
    ExpCopy(int x) : v(x) {}
    explicit ExpCopy(const ExpCopy& o) : v(o.v) {}
};

int main() {
    try {
    } catch (ExpCopy e) {
    }
    return 0;
}
"#;
    let _ = assert_compile_error(source_catch, "copy constructor");

    let source_catch_ref = r#"
#include <cstdio>

struct ExpCopy {
    int v;
    ExpCopy(int x) : v(x) {}
    explicit ExpCopy(const ExpCopy& o) : v(o.v) {}
};

int main() {
    try {
        throw ExpCopy(99);
    } catch (const ExpCopy& e) {
        std::printf("CaughtRef:%d", e.v);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source_catch_ref)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtRef:99");
    Ok(())
}

#[test]
fn test_throw_std_move_with_deleted_move_ctor_fails() -> Result<(), String> {
    let source = r#"
#include <utility>
struct T { T(); T(const T&); T(T&&) = delete; };
int main(){ T e; throw std::move(e); }
"#;
    assert_compile_error(
        source,
        "move constructor for exception object of type 'T' is deleted",
    )
}

#[test]
fn test_throw_std_move_with_private_move_ctor_fails() -> Result<(), String> {
    let source = r#"
#include <utility>
struct T { T(); T(const T&); private: T(T&&); };
int main(){ T e; throw std::move(e); }
"#;
    assert_compile_error(
        source,
        "move constructor for exception object of type 'T' is inaccessible",
    )
}

#[test]
fn test_throw_lvalue_of_move_only_class_fails() -> Result<(), String> {
    let source = r#"
struct T { T(); T(T&&); };
void f(T& ref) { throw ref; }
int main(){ return 0; }
"#;
    assert_compile_error(
        source,
        "copy constructor for exception object of type 'T' is deleted or inaccessible",
    )
}

#[test]
fn test_throw_const_lvalue_with_nonconst_copy_ctor_fails() -> Result<(), String> {
    let source = r#"
struct T { T(); T(T&); };
const T g;
int main(){ throw g; }
"#;
    assert_compile_error(
        source,
        "copy constructor for exception object of type 'T' is deleted or inaccessible",
    )
}

#[test]
fn test_throw_std_move_with_nonconst_copy_ctor_fails() -> Result<(), String> {
    let source = r#"
#include <utility>
struct T { T(); T(T&); };
int main(){ T e; throw std::move(e); }
"#;
    assert_compile_error(
        source,
        "copy constructor for exception object of type 'T' is deleted or inaccessible",
    )
}
