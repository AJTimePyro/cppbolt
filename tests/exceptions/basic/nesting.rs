use crate::common::run_cpp;

#[test]
fn test_nested_scopes_with_exceptions_at_different_depths() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

void run() {
    Tracer t1(1);
    {
        Tracer t2(2);
        {
            Tracer t3(3);
            throw 42;
        }
    }
}

int main() {
    try {
        run();
    } catch (int e) {
        std::printf("Caught:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 C:3 D:3 D:2 D:1 Caught:42");
    Ok(())
}

#[test]
fn test_catch_preserves_outer_local_objects() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int main() {
    Tracer outer(1);
    try {
        Tracer inner(2);
        throw 55;
    } catch (int e) {
        std::printf("Caught:%d ", e);
    }
    std::printf("After ");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 D:2 Caught:55 After D:1");
    Ok(())
}

#[test]
fn test_nested_try_blocks_and_handler_precedence() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int main() {
    try {
        Tracer t1(1);
        try {
            Tracer t2(2);
            throw 42;
        } catch (int e) {
            std::printf("Inner:%d ", e);
        }
        std::printf("Between ");
    } catch (...) {
        std::printf("Outer ");
    }
    std::printf("End");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:1 C:2 D:2 Inner:42 Between D:1 End");
    Ok(())
}

#[test]
fn test_nested_catch_depth_5_levels_and_rethrow() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    try {
        try {
            throw 10;
        } catch (...) {
            std::printf("L1 ");
            try {
                throw 20;
            } catch (...) {
                std::printf("L2 ");
                try {
                    throw 30;
                } catch (...) {
                    std::printf("L3 ");
                    try {
                        throw 40;
                    } catch (...) {
                        std::printf("L4 ");
                        try {
                            throw 50;
                        } catch (...) {
                            std::printf("L5 ");
                        }
                        std::printf("A4 ");
                    }
                    std::printf("A3 ");
                }
                std::printf("A2 ");
                throw; // Rethrows 20!
            }
        }
    } catch (int e) {
        std::printf("CaughtFinal:%d", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "L1 L2 L3 L4 L5 A4 A3 A2 CaughtFinal:20");
    Ok(())
}

#[test]
fn test_mixed_catch_int_and_catch_all_nesting_rethrow() -> Result<(), String> {
    let source = r#"
#include <cstdio>

int main() {
    try {
        try {
            throw 111;
        } catch (int e1) {
            std::printf("L1:%d ", e1);
            try {
                throw 222;
            } catch (...) {
                std::printf("L2 ");
                try {
                    throw 333;
                } catch (int e3) {
                    std::printf("L3:%d ", e3);
                    try {
                        throw 444;
                    } catch (...) {
                        std::printf("L4 ");
                        try {
                            throw 555;
                        } catch (int e5) {
                            std::printf("L5:%d ", e5);
                        }
                        std::printf("A4 ");
                    }
                    std::printf("A3 ");
                }
                std::printf("A2 ");
                throw; // Rethrows 222 (from catch-all L2)
            }
        }
    } catch (int final_e) {
        std::printf("CaughtFinal:%d", final_e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "L1:111 L2 L3:333 L4 L5:555 A4 A3 A2 CaughtFinal:222"
    );
    Ok(())
}
