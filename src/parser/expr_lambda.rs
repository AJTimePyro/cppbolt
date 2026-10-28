//! Parsing of C++17 lambda expressions.

use super::*;
use crate::ast::{DefaultCapture, Expr, ExprPos, LambdaCapture};

impl Parser {
    pub(super) fn parse_lambda(&mut self, line: usize, col: usize) -> Result<Expr, CompileError> {
        let (default_capture, captures) = self.parse_lambda_captures()?;
        let params = if self.peek() == &TokenKind::LParen {
            self.advance();
            self.parse_function_params()?
        } else {
            Vec::new()
        };
        let is_mutable = if matches!(self.peek(), TokenKind::Ident(s) if s == "mutable") {
            self.advance();
            true
        } else {
            false
        };
        let return_type = if self.peek() == &TokenKind::Arrow {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_compound_stmt()?;
        Ok(Expr::Lambda {
            default_capture,
            captures,
            params,
            return_type,
            body: Box::new(body),
            is_mutable,
            pos: ExprPos::new(line, col),
        })
    }

    fn parse_lambda_captures(
        &mut self,
    ) -> Result<(Option<DefaultCapture>, Vec<LambdaCapture>), CompileError> {
        let mut default_capture = None;
        let mut captures = Vec::new();
        if self.peek() == &TokenKind::RBracket {
            self.advance();
            return Ok((None, captures));
        }
        while self.peek() != &TokenKind::RBracket && self.peek() != &TokenKind::Eof {
            if self.peek() == &TokenKind::Assign {
                let tok = self.advance().clone();
                if !captures.is_empty() {
                    return Err(CompileError::new(
                        "default capture must appear first in capture list",
                        tok.line,
                        tok.col,
                    ));
                }
                default_capture = Some(DefaultCapture::ByValue);
            } else if self.peek() == &TokenKind::Amp
                && matches!(self.peek_ahead(1), TokenKind::Comma | TokenKind::RBracket)
            {
                let tok = self.advance().clone();
                if !captures.is_empty() {
                    return Err(CompileError::new(
                        "default capture must appear first in capture list",
                        tok.line,
                        tok.col,
                    ));
                }
                default_capture = Some(DefaultCapture::ByReference);
            } else {
                let is_reference = if self.peek() == &TokenKind::Amp {
                    self.advance();
                    true
                } else {
                    false
                };
                let tok = self.advance().clone();
                let name = match tok.kind {
                    TokenKind::Ident(id) => id,
                    TokenKind::KwThis => "this".to_string(),
                    _ => {
                        return Err(CompileError::new(
                            format!(
                                "expected variable name in capture list, found {}",
                                tok.kind.diagnostic_name()
                            ),
                            tok.line,
                            tok.col,
                        ));
                    }
                };
                captures.push(LambdaCapture { name, is_reference });
            }
            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RBracket)?;
        Ok((default_capture, captures))
    }
}
