//! Parser methods for exception statements (`try` and `throw`).

use super::*;

impl Parser {
    pub(super) fn parse_throw_stmt(
        &mut self,
        line: usize,
        col: usize,
    ) -> Result<Stmt, CompileError> {
        self.advance();
        let expr = if self.peek() != &TokenKind::Semi && self.peek() != &TokenKind::Eof {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(TokenKind::Semi)?;
        Ok(Stmt::new(StmtKind::Throw(expr), line, col))
    }

    pub(super) fn parse_try_stmt(&mut self, line: usize, col: usize) -> Result<Stmt, CompileError> {
        self.advance();
        let try_body = self.parse_stmt()?;
        self.expect(TokenKind::KwCatch)?;
        self.expect(TokenKind::LParen)?;
        let catch_param = if self.peek() == &TokenKind::Ellipsis {
            self.advance();
            None
        } else {
            let ty = self.parse_type()?;
            let name = if let TokenKind::Ident(id) = self.peek() {
                let n = id.clone();
                self.advance();
                n
            } else {
                String::new()
            };
            Some((ty, name))
        };
        self.expect(TokenKind::RParen)?;
        let catch_body = self.parse_stmt()?;
        Ok(Stmt::new(
            StmtKind::Try {
                try_body: Box::new(try_body),
                catch_param,
                catch_body: Box::new(catch_body),
            },
            line,
            col,
        ))
    }
}
