use crate::common::{assert_compile_error, run_cpp};

#[test]
fn test_throw_function_parameter_requires_copy_constructor() -> Result<(), String> {
    let source = r#"
struct MoveOnly {
    int v;
    MoveOnly() : v(0) {}
    MoveOnly(const MoveOnly&) = delete;
    MoveOnly(MoveOnly&& o) : v(o.v) {}
};
void f(MoveOnly m) {
    throw m;
}
int main() { return 0; }
"#;
    assert_compile_error(source, "copy constructor")
}

#[test]
fn test_throw_function_parameter_calls_copy_constructor() -> Result<(), String> {
    let source = r#"
static int copies = 0;
static int moves = 0;
struct Tracked {
    Tracked() {}
    Tracked(const Tracked&) { copies++; }
    Tracked(Tracked&&) { moves++; }
};
void f(Tracked t) {
    throw t;
}
int main() {
    try {
        Tracked obj;
        f(obj);
    } catch (Tracked) {}
    return (copies >= 1 && moves == 0) ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0);
    Ok(())
}

#[test]
fn test_throw_catch_parameter_requires_copy_constructor() -> Result<(), String> {
    let source = r#"
struct MoveOnly {
    int v;
    MoveOnly() : v(0) {}
    MoveOnly(const MoveOnly&) = delete;
    MoveOnly(MoveOnly&& o) : v(o.v) {}
};
int main() {
    try {
    } catch (MoveOnly m) {
        throw m;
    }
    return 0;
}
"#;
    assert_compile_error(source, "copy constructor")
}

#[test]
fn test_throw_catch_parameter_calls_copy_constructor() -> Result<(), String> {
    let source = r#"
static int copies = 0;
static int moves = 0;
struct Tracked {
    Tracked() {}
    Tracked(const Tracked&) { copies++; }
    Tracked(Tracked&&) { moves++; }
};
int main() {
    try {
        try {
            const Tracked obj;
            throw obj;
        } catch (Tracked t) {
            moves = 0;
            throw t;
        }
    } catch (Tracked) {}
    return (moves == 0) ? 0 : 1;
}
"#;
    let (code, _) = run_cpp(source)?;
    assert_eq!(code, 0);
    Ok(())
}
