use super::*;
use crate::lexer::Lexer;
use crate::parser::Parser;

#[test]
fn test_sema_namespace_alias_resolution() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        namespace Original { int value = 42; int get() { return value; } }
        namespace Alias = Original;
        int main() {
            return Alias::get() + Alias::value;
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    sema.analyze_program(&mut program)?;
    Ok(())
}

#[test]
fn test_reinterpret_cast_rejects_float_to_int() -> Result<(), Box<dyn std::error::Error>> {
    let code = "int x = reinterpret_cast<int>(3.14f);";
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    let res = sema.analyze_program(&mut program);
    assert!(res.is_err(), "reinterpret_cast between float and int must be rejected");
    Ok(())
}

#[test]
fn test_reinterpret_cast_bypasses_conversion_operator() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        struct Foo { operator int*() const { return nullptr; } };
        void test() { Foo f; int* p = reinterpret_cast<int*>(f); }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    let res = sema.analyze_program(&mut program);
    assert!(res.is_err(), "reinterpret_cast cannot invoke user conversion operator on class");
    Ok(())
}

#[test]
fn test_reinterpret_cast_reference_rejects_rvalue() -> Result<(), Box<dyn std::error::Error>> {
    let code = "void test() { int& r = reinterpret_cast<int&>(42); }";
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    let res = sema.analyze_program(&mut program);
    assert!(res.is_err(), "reinterpret_cast to reference must reject rvalue");
    Ok(())
}

#[test]
fn test_reinterpret_cast_reference_accepts_float() -> Result<(), Box<dyn std::error::Error>> {
    let code = "void test() { float f = 1.0f; int& r = reinterpret_cast<int&>(f); }";
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    let res = sema.analyze_program(&mut program);
    assert!(res.is_ok(), "reinterpret_cast to reference must accept float lvalues: {:?}", res.err());
    Ok(())
}
