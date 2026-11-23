use crate::common::run_cpp;

#[test]
fn test_stdexcept_hierarchy_and_what() -> Result<(), String> {
    let source = r#"
#include <stdexcept>
#include <cstdio>
#include <string>

void test_classes() {
    // 1. logic_error
    try {
        throw std::logic_error("logic error msg");
    } catch (const std::logic_error& e) {
        std::printf("1:%s ", e.what());
    }

    // 2. domain_error
    try {
        throw std::domain_error("domain error msg");
    } catch (const std::domain_error& e) {
        std::printf("2:%s ", e.what());
    }

    // 3. invalid_argument
    try {
        throw std::invalid_argument("invalid argument msg");
    } catch (const std::invalid_argument& e) {
        std::printf("3:%s ", e.what());
    }

    // 4. length_error
    try {
        throw std::length_error("length error msg");
    } catch (const std::length_error& e) {
        std::printf("4:%s ", e.what());
    }

    // 5. out_of_range
    try {
        throw std::out_of_range("out of range msg");
    } catch (const std::out_of_range& e) {
        std::printf("5:%s ", e.what());
    }

    // 6. runtime_error
    try {
        throw std::runtime_error("runtime error msg");
    } catch (const std::runtime_error& e) {
        std::printf("6:%s ", e.what());
    }

    // 7. range_error
    try {
        throw std::range_error("range error msg");
    } catch (const std::range_error& e) {
        std::printf("7:%s ", e.what());
    }

    // 8. overflow_error
    try {
        throw std::overflow_error("overflow error msg");
    } catch (const std::overflow_error& e) {
        std::printf("8:%s ", e.what());
    }

    // 9. underflow_error
    try {
        throw std::underflow_error("underflow error msg");
    } catch (const std::underflow_error& e) {
        std::printf("9:%s ", e.what());
    }
}

int main() {
    test_classes();
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "1:logic error msg 2:domain error msg 3:invalid argument msg 4:length error msg 5:out of range msg 6:runtime error msg 7:range error msg 8:overflow error msg 9:underflow error msg"
    );
    Ok(())
}

#[test]
fn test_stdexcept_polymorphic_catch_base_classes() -> Result<(), String> {
    let source = r#"
#include <stdexcept>
#include <cstdio>
#include <string>

int main() {
    // Catch derived logic_error through std::exception
    try {
        throw std::out_of_range("out of range through std::exception");
    } catch (const std::exception& e) {
        std::printf("ExCatch:%s ", e.what());
    }

    // Catch derived logic_error through std::logic_error
    try {
        throw std::invalid_argument("invalid argument through std::logic_error");
    } catch (const std::logic_error& e) {
        std::printf("LogicCatch:%s ", e.what());
    }

    // Catch derived runtime_error through std::exception
    try {
        throw std::overflow_error("overflow error through std::exception");
    } catch (const std::exception& e) {
        std::printf("ExCatch2:%s ", e.what());
    }

    // Catch derived runtime_error through std::runtime_error
    try {
        throw std::underflow_error("underflow error through std::runtime_error");
    } catch (const std::runtime_error& e) {
        std::printf("RuntimeCatch:%s", e.what());
    }

    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "ExCatch:out of range through std::exception LogicCatch:invalid argument through std::logic_error ExCatch2:overflow error through std::exception RuntimeCatch:underflow error through std::runtime_error"
    );
    Ok(())
}

#[test]
fn test_stdexcept_string_constructor_and_copy() -> Result<(), String> {
    let source = r#"
#include <stdexcept>
#include <cstdio>
#include <string>

int main() {
    std::string base_msg = "constructed from std::string";
    try {
        std::runtime_error original(base_msg);
        std::runtime_error copy = original;
        throw copy;
    } catch (const std::runtime_error& caught) {
        std::printf("CaughtCopy:%s", caught.what());
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "CaughtCopy:constructed from std::string");
    Ok(())
}

#[test]
fn test_stdexcept_realistic_scenarios() -> Result<(), String> {
    let source = r#"
#include <stdexcept>
#include <cstdio>
#include <string>

int safe_divide(int a, int b) {
    if (b == 0) {
        throw std::invalid_argument("division by zero");
    }
    return a / b;
}

int get_element(int* arr, int size, int index) {
    if (index < 0 || index >= size) {
        throw std::out_of_range("array index out of bounds");
    }
    return arr[index];
}

void process_transaction(int amount) {
    if (amount > 1000000) {
        throw std::runtime_error("transaction limit exceeded");
    }
}

int main() {
    try {
        safe_divide(10, 0);
    } catch (const std::invalid_argument& e) {
        std::printf("CaughtDivZero:%s ", e.what());
    }

    int nums[3] = {10, 20, 30};
    try {
        get_element(nums, 3, 5);
    } catch (const std::out_of_range& e) {
        std::printf("CaughtBounds:%s ", e.what());
    }

    try {
        process_transaction(2000000);
    } catch (const std::runtime_error& e) {
        std::printf("CaughtTx:%s", e.what());
    }

    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "CaughtDivZero:division by zero CaughtBounds:array index out of bounds CaughtTx:transaction limit exceeded"
    );
    Ok(())
}

#[test]
fn test_stdexcept_nested_throw_in_catch() -> Result<(), String> {
    let source = r#"
#include <stdexcept>
#include <cstdio>

int main() {
    try {
        try {
            throw std::runtime_error("first runtime error");
        } catch (const std::exception& e1) {
            std::printf("FirstCaught:%s ", e1.what());
            throw std::logic_error("second logic error");
        }
    } catch (const std::exception& e2) {
        std::printf("SecondCaught:%s", e2.what());
    }
    return 0;
}
"#;
    let (code, stdout) = run_cpp(source)?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout.trim(),
        "FirstCaught:first runtime error SecondCaught:second logic error"
    );
    Ok(())
}
