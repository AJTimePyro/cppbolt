//! Native stack-safety tests for the per-construct nesting limits.
//!
//! Wide left-associative chains must compile; deeply nested constructs must
//! produce a normal diagnostic instead of aborting.  Every nesting construct
//! has its own limit (see `cppwasm::nesting_limits`), and the tests check
//! the last accepted depth and the first rejected one.  Programs run on an
//! explicitly sized thread so unoptimized builds have room for the parser,
//! semantic-analysis, and code-generation frames.

use cppwasm::nesting_limits;
use std::thread;

fn compile_on_big_stack(source: String) -> Result<Vec<u8>, String> {
    let handle = thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || cppwasm::compile(&source).map_err(|error| error.message))
        .map_err(|error| format!("cannot start test thread: {error}"))?;
    handle
        .join()
        .map_err(|_| "compiler thread panicked".to_string())?
}

fn binary_chain(terms: usize) -> String {
    let mut source = String::from("#include <cstdio>\nint main() { int a = 1; int s = ");
    for index in 0..terms {
        if index > 0 {
            source.push('+');
        }
        source.push('a');
    }
    source.push_str("; printf(\"%d\\n\", s); return 0; }\n");
    source
}

fn string_concat(terms: usize) -> String {
    let mut source = String::from("#include <string>\nint main() { std::string s; s = ");
    for index in 0..terms {
        if index > 0 {
            source.push('+');
        }
        source.push_str("\"x\"");
    }
    source.push_str("; return (int)s.size(); }\n");
    source
}

fn else_if_chain(branches: usize) -> String {
    let mut source = String::from("int main() { int x = 0; int r = -1; if (x == 0) r = 0;");
    for index in 1..branches {
        source.push_str(&format!(" else if (x == {index}) r = {index};"));
    }
    source.push_str(" return r; }\n");
    source
}

fn nested_calls(depth: usize) -> String {
    let mut source = String::from("int inc(int x) { return x + 1; }\nint main() { return ");
    for _ in 0..depth {
        source.push_str("inc(");
    }
    source.push('0');
    for _ in 0..depth {
        source.push(')');
    }
    source.push_str("; }\n");
    source
}

fn nested_blocks(depth: usize) -> String {
    let mut source = String::from("int main() { ");
    source.push_str(&"{".repeat(depth));
    source.push_str("int x = 5; x += 1; ");
    source.push_str(&"}".repeat(depth));
    source.push_str(" return 0; }\n");
    source
}

fn nested_parentheses(depth: usize) -> String {
    let mut source = String::from("int main() { int v = ");
    source.push_str(&"(".repeat(depth));
    source.push('1');
    source.push_str(&")".repeat(depth));
    source.push_str("; return v; }\n");
    source
}

fn nested_parenthesized_ternary(depth: usize) -> String {
    let mut source = String::from("int main() { int v = ");
    for _ in 0..depth {
        source.push('(');
    }
    source.push_str("1?1:");
    for _ in 1..depth {
        source.push_str("1?1:");
    }
    source.push('0');
    source.push_str(&")".repeat(depth));
    source.push_str("; return v; }\n");
    source
}

fn nested_ifs(depth: usize) -> String {
    let mut source = String::from("int main() { int v = 0;\n");
    for _ in 0..depth {
        source.push_str("  if (v >= 0) {\n");
    }
    source.push_str("  v += 1;\n");
    for _ in 0..depth {
        source.push_str("  }\n");
    }
    source.push_str(" return v; }\n");
    source
}

/// Compiles a source and returns the diagnostic when it is rejected.
fn expect_diagnostic(source: String) -> Result<(), Box<dyn std::error::Error>> {
    match compile_on_big_stack(source) {
        Ok(_) => Err("expected a nesting-depth diagnostic".into()),
        Err(error) => {
            assert!(
                error.contains("Nesting depth"),
                "unexpected diagnostic: {error}"
            );
            Ok(())
        }
    }
}

#[test]
fn wide_binary_chain_compiles() -> Result<(), Box<dyn std::error::Error>> {
    compile_on_big_stack(binary_chain(20_000))?;
    Ok(())
}

#[test]
fn wide_string_concatenation_compiles() -> Result<(), Box<dyn std::error::Error>> {
    compile_on_big_stack(string_concat(4_000))?;
    Ok(())
}

#[test]
fn wide_else_if_chain_compiles() -> Result<(), Box<dyn std::error::Error>> {
    compile_on_big_stack(else_if_chain(5_000))?;
    Ok(())
}

#[test]
fn deeply_nested_calls_are_diagnosed() -> Result<(), Box<dyn std::error::Error>> {
    expect_diagnostic(nested_calls(5_000))
}

#[test]
fn deeply_nested_blocks_are_diagnosed() -> Result<(), Box<dyn std::error::Error>> {
    expect_diagnostic(nested_blocks(5_000))
}

#[test]
fn thousand_deep_calls_still_compile() -> Result<(), Box<dyn std::error::Error>> {
    compile_on_big_stack(nested_calls(1_000))?;
    Ok(())
}

#[test]
fn nested_calls_around_the_limit() -> Result<(), Box<dyn std::error::Error>> {
    let limit = nesting_limits().calls;
    compile_on_big_stack(nested_calls(limit - 1))?;
    expect_diagnostic(nested_calls(limit + 1))
}

#[test]
fn nested_parentheses_around_the_limit() -> Result<(), Box<dyn std::error::Error>> {
    let limit = nesting_limits().parentheses;
    compile_on_big_stack(nested_parentheses(limit - 1))?;
    expect_diagnostic(nested_parentheses(limit + 1))
}

#[test]
fn nested_parenthesized_ternaries_around_the_limit() -> Result<(), Box<dyn std::error::Error>> {
    let limit = nesting_limits().parentheses;
    compile_on_big_stack(nested_parenthesized_ternary(limit - 1))?;
    expect_diagnostic(nested_parenthesized_ternary(limit + 1))
}

#[test]
fn nested_ifs_around_the_limit() -> Result<(), Box<dyn std::error::Error>> {
    // The innermost statement of the nest is one level deeper than the `if`
    // statements themselves.
    let limit = nesting_limits().statements - 1;
    compile_on_big_stack(nested_ifs(limit - 1))?;
    expect_diagnostic(nested_ifs(limit + 1))
}
