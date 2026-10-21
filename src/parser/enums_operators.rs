use super::*;

impl Parser {
    pub(super) fn parse_enum(&mut self) -> Result<Decl, CompileError> {
        self.advance();
        let name = match self.advance().kind.clone() {
            TokenKind::Ident(id) => id,
            _ => {
                return Err(CompileError::new(
                    "Expected enum name",
                    self.current().line,
                    self.current().col,
                ));
            }
        };
        self.known_types.insert(name.clone());
        self.known_enums.insert(name.clone());

        self.expect(TokenKind::LBrace)?;
        let mut variants = Vec::new();
        let mut next_val = 0i64;

        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            let vname = match self.advance().kind.clone() {
                TokenKind::Ident(id) => id,
                _ => {
                    return Err(CompileError::new(
                        "Expected enum variant identifier",
                        self.current().line,
                        self.current().col,
                    ));
                }
            };

            let val = if self.peek() == &TokenKind::Assign {
                self.advance();
                let expr = self.parse_expr()?;
                match expr {
                    Expr::IntLiteral(v, _) => v,
                    _ => {
                        return Err(CompileError::new(
                            "Enum variant value must be constant integer",
                            self.current().line,
                            self.current().col,
                        ));
                    }
                }
            } else {
                next_val
            };

            variants.push((vname, val));
            next_val = val + 1;

            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::Semi)?;

        Ok(Decl::EnumDecl { name, variants })
    }

    pub(super) fn parse_operator_name(&mut self) -> Result<String, CompileError> {
        let tok = self.advance().clone();
        let name = match tok.kind {
            TokenKind::Assign => "operator=",
            TokenKind::Plus => "operator+",
            TokenKind::Minus => "operator-",
            TokenKind::Star => "operator*",
            TokenKind::Slash => "operator/",
            TokenKind::Percent => "operator%",
            TokenKind::EqEq => "operator==",
            TokenKind::BangEq => "operator!=",
            TokenKind::Lt => "operator<",
            TokenKind::LtEq => "operator<=",
            TokenKind::Gt => "operator>",
            TokenKind::GtEq => "operator>=",
            TokenKind::Shl => "operator<<",
            TokenKind::Shr => "operator>>",
            TokenKind::LBracket => {
                self.expect(TokenKind::RBracket)?;
                "operator[]"
            }
            TokenKind::PlusAssign => "operator+=",
            TokenKind::MinusAssign => "operator-=",
            TokenKind::StarAssign => "operator*=",
            TokenKind::SlashAssign => "operator/=",
            TokenKind::PlusPlus => "operator++",
            TokenKind::MinusMinus => "operator--",
            TokenKind::Bang => "operator!",
            TokenKind::Tilde => "operator~",
            TokenKind::Amp => "operator&",
            TokenKind::Arrow => "operator->",
            TokenKind::KwBool => "operator bool",
            TokenKind::LParen => {
                self.expect(TokenKind::RParen)?;
                "operator()"
            }
            TokenKind::KwNew => {
                if self.peek() == &TokenKind::LBracket {
                    self.advance();
                    self.expect(TokenKind::RBracket)?;
                    "operator new[]"
                } else {
                    "operator new"
                }
            }
            TokenKind::KwDelete => {
                if self.peek() == &TokenKind::LBracket {
                    self.advance();
                    self.expect(TokenKind::RBracket)?;
                    "operator delete[]"
                } else {
                    "operator delete"
                }
            }
            _ => {
                return Err(CompileError::new(
                    format!(
                        "Unsupported operator in operator overload: {}",
                        tok.kind.diagnostic_name()
                    ),
                    tok.line,
                    tok.col,
                ));
            }
        };
        Ok(name.to_string())
    }

    pub(super) fn parse_qualified_name(&mut self) -> Result<String, CompileError> {
        let mut name = if self.peek() == &TokenKind::ColonColon {
            self.advance();
            match self.advance().kind.clone() {
                TokenKind::Ident(id) => id,
                _ => {
                    return Err(CompileError::new(
                        "Expected identifier after '::'",
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
                        "Expected identifier",
                        self.current().line,
                        self.current().col,
                    ));
                }
            }
        };

        while self.peek() == &TokenKind::ColonColon {
            self.advance();
            match self.advance().kind.clone() {
                TokenKind::Ident(id) => {
                    name = format!("{}::{}", name, id);
                }
                _ => {
                    return Err(CompileError::new(
                        "Expected identifier after '::'",
                        self.current().line,
                        self.current().col,
                    ));
                }
            }
        }

        name = QualifiedName::new(&name).plain().to_string();

        Ok(name)
    }
}
