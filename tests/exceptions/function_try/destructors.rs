use crate::common::run_cpp;

#[test]
fn test_destructor_function_try_block_rethrows_on_fallthrough() -> Result<(), String> {
    let source = r#"
#include <cstdio>

void throwing_fn() { throw 99; }

struct DtorTry {
    ~DtorTry() noexcept(false) try {
        throwing_fn();
    } catch (int e) {
        std::printf("Handled:%d ", e);
    }
};

int main() {
    try {
        DtorTry d;
    } catch (int e) {
        std::printf("OuterCatch:%d ", e);
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Handled:99 OuterCatch:99");
    Ok(())
}
