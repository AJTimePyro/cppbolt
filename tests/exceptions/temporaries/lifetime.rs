use crate::common::run_cpp;

#[test]
fn test_reference_binding_temporary_lifetime_extension() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Temp {
    int id;
    Temp(int i) : id(i) { std::printf("C_Temp:%d ", id); }
    ~Temp() { std::printf("D_Temp:%d ", id); }
};

void test_ref_normal() {
    std::printf("Start ");
    const Temp& r = Temp(50);
    std::printf("RefBound ");
    std::printf("EndFn ");
}

void test_ref_throw() {
    const Temp& r = Temp(60);
    std::printf("RefActive ");
    throw 99;
}

int main() {
    test_ref_normal();
    std::printf("AfterNormal ");
    try {
        test_ref_throw();
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
        "Start C_Temp:50 RefBound EndFn D_Temp:50 AfterNormal C_Temp:60 RefActive D_Temp:60 Caught:99"
    );
    Ok(())
}

#[test]
fn test_reference_binding_ternary_lifetime_extension() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Widget {
    int id;
    Widget(int i) : id(i) { std::printf("C_W:%d ", id); }
    ~Widget() { std::printf("D_W:%d ", id); }
};

int main() {
    std::printf("Start ");
    {
        // When cond is true, Widget(1) is constructed, Widget(2) is not.
        // Widget(1) must stay alive until end of scope.
        const Widget& w1 = 1 ? Widget(1) : Widget(2);
        std::printf("Scope1Active ");
    }
    std::printf("Scope1End ");
    {
        // When cond is false, Widget(4) is constructed, Widget(3) is not.
        // Widget(4) must stay alive until end of scope.
        const Widget& w2 = 0 ? Widget(3) : Widget(4);
        std::printf("Scope2Active ");
    }
    std::printf("Scope2End");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "Start C_W:1 Scope1Active D_W:1 Scope1End C_W:4 Scope2Active D_W:4 Scope2End"
    );
    Ok(())
}
