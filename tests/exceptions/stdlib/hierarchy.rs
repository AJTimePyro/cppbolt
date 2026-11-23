use crate::common::run_cpp;

#[test]
fn test_std_exception_base_what() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>

int main() {
    try {
        throw std::exception();
    } catch (const std::exception& e) {
        std::printf("Caught:%s", e.what());
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Caught:std::exception");
    Ok(())
}

#[test]
fn test_std_bad_exception_polymorphic_catch() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>

void do_throw() {
    throw std::bad_exception();
}

int main() {
    try {
        do_throw();
    } catch (const std::exception& e) {
        std::printf("Polymorphic:%s", e.what());
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "Polymorphic:std::bad_exception");
    Ok(())
}

#[test]
fn test_std_exception_copy_and_slicing() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>

int main() {
    std::exception e1;
    std::exception e2 = e1;
    std::exception e3;
    e3 = e2;

    try {
        throw std::bad_exception();
    } catch (std::exception sliced) {
        // Slicing occurs when caught by value
        std::printf("Sliced:%s ", sliced.what());
    }

    try {
        throw std::bad_exception();
    } catch (const std::exception& ref) {
        // Virtual dispatch preserved when caught by reference
        std::printf("Ref:%s", ref.what());
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "Sliced:std::exception Ref:std::bad_exception"
    );
    Ok(())
}

#[test]
fn test_std_exception_derived_custom_virtual_dtor() -> Result<(), String> {
    let source = r#"
#include <exception>
#include <cstdio>

static int dtor_count = 0;

class MyCustomException : public std::exception {
public:
    MyCustomException() noexcept {}
    virtual ~MyCustomException() noexcept {
        dtor_count++;
    }
    virtual const char* what() const noexcept {
        return "custom exception message";
    }
};

void throw_custom() {
    throw MyCustomException();
}

int main() {
    try {
        throw_custom();
    } catch (const std::exception& e) {
        std::printf("CaughtCustom:%s ", e.what());
    }
    std::printf("DtorCount:%d", dtor_count);
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "CaughtCustom:custom exception message DtorCount:1"
    );
    Ok(())
}
