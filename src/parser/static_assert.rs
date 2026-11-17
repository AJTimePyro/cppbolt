//! static_assert declaration parsing.

use super::*;

impl Parser {
    pub(super) fn parse_static_assert_decl(&mut self) -> Result<Decl, CompileError> {
        let tok = self.current().clone();
        self.advance();
        self.expect(TokenKind::LParen)?;
        let condition = self.parse_expr_prec(2)?;
        let message = if self.peek() == &TokenKind::Comma {
            self.advance();
            let msg = self.current().clone();
            match &msg.kind {
                TokenKind::StringLiteral(s) => {
                    self.advance();
                    Some(s.clone())
                }
                _ => {
                    return Err(CompileError::new(
                        "static_assert message must be a string literal",
                        msg.line,
                        msg.col,
                    ));
                }
            }
        } else {
            None
        };
        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::Semi)?;
        Ok(Decl::StaticAssert {
            condition,
            message,
            line: tok.line,
            col: tok.col,
        })
    }
}
