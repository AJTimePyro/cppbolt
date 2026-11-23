use crate::common::run_cpp;

#[test]
fn test_objects_constructed_inside_loops() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

void loop_run() {
    Tracer outer(100);
    for (int i = 0; i < 3; ++i) {
        Tracer t(i);
        if (i == 2) throw 77;
    }
}

int main() {
    try {
        loop_run();
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
        "C:100 C:0 D:0 C:1 D:1 C:2 D:2 D:100 Caught:77"
    );
    Ok(())
}

#[test]
fn test_break_and_continue_unwinds_exact_scopes() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int main() {
    for (int i = 0; i < 3; i++) {
        Tracer t(i);
        if (i == 0) {
            continue;
        }
        if (i == 1) {
            break;
        }
    }
    std::printf("AfterLoop");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "C:0 D:0 C:1 D:1 AfterLoop");
    Ok(())
}

#[test]
fn test_exception_caught_inside_loop_preserves_loop_variables() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Tracer {
    int id;
    Tracer(int i) : id(i) { std::printf("C:%d ", id); }
    ~Tracer() { std::printf("D:%d ", id); }
};

int main() {
    for (int i = 0; i < 2; i++) {
        Tracer outer(i + 10);
        try {
            Tracer inner(i + 20);
            throw i;
        } catch (int e) {
            std::printf("Caught:%d ", e);
        }
        std::printf("LoopEnd:%d ", i);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "C:10 C:20 D:20 Caught:0 LoopEnd:0 D:10 C:11 C:21 D:21 Caught:1 LoopEnd:1 D:11"
    );
    Ok(())
}

#[test]
fn test_loop_break_and_continue_from_catch_clears_slot() -> Result<(), String> {
    let source = r#"
static int dtor_count = 0;
struct Tracker {
    int id;
    Tracker(int i) : id(i) {}
    ~Tracker() { dtor_count++; }
};
int main() {
    for (int i = 0; i < 3; i++) {
        try {
            throw Tracker(i);
        } catch (Tracker& t) {
            if (t.id == 0) continue;
            break;
        }
    }
    return dtor_count == 2 ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0);
    Ok(())
}
