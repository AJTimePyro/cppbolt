use crate::common::run_cpp;

#[test]
fn test_throw_and_catch_pointer_payload() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    try {
        const char* msg = "hello exception";
        throw msg;
    } catch (const char* e) {
        std::printf("Caught:%s", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Caught:hello exception");
    Ok(())
}

#[test]
fn test_pointer_const_qualification_conversion() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Widget {
    int val;
    Widget(int v) : val(v) {}
};

Widget g_w(99);

int main() {
    // 1. const Widget* should NOT be caught by non-const Widget*
    try {
        const Widget* pw = &g_w;
        throw pw;
    } catch (Widget* w) {
        std::printf("ErrorShouldNotCatch ");
    } catch (const Widget* cw) {
        std::printf("CaughtConstWidget:%d ", cw->val);
    }

    // 2. non-const Widget* CAN be caught by const Widget*
    try {
        Widget* pw = &g_w;
        throw pw;
    } catch (const Widget* cw) {
        std::printf("CaughtNonConstByConst:%d ", cw->val);
    }

    // 3. const char* should NOT be caught by void*
    try {
        const char* msg = "test";
        throw msg;
    } catch (void* v) {
        std::printf("ErrorVoidPtr ");
    } catch (const void* cv) {
        std::printf("CaughtConstVoid");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "CaughtConstWidget:99 CaughtNonConstByConst:99 CaughtConstVoid"
    );
    Ok(())
}

#[test]
fn test_pointer_exception_conversions_and_null_pointer() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Base1 { int b1; Base1() : b1(100) {} };
struct Base2 { int b2; Base2() : b2(200) {} };
struct DerivedMI : Base1, Base2 { int d; DerivedMI() : d(300) {} };

int main() {
    int val = 42;
    int* p = &val;

    // 1. int* should NOT match double* (no type collision)
    try {
        throw p;
    } catch (double* pd) {
        std::printf("ErrorDouble ");
    } catch (int* pi) {
        std::printf("IntPtr:%d ", *pi);
    }

    // 2. Multiple inheritance pointer adjustment for Base2*
    DerivedMI obj;
    try {
        throw &obj;
    } catch (Base2* pb2) {
        std::printf("MI_Base2:%d ", pb2->b2);
    }

    // 3. Null pointer matching Base2* remains null (0)
    try {
        DerivedMI* null_d = nullptr;
        throw null_d;
    } catch (Base2* pb2) {
        std::printf("NullMI:%s ", pb2 == nullptr ? "null" : "non-null");
    }

    // 4. nullptr exception caught by pointer handler
    try {
        throw nullptr;
    } catch (int* pi) {
        std::printf("NullptrCaught:%s", pi == nullptr ? "null" : "non-null");
    }

    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "IntPtr:42 MI_Base2:200 NullMI:null NullptrCaught:null"
    );
    Ok(())
}

#[test]
fn test_pointer_indirection_depth_type_id_distinct() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    int val = 42;
    int* p1 = &val;
    int** p2 = &p1;
    int*** p3 = &p2;

    // int*** should NOT be caught by const int**
    try {
        throw p3;
    } catch (const int** p) {
        std::printf("CollisionError ");
    } catch (int*** p) {
        std::printf("CorrectIntTriplePtr:%d", ***p);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectIntTriplePtr:42");
    Ok(())
}

#[test]
fn test_multilevel_pointer_qualification_conversions() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    int val = 42;
    int* p1 = &val;
    int** p2 = &p1;

    // 1. int** caught by const int* const* (valid in C++17)
    try {
        throw p2;
    } catch (const int* const* p) {
        std::printf("CaughtConstIntPtrConstPtr:%d ", **p);
    } catch (...) {
        std::printf("FailedPCP ");
    }

    // 2. int** caught by int* const* (valid in C++17)
    try {
        throw p2;
    } catch (int* const* p) {
        std::printf("CaughtIntPtrConstPtr:%d ", **p);
    } catch (...) {
        std::printf("FailedIPCP ");
    }

    // 3. int** should NOT be caught by const int** (invalid in C++)
    try {
        throw p2;
    } catch (const int** p) {
        std::printf("InvalidMatch ");
    } catch (...) {
        std::printf("CorrectlyRejectedConstIntPtrPtr ");
    }

    // 4. const int** caught by const int* const* (valid in C++17)
    const int cval = 100;
    const int* cp1 = &cval;
    const int** cp2 = &cp1;
    try {
        throw cp2;
    } catch (const int* const* p) {
        std::printf("CaughtCP2ByConstIntPtrConstPtr:%d", **p);
    } catch (...) {
        std::printf("FailedCP2 ");
    }

    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "CaughtConstIntPtrConstPtr:42 CaughtIntPtrConstPtr:42 CorrectlyRejectedConstIntPtrPtr CaughtCP2ByConstIntPtrConstPtr:100"
    );
    Ok(())
}

