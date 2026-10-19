use super::*;

impl Parser {
    pub(super) fn parse_prefix(&mut self) -> Result<Expr, CompileError> {
        match self.peek() {
            TokenKind::Plus => {
                self.advance();
                let expr = self.parse_prefix()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Plus,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_prefix()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Bang => {
                self.advance();
                let expr = self.parse_prefix()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Tilde => {
                self.advance();
                let expr = self.parse_prefix()?;
                Ok(Expr::Unary {
                    op: UnaryOp::BitNot,
                    expr: Box::new(expr),
                })
            }
            TokenKind::KwNew => {
                self.advance();
                let placement = if self.peek() == &TokenKind::LParen {
                    self.advance();
                    let p_expr = self.parse_expr()?;
                    self.expect(TokenKind::RParen)?;
                    Some(Box::new(p_expr))
                } else {
                    None
                };
                let ty = self.parse_type()?;
                if self.peek() == &TokenKind::LBracket {
                    self.advance();
                    let size = self.parse_expr()?;
                    self.expect(TokenKind::RBracket)?;
                    Ok(Expr::New {
                        ty,
                        args: vec![],
                        is_array: true,
                        array_size: Some(Box::new(size)),
                        placement,
                    })
                } else if self.peek() == &TokenKind::LParen {
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
                    Ok(Expr::New {
                        ty,
                        args,
                        is_array: false,
                        array_size: None,
                        placement,
                    })
                } else {
                    Ok(Expr::New {
                        ty,
                        args: vec![],
                        is_array: false,
                        array_size: None,
                        placement,
                    })
                }
            }
            TokenKind::KwDelete => {
                self.advance();
                let is_array = if self.peek() == &TokenKind::LBracket {
                    self.advance();
                    self.expect(TokenKind::RBracket)?;
                    true
                } else {
                    false
                };
                let expr = self.parse_prefix()?;
                Ok(Expr::Delete {
                    expr: Box::new(expr),
                    is_array,
                })
            }
            TokenKind::Star => {
                self.advance();
                let expr = self.parse_prefix()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Deref,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Amp => {
                self.advance();
                let expr = self.parse_prefix()?;
                Ok(Expr::Unary {
                    op: UnaryOp::AddrOf,
                    expr: Box::new(expr),
                })
            }
            TokenKind::PlusPlus => {
                self.advance();
                let expr = self.parse_prefix()?;
                Ok(Expr::Unary {
                    op: UnaryOp::PreInc,
                    expr: Box::new(expr),
                })
            }
            TokenKind::MinusMinus => {
                self.advance();
                let expr = self.parse_prefix()?;
                Ok(Expr::Unary {
                    op: UnaryOp::PreDec,
                    expr: Box::new(expr),
                })
            }
            TokenKind::KwStaticCast => {
                self.advance();
                self.expect(TokenKind::Lt)?;
                let target_type = self.parse_type()?;
                self.expect(TokenKind::Gt)?;
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Cast {
                    target_type,
                    expr: Box::new(expr),
                })
            }
            TokenKind::KwSizeof => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                if self.is_type_start() {
                    let ty = self.parse_type()?;
                    self.expect(TokenKind::RParen)?;
                    Ok(Expr::SizeofType(ty))
                } else {
                    let expr = self.parse_expr()?;
                    self.expect(TokenKind::RParen)?;
                    Ok(Expr::SizeofExpr(Box::new(expr)))
                }
            }
            TokenKind::LParen => {
                // Check if cast: (int)x or (char)x
                if self.is_cast_ahead() {
                    let saved_pos = self.pos;
                    self.advance();
                    let cast_res = (|| -> Result<Expr, CompileError> {
                        let ty = self.parse_type()?;
                        self.expect(TokenKind::RParen)?;
                        let expr = self.parse_prefix()?;
                        Ok(Expr::Cast {
                            target_type: ty,
                            expr: Box::new(expr),
                        })
                    })();
                    match cast_res {
                        Ok(cast_expr) => Ok(cast_expr),
                        Err(_) => {
                            self.pos = saved_pos;
                            self.advance();
                            let expr = self.parse_expr()?;
                            self.expect(TokenKind::RParen)?;
                            self.parse_postfix(expr)
                        }
                    }
                } else {
                    self.advance();
                    let expr = self.parse_expr()?;
                    self.expect(TokenKind::RParen)?;
                    self.parse_postfix(expr)
                }
            }
            _ => {
                let atom = self.parse_atom()?;
                self.parse_postfix(atom)
            }
        }
    }

    pub(super) fn is_cast_ahead(&self) -> bool {
        if self.peek() != &TokenKind::LParen {
            return false;
        }
        match self.peek_ahead(1) {
            TokenKind::KwVoid
            | TokenKind::KwInt
            | TokenKind::KwLong
            | TokenKind::KwChar
            | TokenKind::KwBool
            | TokenKind::KwFloat
            | TokenKind::KwDouble
            | TokenKind::KwUnsigned
            | TokenKind::KwSigned
            | TokenKind::KwShort
            | TokenKind::KwConst
            | TokenKind::ColonColon => true,
            TokenKind::Ident(id) => {
                if id == "std" {
                    if self.peek_ahead(2) == &TokenKind::ColonColon {
                        if let TokenKind::Ident(sub) = self.peek_ahead(3) {
                            sub == "size_t"
                                || sub == "ptrdiff_t"
                                || sub == "nullptr_t"
                                || self.known_types.contains(sub)
                                || self.known_types.contains(&format!("std::{}", sub))
                                || self.type_aliases.contains_key(sub)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    id == "size_t"
                        || id == "ptrdiff_t"
                        || self.known_types.contains(id)
                        || self.type_aliases.contains_key(id)
                }
            }
            _ => false,
        }
    }
}
