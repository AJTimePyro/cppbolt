use crate::common::run_cpp;

#[test]
fn test_regular_function_try_block() -> Result<(), String> {
    let source = r#"
#include <cstdio>

void f() try {
    std::printf("Body ");
    throw 7;
} catch (int e) {
    std::printf("Caught:%d ", e);
}

int main() {
    f();
    std::printf("Done");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Body Caught:7 Done");
    Ok(())
}
