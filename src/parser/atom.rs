use super::*;

impl Parser {
    pub(super) fn parse_atom(&mut self) -> Result<Expr, CompileError> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::IntLiteral(val, raw) => {
                let lower = raw.to_lowercase();
                let ty = if lower.contains("ll") {
                    Type::LongLong
                } else if lower.contains("l") {
                    Type::Long
                } else if lower.contains("u") {
                    Type::UnsignedInt
                } else {
                    Type::Int
                };
                Ok(Expr::IntLiteral(val, ty))
            }
            TokenKind::FloatLiteral(val, raw) => {
                let ty = if raw.ends_with('f') || raw.ends_with('F') {
                    Type::Float
                } else {
                    Type::Double
                };
                Ok(Expr::FloatLiteral(val, ty))
            }
            TokenKind::KwThis => Ok(Expr::This),
            TokenKind::KwNullptr => Ok(Expr::IntLiteral(0, Type::Pointer(Box::new(Type::Void)))),
            TokenKind::BoolLiteral(val) => Ok(Expr::BoolLiteral(val)),
            TokenKind::CharLiteral(val) => Ok(Expr::CharLiteral(val)),
            TokenKind::StringLiteral(val) => Ok(Expr::StringLiteral(val)),
            TokenKind::ColonColon => {
                let first = match self.advance().kind.clone() {
                    TokenKind::Ident(sub) => sub,
                    _ => {
                        return Err(CompileError::new(
                            "Expected identifier after '::'",
                            tok.line,
                            tok.col,
                        ));
                    }
                };
                let id = self.parse_qualified_identifier(first, tok.line, tok.col)?;
                if let Some(stripped) = id.strip_prefix("std::") {
                    Ok(Expr::Identifier(stripped.to_string()))
                } else {
                    Ok(Expr::Identifier(format!("::{}", id)))
                }
            }
            TokenKind::Ident(id) => {
                let mut id = self.parse_qualified_identifier(id, tok.line, tok.col)?;
                if let Some(stripped) = id.strip_prefix("std::") {
                    id = stripped.to_string();
                }
                Ok(Expr::Identifier(id))
            }
            _ => Err(CompileError::new(
                format!(
                    "Unexpected token in expression: {}",
                    tok.kind.diagnostic_name()
                ),
                tok.line,
                tok.col,
            )),
        }
    }

    /// Parses the `::`-separated tail of a qualified name.
    fn parse_qualified_identifier(
        &mut self,
        mut id: String,
        line: usize,
        col: usize,
    ) -> Result<String, CompileError> {
        while self.peek() == &TokenKind::ColonColon {
            self.advance();
            match self.advance().kind.clone() {
                TokenKind::Ident(sub) => {
                    id = format!("{}::{}", id, sub);
                }
                _ => {
                    return Err(CompileError::new(
                        "Expected identifier after '::'",
                        line,
                        col,
                    ));
                }
            }
        }
        Ok(id)
    }

    pub(super) fn parse_postfix(
        &mut self,
        mut expr: Expr,
        start: ExprPos,
    ) -> Result<Expr, CompileError> {
        loop {
            match self.peek() {
                TokenKind::LParen => {
                    self.advance();
                    let args = self.parse_call_args()?;
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                        pos: start,
                    };
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(TokenKind::RBracket)?;
                    expr = Expr::ArraySubscript {
                        array: Box::new(expr),
                        index: Box::new(index),
                        pos: start,
                    };
                }
                TokenKind::Dot => {
                    self.advance();
                    let member = self.parse_member_name(".")?;
                    expr = Expr::MemberAccess {
                        object: Box::new(expr),
                        member,
                        is_arrow: false,
                        pos: start,
                    };
                }
                TokenKind::Arrow => {
                    self.advance();
                    let member = self.parse_member_name("->")?;
                    expr = Expr::MemberAccess {
                        object: Box::new(expr),
                        member,
                        is_arrow: true,
                        pos: start,
                    };
                }
                TokenKind::PlusPlus => {
                    self.advance();
                    expr = Expr::Unary {
                        op: UnaryOp::PostInc,
                        expr: Box::new(expr),
                        pos: start,
                    };
                }
                TokenKind::MinusMinus => {
                    self.advance();
                    expr = Expr::Unary {
                        op: UnaryOp::PostDec,
                        expr: Box::new(expr),
                        pos: start,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, CompileError> {
        // One call costs one level of the call budget.
        let _guard = self.enter_call()?;
        let mut args = Vec::new();
        while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
            args.push(self.parse_expr()?);
            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(args)
    }

    fn parse_member_name(&mut self, operator: &str) -> Result<String, CompileError> {
        if self.peek() == &TokenKind::Tilde {
            self.advance();
            match self.advance().kind.clone() {
                TokenKind::Ident(id) => Ok(format!("~{}", id)),
                _ => Err(CompileError::new(
                    "Expected destructor name after '~'",
                    self.current().line,
                    self.current().col,
                )),
            }
        } else {
            match self.advance().kind.clone() {
                TokenKind::Ident(id) => Ok(id),
                _ => Err(CompileError::new(
                    format!("Expected member name after '{}'", operator),
                    self.current().line,
                    self.current().col,
                )),
            }
        }
    }
}