#[test]
fn test_derived_to_base_pointer_qualification_and_adjustment() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Base1 {
    int b1;
    Base1(int x) : b1(x) {}
};

struct Base2 {
    int b2;
    Base2(int x) : b2(x) {}
};

struct Derived : Base1, Base2 {
    int d;
    Derived(int x, int y, int z) : Base1(x), Base2(y), d(z) {}
};

int main() {
    Derived obj(10, 20, 30);
    Derived* p_derived = &obj;

    // 1. Derived* caught by const Base2* (offset adjustment + qualification)
    try {
        throw p_derived;
    } catch (const Base2* pb2) {
        std::printf("CaughtConstBase2:%d ", pb2->b2);
    } catch (...) {
        std::printf("FailedConstBase2 ");
    }

    // 2. Derived* caught by Base2* (offset adjustment)
    try {
        throw p_derived;
    } catch (Base2* pb2) {
        std::printf("CaughtBase2:%d ", pb2->b2);
    } catch (...) {
        std::printf("FailedBase2 ");
    }

    // 3. const Derived* caught by const Base2*
    const Derived* cp_derived = &obj;
    try {
        throw cp_derived;
    } catch (const Base2* pb2) {
        std::printf("CaughtConstDByConstBase2:%d ", pb2->b2);
    } catch (...) {
        std::printf("FailedConstDByConstBase2 ");
    }

    // 4. const Derived* must NOT be caught by non-const Base2* (dropping const)
    try {
        throw cp_derived;
    } catch (Base2* pb2) {
        std::printf("ShouldNotCatchDroppingConst ");
    } catch (...) {
        std::printf("CorrectlyRejectedDroppingConst ");
    }

    // 5. Derived** must NOT be caught by const Base2* const* (multi-level inheritance conversion forbidden)
    Derived** pp_derived = &p_derived;
    try {
        throw pp_derived;
    } catch (const Base2* const*) {
        std::printf("ShouldNotCatchMultiLevelBase ");
    } catch (...) {
        std::printf("CorrectlyRejectedMultiLevelBase");
    }

    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "CaughtConstBase2:20 CaughtBase2:20 CaughtConstDByConstBase2:20 CorrectlyRejectedDroppingConst CorrectlyRejectedMultiLevelBase"
    );
    Ok(())
}

#[test]
fn test_catch_raw_function_pointer_syntax() -> Result<(), String> {
    let source = r#"
#include <cstdio>

void target() {}

int main() {
    try {
        throw target;
    } catch (void (*fp)()) {
        fp();
        std::printf("CaughtFuncPtr");
    } catch (...) {
        std::printf("Fallback");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtFuncPtr");
    Ok(())
}

#[test]
fn test_invalid_multilevel_pointer_qualification_rejected() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    int val = 42;
    int* p1 = &val;
    int** p2 = &p1;
    int*** p3 = &p2;

    try {
        throw p3;
    } catch (const int** const*) {
        std::printf("WrongInvalid ");
    } catch (const int* const* const*) {
        std::printf("CorrectValid:%d", ***p3);
    } catch (...) {
        std::printf("Fallback ");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectValid:42");
    Ok(())
}

#[test]
fn test_enum_exception_type_id_is_distinct() -> Result<(), String> {
    let source = r#"
#include <cstdio>

enum class Color { Red, Green };
enum class Fruit { Apple };

int main() {
    try {
        throw Color::Red;
    } catch (int) {
        std::printf("WrongInt ");
    } catch (Fruit) {
        std::printf("WrongFruit ");
    } catch (Color c) {
        std::printf("CorrectColor:%d", (int)c);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectColor:0");
    Ok(())
}

#[test]
fn test_function_pointer_exception_type_ids_are_distinct() -> Result<(), String> {
    let source = r#"
#include <cstdio>

using VA = void(*)();
using IA = int(*)(int);
void foo() {}

int main() {
    VA p = &foo;
    try {
        throw p;
    } catch (IA) {
        std::printf("WrongSignature ");
    } catch (VA) {
        std::printf("CorrectVA");
    } catch (...) {
        std::printf("Fallback");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectVA");
    Ok(())
}

#[test]
fn test_member_pointer_exception_type_id_is_distinct() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct S { int x; };
using MP = int S::*;
using FP = void(*)();
void foo() {}

int main() {
    MP p = &S::x;
    try {
        throw p;
    } catch (FP) {
        std::printf("WrongFP ");
    } catch (MP) {
        std::printf("CorrectMP");
    } catch (...) {
        std::printf("Fallback");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectMP");
    Ok(())
}

#[test]
fn test_nullptr_matches_member_pointer_handler() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct S { int x; };
using MP = int S::*;

int main() {
    try {
        throw nullptr;
    } catch (int) {
        std::printf("WrongInt ");
    } catch (MP) {
        std::printf("CorrectNullMP");
    } catch (...) {
        std::printf("Fallback");
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CorrectNullMP");
    Ok(())
}
