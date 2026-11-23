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
        let mut catches = Vec::new();
        while self.peek() == &TokenKind::KwCatch {
            self.advance();
            self.expect(TokenKind::LParen)?;
            let catch_param = if self.peek() == &TokenKind::Ellipsis {
                self.advance();
                None
            } else {
                let base_type = self.parse_type()?;
                let (ty, name) = if self.is_function_pointer_declarator_ahead() {
                    self.parse_function_pointer_declarator(base_type, None)?
                } else {
                    let name = if let TokenKind::Ident(id) = self.peek() {
                        let n = id.clone();
                        self.advance();
                        n
                    } else {
                        String::new()
                    };
                    (base_type, name)
                };
                Some((ty, name))
            };
            self.expect(TokenKind::RParen)?;
            let catch_body = self.parse_stmt()?;
            catches.push(CatchClause {
                param: catch_param,
                body: Box::new(catch_body),
            });
        }
        if catches.is_empty() {
            return Err(CompileError::new(
                "expected 'catch' after try block",
                line,
                col,
            ));
        }
        Ok(Stmt::new(
            StmtKind::Try {
                try_body: Box::new(try_body),
                catches,
            },
            line,
            col,
        ))
    }
}
