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
