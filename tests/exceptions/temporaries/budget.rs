use crate::common::{PathBuf, RepoIncludes, assert_compile_error, compile_with_provider, run_cpp};

#[test]
fn test_exceeding_max_cleanup_objects_errors() -> Result<(), String> {
    let mut source = String::from("struct S { ~S() {} }; int main() {\n");
    for i in 0..65 {
        source.push_str(&format!("    S s{};\n", i));
    }
    source.push_str("    return 0;\n}\n");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    match compile_with_provider(&source, "<input>", 256, &includes) {
        Err(err) => {
            let err_str = err.to_string();
            if err_str.contains("maximum of 64 cleanup objects") {
                Ok(())
            } else {
                Err(format!("unexpected error: {err_str}"))
            }
        }
        Ok(_) => Err("must fail compilation when exceeding 64 cleanup objects".to_string()),
    }
}

#[test]
fn test_temp_cleanup_overflow_fails_compilation() -> Result<(), String> {
    let mut source = String::from("struct S { ~S() {} }; void sink(S s) {} int main() {\n");
    for i in 0..64 {
        source.push_str(&format!("    S s{i};\n"));
    }
    source.push_str("    sink(S());\n    return 0;\n}\n");
    assert_compile_error(&source, "maximum of 64 cleanup objects")
}

#[test]
fn test_more_than_64_sequential_temporaries_compile_and_run() -> Result<(), String> {
    let mut source = String::from(
        "struct S { int v; S(int x) : v(x) {} ~S() {} }; int sum = 0; void sink(S s) { sum += s.v; } int main() {\n",
    );
    for i in 0..100 {
        source.push_str(&format!("    sink(S({i}));\n"));
    }
    source.push_str("    return sum == 4950 ? 0 : 1;\n}\n");
    let (code, _) = run_cpp(&source)?;
    assert_eq!(code, 0);
    Ok(())
}

#[test]
fn test_recycled_temporary_slot_zeroed_after_destruction() -> Result<(), String> {
    let source = r#"
#include <cstdio>

struct Item {
    int val;
    ~Item() {
        std::printf("~Item(%d) ", val);
    }
};

Item make_item(int x) {
    return Item{x};
}

int main() {
    make_item(10);
    make_item(20);
    std::printf("Done");
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "~Item(10) ~Item(20) Done");
    Ok(())
}
