//! Parsing of function declarations.

use super::*;

impl Parser {
    pub(super) fn parse_function_tail(
        &mut self,
        name: String,
        return_type: Type,
        line: usize,
        col: usize,
    ) -> Result<FunctionDecl, CompileError> {
        self.expect(TokenKind::LParen)?;
        let params = self.parse_function_params()?;
        let is_const = self.parse_function_qualifiers()?;
        let ctor_initializers = self.parse_ctor_initializers()?;
        let (body, is_pure_virtual) = self.parse_function_body(line, col)?;

        Ok(FunctionDecl {
            name,
            return_type,
            params,
            body,
            is_virtual: false,
            is_pure_virtual,
            is_const,
            is_static: false,
            ctor_initializers,
            line,
            col,
        })
    }

    pub(super) fn parse_function_params(&mut self) -> Result<Vec<Param>, CompileError> {
        let mut params = Vec::new();

        while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
            if self.peek() == &TokenKind::Dot
                && self.peek_ahead(1) == &TokenKind::Dot
                && self.peek_ahead(2) == &TokenKind::Dot
            {
                self.advance();
                self.advance();
                self.advance();
                break;
            }
            let base_type = self.parse_type()?;
            let (ptype, pname) =
                if self.peek() == &TokenKind::LParen && self.peek_ahead(1) == &TokenKind::Star {
                    self.parse_function_pointer_declarator(base_type, None)?
                } else {
                    let name = if let TokenKind::Ident(id) = self.peek().clone() {
                        self.advance();
                        id
                    } else {
                        String::new()
                    };
                    (base_type, name)
                };

            let default_val = if self.peek() == &TokenKind::Assign {
                self.advance();
                Some(self.parse_expr()?)
            } else {
                None
            };

            params.push(Param {
                name: pname,
                ty: ptype,
                default_val,
            });

            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(TokenKind::RParen)?;
        if params.len() == 1 && params[0].ty == Type::Void && params[0].name.is_empty() {
            params.clear();
        }

        Ok(params)
    }

    /// Parses `(*name)(args)` after the base type of a function pointer and
    /// returns the pointer-to-function type together with the name.
    ///
    /// `name_error` is the diagnostic for a missing name; `None` accepts an
    /// unnamed declarator, as in a function-pointer parameter.
    pub(super) fn parse_function_pointer_declarator(
        &mut self,
        base_type: Type,
        name_error: Option<&str>,
    ) -> Result<(Type, String), CompileError> {
        self.advance();
        self.advance();
        let name = match name_error {
            Some(message) => match self.advance().kind.clone() {
                TokenKind::Ident(id) => id,
                _ => {
                    return Err(CompileError::new(
                        message,
                        self.current().line,
                        self.current().col,
                    ));
                }
            },
            None => {
                if let TokenKind::Ident(id) = self.peek().clone() {
                    self.advance();
                    id
                } else {
                    String::new()
                }
            }
        };
        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::LParen)?;
        let mut fn_params = Vec::new();
        while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
            let pt = self.parse_type()?;
            fn_params.push(pt);
            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok((
            Type::Pointer(Box::new(Type::Function {
                return_type: Box::new(base_type),
                param_types: fn_params,
            })),
            name,
        ))
    }

    pub(super) fn parse_function_qualifiers(&mut self) -> Result<bool, CompileError> {
        let mut is_const = false;
        while matches!(self.peek(), TokenKind::KwConst | TokenKind::KwOverride)
            || matches!(self.peek(), TokenKind::Ident(id) if id == "noexcept")
        {
            if self.peek() == &TokenKind::KwConst {
                self.advance();
                is_const = true;
            } else if self.peek() == &TokenKind::KwOverride {
                self.advance();
            } else if matches!(self.peek(), TokenKind::Ident(id) if id == "noexcept") {
                self.advance();
                if self.peek() == &TokenKind::LParen {
                    self.advance();
                    while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
                        self.advance();
                    }
                    if self.peek() == &TokenKind::RParen {
                        self.advance();
                    }
                }
            }
        }

        Ok(is_const)
    }

    pub(super) fn parse_ctor_initializers(
        &mut self,
    ) -> Result<Vec<(String, Vec<Expr>)>, CompileError> {
        let mut ctor_initializers = Vec::new();
        if self.peek() == &TokenKind::Colon {
            self.advance();
            loop {
                let iname = match self.advance().kind.clone() {
                    TokenKind::Ident(id) => id,
                    _ => {
                        return Err(CompileError::new(
                            "Expected member name in initializer list",
                            self.current().line,
                            self.current().col,
                        ));
                    }
                };
                self.expect(TokenKind::LParen)?;
                let args = self.parse_paren_expr_list()?;
                ctor_initializers.push((iname, args));
                if self.peek() == &TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        Ok(ctor_initializers)
    }

    pub(super) fn parse_function_body(
        &mut self,
        line: usize,
        col: usize,
    ) -> Result<(Option<Stmt>, bool), CompileError> {
        let mut is_pure_virtual = false;
        let body = if self.peek() == &TokenKind::Assign {
            self.advance();
            match self.advance().kind {
                TokenKind::IntLiteral(0, _) => {
                    is_pure_virtual = true;
                }
                _ => {
                    return Err(CompileError::new(
                        "Expected 0 in pure virtual declaration",
                        line,
                        col,
                    ));
                }
            }
            self.expect(TokenKind::Semi)?;
            None
        } else if self.peek() == &TokenKind::LBrace {
            Some(self.parse_compound_stmt()?)
        } else {
            self.expect(TokenKind::Semi)?;
            None
        };

        Ok((body, is_pure_virtual))
    }
}
