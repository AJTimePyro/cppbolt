use crate::common::run_cpp;

#[test]
fn test_dce_eliminates_unreferenced_exception_class_destructors() -> Result<(), String> {
    let source = r#"
#include <cstdio>

// An undefined function declaration. If DeadException's destructor is kept by DCE,
// compilation will fail trying to resolve this undefined function.
void undefined_function_that_does_not_exist();

struct DeadException {
    ~DeadException() {
        undefined_function_that_does_not_exist();
    }
};

struct LiveException {
    int code;
    LiveException(int c) : code(c) {}
    ~LiveException() {
        std::printf("LiveExceptionDtor:%d ", code);
    }
};

int main() {
    try {
        throw LiveException(77);
    } catch (LiveException e) {
        std::printf("CaughtLive:%d ", e.code);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "CaughtLive:77 LiveExceptionDtor:77 LiveExceptionDtor:77"
    );
    Ok(())
}
