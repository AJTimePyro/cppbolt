use super::*;
use crate::lexer::Lexer;

#[test]
fn test_parse_fold_expressions_all_forms() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("(... + args)", true),
        ("(args + ...)", true),
        ("(0 + ... + args)", true),
        ("(args + ... + 0)", true),
        ("(args + ... - 0)", false),
        ("(args .* ...)", true),
        ("(obj .* ... .* args)", true),
        ("(args , ...)", true),
        ("(s << ... << args)", true),
        ("(init += ... += args)", true),
        ("(... && conds)", true),
    ];
    for (src, should_succeed) in cases {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let res = parser.parse_expr();
        assert_eq!(res.is_ok(), should_succeed, "Failed on: {}", src);
    }

    let mut lexer = Lexer::new("(... + args)");
    let tokens = lexer.tokenize()?;
    let expr = Parser::new(tokens).parse_expr()?;
    match expr {
        Expr::FoldExpr {
            op,
            ref left,
            ref right,
            ..
        } => {
            assert_eq!(
                op,
                crate::ast::fold_op::FoldOp::Binary(crate::ast::expr::BinaryOp::Add)
            );
            assert!(left.is_none());
            assert!(right.is_some());
        }
        _ => return Err("expected FoldExpr".into()),
    }

    let mut lexer = Lexer::new("(args .* ...)");
    let tokens = lexer.tokenize()?;
    let expr = Parser::new(tokens).parse_expr()?;
    match expr {
        Expr::FoldExpr {
            op,
            ref left,
            ref right,
            ..
        } => {
            assert_eq!(op, crate::ast::fold_op::FoldOp::DotStar);
            assert!(left.is_some());
            assert!(right.is_none());
        }
        _ => return Err("expected FoldExpr".into()),
    }
    Ok(())
}
