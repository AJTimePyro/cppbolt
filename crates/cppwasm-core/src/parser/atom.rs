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
                let mut id = match self.advance().kind.clone() {
                    TokenKind::Ident(sub) => sub,
                    _ => {
                        return Err(CompileError::new(
                            "Expected identifier after '::'",
                            tok.line,
                            tok.col,
                        ));
                    }
                };
                while self.peek() == &TokenKind::ColonColon {
                    self.advance();
                    match self.advance().kind.clone() {
                        TokenKind::Ident(sub) => {
                            id = format!("{}::{}", id, sub);
                        }
                        _ => {
                            return Err(CompileError::new(
                                "Expected identifier after '::'",
                                tok.line,
                                tok.col,
                            ));
                        }
                    }
                }
                if id.starts_with("std::") {
                    id = id["std::".len()..].to_string();
                    Ok(Expr::Identifier(id))
                } else {
                    Ok(Expr::Identifier(format!("::{}", id)))
                }
            }
            TokenKind::Ident(mut id) => {
                while self.peek() == &TokenKind::ColonColon {
                    self.advance();
                    match self.advance().kind.clone() {
                        TokenKind::Ident(sub) => {
                            id = format!("{}::{}", id, sub);
                        }
                        _ => {
                            return Err(CompileError::new(
                                "Expected identifier after '::'",
                                tok.line,
                                tok.col,
                            ));
                        }
                    }
                }
                if id.starts_with("std::") {
                    id = id["std::".len()..].to_string();
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

    pub(super) fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, CompileError> {
        loop {
            match self.peek() {
                TokenKind::LParen => {
                    self.advance();
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
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(TokenKind::RBracket)?;
                    expr = Expr::ArraySubscript {
                        array: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                TokenKind::Dot => {
                    self.advance();
                    let member = if self.peek() == &TokenKind::Tilde {
                        self.advance();
                        match self.advance().kind.clone() {
                            TokenKind::Ident(id) => format!("~{}", id),
                            _ => {
                                return Err(CompileError::new(
                                    "Expected destructor name after '~'",
                                    self.current().line,
                                    self.current().col,
                                ));
                            }
                        }
                    } else {
                        match self.advance().kind.clone() {
                            TokenKind::Ident(id) => id,
                            _ => {
                                return Err(CompileError::new(
                                    "Expected member name after '.'",
                                    self.current().line,
                                    self.current().col,
                                ));
                            }
                        }
                    };
                    expr = Expr::MemberAccess {
                        object: Box::new(expr),
                        member,
                        is_arrow: false,
                    };
                }
                TokenKind::Arrow => {
                    self.advance();
                    let member = if self.peek() == &TokenKind::Tilde {
                        self.advance();
                        match self.advance().kind.clone() {
                            TokenKind::Ident(id) => format!("~{}", id),
                            _ => {
                                return Err(CompileError::new(
                                    "Expected destructor name after '~'",
                                    self.current().line,
                                    self.current().col,
                                ));
                            }
                        }
                    } else {
                        match self.advance().kind.clone() {
                            TokenKind::Ident(id) => id,
                            _ => {
                                return Err(CompileError::new(
                                    "Expected member name after '->'",
                                    self.current().line,
                                    self.current().col,
                                ));
                            }
                        }
                    };
                    expr = Expr::MemberAccess {
                        object: Box::new(expr),
                        member,
                        is_arrow: true,
                    };
                }
                TokenKind::PlusPlus => {
                    self.advance();
                    expr = Expr::Unary {
                        op: UnaryOp::PostInc,
                        expr: Box::new(expr),
                    };
                }
                TokenKind::MinusMinus => {
                    self.advance();
                    expr = Expr::Unary {
                        op: UnaryOp::PostDec,
                        expr: Box::new(expr),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }
}
