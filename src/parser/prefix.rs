use super::*;

impl Parser {
    pub(super) fn parse_prefix(&mut self) -> Result<Expr, CompileError> {
        if let Some(expr) = self.parse_unary_prefix()? {
            return Ok(expr);
        }
        match self.peek() {
            TokenKind::KwNew => self.parse_new_expr(),
            TokenKind::KwDelete => self.parse_delete_expr(),
            TokenKind::KwStaticCast => self.parse_static_cast_expr(),
            TokenKind::KwSizeof => self.parse_sizeof_expr(),
            TokenKind::LParen => self.parse_parenthesized_expr(),
            _ => {
                let start = self.current_pos();
                let atom = self.parse_atom()?;
                self.parse_postfix(atom, start)
            }
        }
    }

    fn parse_unary_prefix(&mut self) -> Result<Option<Expr>, CompileError> {
        let op = match self.peek() {
            TokenKind::Plus => UnaryOp::Plus,
            TokenKind::Minus => UnaryOp::Neg,
            TokenKind::Bang => UnaryOp::Not,
            TokenKind::Tilde => UnaryOp::BitNot,
            TokenKind::Star => UnaryOp::Deref,
            TokenKind::Amp => UnaryOp::AddrOf,
            TokenKind::PlusPlus => UnaryOp::PreInc,
            TokenKind::MinusMinus => UnaryOp::PreDec,
            _ => return Ok(None),
        };
        let pos = self.current_pos();
        self.advance();
        let expr = self.parse_prefix()?;
        Ok(Some(Expr::Unary {
            op,
            expr: Box::new(expr),
            pos,
        }))
    }

    fn parse_new_expr(&mut self) -> Result<Expr, CompileError> {
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
            let args = self.parse_paren_expr_list()?;
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

    fn parse_delete_expr(&mut self) -> Result<Expr, CompileError> {
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

    fn parse_static_cast_expr(&mut self) -> Result<Expr, CompileError> {
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

    fn parse_sizeof_expr(&mut self) -> Result<Expr, CompileError> {
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

    fn parse_parenthesized_expr(&mut self) -> Result<Expr, CompileError> {
        // One parenthesis costs one level of the parenthesis budget.
        let _guard = self.enter_parenthesis()?;
        let start = self.current_pos();
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
                Err(error) => {
                    if self.nesting_exceeded.get() {
                        return Err(error);
                    }
                    self.pos = saved_pos;
                    self.advance();
                    let expr = self.parse_expr()?;
                    self.expect(TokenKind::RParen)?;
                    self.parse_postfix(expr, start)
                }
            }
        } else {
            self.advance();
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen)?;
            self.parse_postfix(expr, start)
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
