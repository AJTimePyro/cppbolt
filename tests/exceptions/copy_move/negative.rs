use crate::common::{PathBuf, RepoIncludes, assert_compile_error, compile_with_provider};

#[test]
fn test_negative_cannot_throw_abstract_class() -> Result<(), String> {
    let source = r#"
struct Abstract {
    virtual void pure() = 0;
};

int main() {
    Abstract a;
    throw a;
}
"#;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    match compile_with_provider(source, "<input>", 256, &includes) {
        Err(err) => {
            let err_str = err.to_string();
            if err_str.contains("abstract class") {
                Ok(())
            } else {
                Err(format!("unexpected error: {err_str}"))
            }
        }
        Ok(_) => Err("must fail compilation when throwing abstract class".to_string()),
    }
}

#[test]
fn test_negative_cannot_catch_abstract_class_by_value() -> Result<(), String> {
    let source = r#"
struct Abstract {
    virtual void pure() = 0;
};

int main() {
    try {
        throw 1;
    } catch (Abstract a) {
    }
}
"#;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    match compile_with_provider(source, "<input>", 256, &includes) {
        Err(err) => {
            let err_str = err.to_string();
            if err_str.contains("cannot catch abstract class") {
                Ok(())
            } else {
                Err(format!("unexpected error: {err_str}"))
            }
        }
        Ok(_) => Err("must fail compilation when catching abstract class by value".to_string()),
    }
}

#[test]
fn test_negative_cannot_catch_rvalue_reference() -> Result<(), String> {
    let source = r#"
int main() {
    try {
        throw 1;
    } catch (int&& e) {
    }
}
"#;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let includes = RepoIncludes {
        stdlib: root.join("stdlib"),
    };
    match compile_with_provider(source, "<input>", 256, &includes) {
        Err(err) => {
            let err_str = err.to_string();
            if err_str.contains("rvalue reference") {
                Ok(())
            } else {
                Err(format!("unexpected error: {err_str}"))
            }
        }
        Ok(_) => Err("must fail compilation when catching rvalue reference".to_string()),
    }
}

#[test]
fn test_negative_throw_deleted_copy_ctor() -> Result<(), String> {
    let source = r#"
struct NoCopy {
    NoCopy() {}
    NoCopy(const NoCopy&) = delete;
    NoCopy(NoCopy&&) = delete;
};
int main() {
    NoCopy a;
    try { throw a; } catch (...) {}
    return 0;
}
"#;
    assert_compile_error(source, "copy constructor")
}

#[test]
fn test_negative_catch_private_copy_ctor() -> Result<(), String> {
    let source = r#"
struct NoCopy {
    NoCopy() {}
private:
    NoCopy(const NoCopy&) {}
};
int main() {
    try { throw 1; } catch (NoCopy) {}
    return 0;
}
"#;
    assert_compile_error(source, "copy constructor")
}
