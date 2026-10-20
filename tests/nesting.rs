//! Native stack-safety tests for the shared nesting limit.
//!
//! Wide left-associative chains must compile; deeply nested constructs must
//! produce a normal diagnostic instead of aborting. Programs run on an
//! explicitly sized thread so unoptimized builds have room for the parser,
//! semantic-analysis, and code-generation frames.

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
    let error = match compile_on_big_stack(nested_calls(5_000)) {
        Ok(_) => return Err("expected a nesting-depth diagnostic".into()),
        Err(error) => error,
    };
    assert!(
        error.contains("Nesting depth"),
        "unexpected diagnostic: {error}"
    );
    Ok(())
}

#[test]
fn deeply_nested_blocks_are_diagnosed() -> Result<(), Box<dyn std::error::Error>> {
    let error = match compile_on_big_stack(nested_blocks(5_000)) {
        Ok(_) => return Err("expected a nesting-depth diagnostic".into()),
        Err(error) => error,
    };
    assert!(
        error.contains("Nesting depth"),
        "unexpected diagnostic: {error}"
    );
    Ok(())
}

#[test]
fn thousand_deep_calls_still_compile() -> Result<(), Box<dyn std::error::Error>> {
    compile_on_big_stack(nested_calls(1_000))?;
    Ok(())
}
