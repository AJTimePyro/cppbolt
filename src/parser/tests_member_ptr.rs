use super::*;
use crate::lexer::Lexer;

fn parse_ok(src: &str) -> bool {
    let Ok(tokens) = Lexer::new(src).tokenize() else {
        return false;
    };
    Parser::new(tokens).parse_program().is_ok()
}

#[test]
fn test_parse_member_pointer_types_and_exprs() {
    assert!(parse_ok("struct Point { int x; int y; }; int Point::* pm;"));
    assert!(parse_ok("struct Point { int x; }; void (Point::* pmf)(int, double);"));
    assert!(parse_ok("struct Point { int x; }; using PM = int Point::*;"));
    assert!(parse_ok(
        "struct Point { int x; int y; }; \
         int main() { Point p; int Point::* ptr; p .* ptr; }"
    ));
    assert!(parse_ok(
        "struct Point { int x; int y; }; \
         int main() { Point* p; int Point::* ptr; p ->* ptr; }"
    ));
}
