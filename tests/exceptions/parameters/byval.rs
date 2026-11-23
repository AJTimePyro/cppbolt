use crate::common::run_cpp;

#[test]
fn test_byval_argument_preserves_subexpression_temporaries() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct SubTemp {
    int id;
    SubTemp(int i) : id(i) { std::printf("C_Sub:%d ", id); }
    ~SubTemp() { std::printf("D_Sub:%d ", id); }
};

struct ArgWidget {
    int id;
    ArgWidget(int i) : id(i) { std::printf("C_Wid:%d ", id); }
    ~ArgWidget() { std::printf("D_Wid:%d ", id); }
};

ArgWidget make_widget(const SubTemp& s) {
    std::printf("MakeWid ");
    return ArgWidget(s.id);
}

void consume_normal(ArgWidget w) {
    std::printf("ConsumeNormal ");
}

void consume_throw(ArgWidget w) {
    std::printf("ConsumeThrow ");
    throw 456;
}

int main() {
    std::printf("---Normal--- ");
    consume_normal(make_widget(SubTemp(10)));
    std::printf("---AfterNormal--- ");
    try {
        consume_throw(make_widget(SubTemp(20)));
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
        "---Normal--- C_Sub:10 MakeWid C_Wid:10 ConsumeNormal D_Wid:10 D_Sub:10 ---AfterNormal--- C_Sub:20 MakeWid C_Wid:20 ConsumeThrow D_Wid:20 D_Sub:20 Caught:456"
    );
    Ok(())
}

#[test]
fn test_byval_parameter_construction_failure_cleans_earlier_parameters() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct ParamA {
    int id;
    ParamA(int i) : id(i) { std::printf("C_A:%d ", id); }
    ~ParamA() { std::printf("D_A:%d ", id); }
};

struct ParamB {
    int id;
    ParamB(int i) : id(i) {
        std::printf("C_B:%d ", id);
        if (id == 99) {
            std::printf("Throw_B ");
            throw 777;
        }
    }
    ~ParamB() { std::printf("D_B:%d ", id); }
};

void target(ParamA a, ParamB b) {
    std::printf("InTarget ");
}

int main() {
    try {
        target(ParamA(1), ParamB(99));
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C_A:1 C_B:99 Throw_B D_A:1 Caught:777");
    Ok(())
}

#[test]
fn test_byval_parameter_callee_throw_cleans_parameters() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Param {
    int id;
    Param(int i) : id(i) { std::printf("C:%d ", id); }
    Param(const Param& o) : id(o.id) { std::printf("Copy:%d ", id); }
    ~Param() { std::printf("D:%d ", id); }
};

void target(Param a, Param b) {
    std::printf("InTarget ");
    throw 888;
}

int main() {
    Param p1(1);
    Param p2(2);
    try {
        target(p1, p2);
    } catch (int e) {
        std::printf("Caught:%d ", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "C:1 C:2 Copy:1 Copy:2 InTarget D:2 D:1 Caught:888 D:2 D:1"
    );
    Ok(())
}
