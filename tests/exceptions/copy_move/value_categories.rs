use crate::common::run_cpp;

#[test]
fn test_exception_object_move_slicing_and_unnamed_catch() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct MoveTracer {
    int id;
    MoveTracer(int i) : id(i) { std::printf("C:%d ", id); }
    MoveTracer(const MoveTracer& o) : id(o.id) { std::printf("Copy:%d ", id); }
    MoveTracer(MoveTracer&& o) : id(o.id) { std::printf("Move:%d ", id); }
    ~MoveTracer() { std::printf("D:%d ", id); }
};

struct SBase {
    int b;
    SBase(int v) : b(v) { std::printf("SBaseC:%d ", b); }
    SBase(const SBase& o) : b(o.b) { std::printf("SBaseCopy:%d ", b); }
    virtual ~SBase() { std::printf("SBaseD:%d ", b); }
};
struct SDerived : SBase {
    int d;
    SDerived(int bv, int dv) : SBase(bv), d(dv) { std::printf("SDerivedC:%d ", d); }
    SDerived(const SDerived& o) : SBase(o), d(o.d) { std::printf("SDerivedCopy:%d ", d); }
    ~SDerived() override { std::printf("SDerivedD:%d ", d); }
};

int main() {
    // 1. Move constructor preferred when throwing local named variable
    try {
        MoveTracer m(42);
        throw m;
    } catch (MoveTracer& t) {
        std::printf("CaughtMove:%d ", t.id);
    }

    // 2. Unnamed catch-by-value constructs and destroys temporary parameter
    try {
        throw MoveTracer(99);
    } catch (MoveTracer) {
        std::printf("InCatchUnnamed ");
    }

    // 3. Catch derived by value as base slices and runs both destructors
    try {
        SDerived obj(10, 20);
        throw obj;
    } catch (SBase b) {
        std::printf("Sliced:%d ", b.b);
    }

    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "C:42 Move:42 D:42 CaughtMove:42 D:42 C:99 Copy:99 InCatchUnnamed D:99 D:99 SBaseC:10 SDerivedC:20 SBaseCopy:10 SDerivedCopy:20 SDerivedD:20 SBaseD:10 SBaseCopy:10 Sliced:10 SBaseD:10 SDerivedD:20 SBaseD:10"
    );
    Ok(())
}

#[test]
fn test_throw_with_private_default_ctor_and_public_copy_ctor() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct MyException {
private:
    MyException() {}
public:
    MyException(int v) : value(v) {}
    MyException(const MyException& o) : value(o.value) {}
    int value;
};

int main() {
    try {
        throw MyException(7);
    } catch (MyException e) {
        std::printf("Caught:%d", e.value);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Caught:7");
    Ok(())
}

#[test]
fn test_throw_with_deleted_default_ctor_and_public_copy_ctor() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct MyException {
    MyException() = delete;
    MyException(int v) : value(v) {}
    MyException(const MyException& o) : value(o.value) {}
    int value;
};

int main() {
    try {
        throw MyException(7);
    } catch (MyException e) {
        std::printf("Caught:%d", e.value);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Caught:7");
    Ok(())
}

#[test]
fn test_throw_private_move_ctor_lvalue_copies_successfully() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct PrivMove {
    int v;
    PrivMove(int x) : v(x) {}
    PrivMove(const PrivMove& o) : v(o.v) { std::printf("Copy "); }
private:
    PrivMove(PrivMove&&);
};

PrivMove global_obj(123);

void f(PrivMove& ref) {
    throw ref;
}

int main() {
    try {
        f(global_obj);
    } catch (const PrivMove& e) {
        std::printf("Caught:%d", e.v);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Copy Caught:123");
    Ok(())
}

#[test]
fn test_throw_deleted_copy_ctor_prvalue_and_move_eligible_local() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct MoveOnly {
    int v;
    MoveOnly(int x) : v(x) {}
    MoveOnly(const MoveOnly&) = delete;
    MoveOnly(MoveOnly&& o) : v(o.v) { std::printf("Move "); }
};

void throw_prvalue() {
    throw MoveOnly(456);
}

void throw_local() {
    try {
        MoveOnly m(789);
        throw m;
    } catch (MoveOnly& caught) {
        std::printf("CaughtLocal:%d ", caught.v);
    }
}

int main() {
    try {
        throw_prvalue();
    } catch (MoveOnly& caught) {
        std::printf("CaughtPrvalue:%d ", caught.v);
    }
    throw_local();
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtPrvalue:456 Move CaughtLocal:789");
    Ok(())
}

#[test]
fn test_throw_deleted_move_ctor_lvalue_copies_successfully() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct DelMove {
    int v;
    DelMove(int x) : v(x) {}
    DelMove(const DelMove& o) : v(o.v) { std::printf("Copy "); }
    DelMove(DelMove&&) = delete;
};

DelMove global_m(321);

void f(DelMove& ref) {
    throw ref;
}

int main() {
    try {
        f(global_m);
    } catch (const DelMove& e) {
        std::printf("Caught:%d", e.v);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Copy Caught:321");
    Ok(())
}

#[test]
fn test_outer_local_variable_throw_copies_instead_of_moving() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct S {
    int v;
    S(int x) : v(x) {}
    S(const S& o) : v(o.v) { std::printf("Copy "); }
    S(S&& o) : v(o.v) { std::printf("Move "); }
    ~S() {}
};

int main() {
    S s(1);
    try {
        throw s;
    } catch (...) {
        std::printf("Caught ");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert!(
        stdout.contains("Copy"),
        "an outer local must be copied on throw: {stdout}"
    );
    assert!(
        !stdout.contains("Move"),
        "an outer local must not be moved on throw: {stdout}"
    );
    Ok(())
}

#[test]
fn test_throw_const_std_move_falls_back_to_copy_ctor() -> Result<(), String> {
    let source = r#"
#include <cstdio>
#include <utility>
struct T { T(){} T(const T&){ std::printf("Copy "); } T(T&&) = delete; };
int main(){ const T e; try { throw std::move(e); } catch(const T&) {} std::printf("Done\n"); return 0; }
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Copy Done");
    Ok(())
}

#[test]
fn test_throw_lvalue_and_xvalue_ignore_explicit_move_ctor() -> Result<(), String> {
    let source = r#"
#include <cstdio>
struct T { T(){} T(const T&){ std::printf("Copy "); } explicit T(T&&){ std::printf("Move "); } };
int main(){
    T e;
    try { throw e; } catch(const T&) {}
    try { throw static_cast<T&&>(e); } catch(const T&) {}
    std::printf("Done\n");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Copy Copy Done");
    Ok(())
}
