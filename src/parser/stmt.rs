use super::*;

impl Parser {
    pub(super) fn parse_stmt(&mut self) -> Result<Stmt, CompileError> {
        let _guard = self.enter_nesting()?;
        let tok = self.current().clone();
        match self.peek() {
            TokenKind::KwUsing => {
                self.advance();
                if self.peek() == &TokenKind::KwNamespace {
                    self.advance();
                    let ns = self.parse_qualified_name()?;
                    self.expect(TokenKind::Semi)?;
                    self.open_namespaces.push(ns.clone());
                    Ok(Stmt::new(StmtKind::UsingNamespace(ns), tok.line, tok.col))
                } else {
                    let name = self.parse_qualified_name()?;
                    self.expect(TokenKind::Semi)?;
                    Ok(Stmt::new(StmtKind::UsingDecl(name), tok.line, tok.col))
                }
            }
            TokenKind::Semi => {
                self.advance();
                Ok(Stmt::new(StmtKind::Empty, tok.line, tok.col))
            }
            TokenKind::LBrace => self.parse_compound_stmt(),
            TokenKind::KwIf => self.parse_if_stmt(),
            TokenKind::KwWhile => self.parse_while_stmt(),
            TokenKind::KwDo => self.parse_do_while_stmt(),
            TokenKind::KwFor => self.parse_for_stmt(),
            TokenKind::KwSwitch => self.parse_switch_stmt(),
            TokenKind::KwCase => {
                self.advance();
                let expr = self.parse_expr()?;
                let val = match expr {
                    Expr::IntLiteral(n, _) => n,
                    _ => {
                        return Err(CompileError::new(
                            "Case value must be integer constant",
                            self.current().line,
                            self.current().col,
                        ));
                    }
                };
                self.expect(TokenKind::Colon)?;
                Ok(Stmt::new(StmtKind::Case(val), tok.line, tok.col))
            }
            TokenKind::KwDefault => {
                self.advance();
                self.expect(TokenKind::Colon)?;
                Ok(Stmt::new(StmtKind::Default, tok.line, tok.col))
            }
            TokenKind::KwBreak => {
                self.advance();
                self.expect(TokenKind::Semi)?;
                Ok(Stmt::new(StmtKind::Break, tok.line, tok.col))
            }
            TokenKind::KwContinue => {
                self.advance();
                self.expect(TokenKind::Semi)?;
                Ok(Stmt::new(StmtKind::Continue, tok.line, tok.col))
            }
            TokenKind::KwReturn => {
                self.advance();
                let expr = if self.peek() != &TokenKind::Semi {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.expect(TokenKind::Semi)?;
                Ok(Stmt::new(StmtKind::Return(expr), tok.line, tok.col))
            }
            TokenKind::KwStatic => {
                self.advance();
                let mut stmt = self.parse_var_decl()?;
                if let StmtKind::VarDecl {
                    ref mut is_static, ..
                } = stmt.kind
                {
                    *is_static = true;
                }
                Ok(stmt)
            }
            _ => {
                if self.is_type_start() {
                    let saved_pos = self.pos;
                    match self.parse_var_decl() {
                        Ok(stmt) => Ok(stmt),
                        Err(error) => {
                            if self.nesting_exceeded.get() {
                                return Err(error);
                            }
                            self.pos = saved_pos;
                            let expr = self.parse_expr()?;
                            self.expect(TokenKind::Semi)?;
                            Ok(Stmt::new(StmtKind::Expr(expr), tok.line, tok.col))
                        }
                    }
                } else {
                    let expr = self.parse_expr()?;
                    self.expect(TokenKind::Semi)?;
                    Ok(Stmt::new(StmtKind::Expr(expr), tok.line, tok.col))
                }
            }
        }
    }

    pub(super) fn parse_compound_stmt(&mut self) -> Result<Stmt, CompileError> {
        let tok = self.current().clone();
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(Stmt::new(StmtKind::Compound(stmts), tok.line, tok.col))
    }

    pub(super) fn parse_var_decl(&mut self) -> Result<Stmt, CompileError> {
        let tok = self.current().clone();
        let base_type = self.parse_type()?;
        let (final_ty, name) =
            if self.peek() == &TokenKind::LParen && self.peek_ahead(1) == &TokenKind::Star {
                // Function-pointer local: Base (*name)(args).
                self.advance();
                self.advance();
                let id = match self.advance().kind.clone() {
                    TokenKind::Ident(id) => id,
                    _ => {
                        return Err(CompileError::new(
                            "Expected variable name",
                            self.current().line,
                            self.current().col,
                        ));
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
                (
                    Type::Pointer(Box::new(Type::Function {
                        return_type: Box::new(base_type),
                        param_types: fn_params,
                    })),
                    id,
                )
            } else {
                let id = match self.advance().kind.clone() {
                    TokenKind::Ident(id) => id,
                    _ => {
                        return Err(CompileError::new(
                            "Expected variable name",
                            self.current().line,
                            self.current().col,
                        ));
                    }
                };
                (self.parse_array_suffix(base_type)?, id)
            };

        let init = if self.peek() == &TokenKind::Assign {
            self.advance();
            Some(self.parse_initializer()?)
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
            Some(Expr::InitializerList(args))
        } else {
            None
        };

        self.expect(TokenKind::Semi)?;
        Ok(Stmt::new(
            StmtKind::VarDecl {
                name,
                ty: final_ty,
                init,
                is_static: false,
            },
            tok.line,
            tok.col,
        ))
    }

    pub(super) fn parse_initializer(&mut self) -> Result<Expr, CompileError> {
        let _guard = self.enter_nesting()?;
        if self.peek() == &TokenKind::LBrace {
            self.advance();
            let mut list = Vec::new();
            while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
                list.push(self.parse_initializer()?);
                if self.peek() == &TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RBrace)?;
            Ok(Expr::InitializerList(list))
        } else {
            self.parse_expr()
        }
    }

    pub(super) fn parse_if_stmt(&mut self) -> Result<Stmt, CompileError> {
        let tok = self.current().clone();
        // Parse `else if` chains iteratively so an arbitrarily long chain costs
        // one parser frame, not one frame per branch.
        let mut branches = Vec::new();
        loop {
            self.advance();
            self.expect(TokenKind::LParen)?;
            let cond = self.parse_expr()?;
            self.expect(TokenKind::RParen)?;
            let then_stmt = Box::new(self.parse_stmt()?);
            branches.push((cond, then_stmt));

            if self.peek() != &TokenKind::KwElse {
                break;
            }
            self.advance();
            if self.peek() != &TokenKind::KwIf {
                let else_stmt = Box::new(self.parse_stmt()?);
                return Ok(Stmt::new(
                    StmtKind::If {
                        branches,
                        else_stmt: Some(else_stmt),
                    },
                    tok.line,
                    tok.col,
                ));
            }
        }
        Ok(Stmt::new(
            StmtKind::If {
                branches,
                else_stmt: None,
            },
            tok.line,
            tok.col,
        ))
    }

    pub(super) fn parse_while_stmt(&mut self) -> Result<Stmt, CompileError> {
        let tok = self.current().clone();
        self.advance();
        self.expect(TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        let body = Box::new(self.parse_stmt()?);
        Ok(Stmt::new(StmtKind::While { cond, body }, tok.line, tok.col))
    }

    pub(super) fn parse_do_while_stmt(&mut self) -> Result<Stmt, CompileError> {
        let tok = self.current().clone();
        self.advance();
        let body = Box::new(self.parse_stmt()?);
        self.expect(TokenKind::KwWhile)?;
        self.expect(TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::Semi)?;
        Ok(Stmt::new(
            StmtKind::DoWhile { body, cond },
            tok.line,
            tok.col,
        ))
    }

    pub(super) fn parse_for_stmt(&mut self) -> Result<Stmt, CompileError> {
        let tok = self.current().clone();
        self.advance();
        self.expect(TokenKind::LParen)?;

        let init = if self.peek() == &TokenKind::Semi {
            self.advance();
            None
        } else if self.is_type_start() {
            Some(Box::new(self.parse_var_decl()?))
        } else {
            let expr_tok = self.current().clone();
            let expr = self.parse_expr()?;
            self.expect(TokenKind::Semi)?;
            Some(Box::new(Stmt::new(
                StmtKind::Expr(expr),
                expr_tok.line,
                expr_tok.col,
            )))
        };

        let cond = if self.peek() != &TokenKind::Semi {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(TokenKind::Semi)?;

        let step = if self.peek() != &TokenKind::RParen {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(TokenKind::RParen)?;

        let body = Box::new(self.parse_stmt()?);
        Ok(Stmt::new(
            StmtKind::For {
                init,
                cond,
                step,
                body,
            },
            tok.line,
            tok.col,
        ))
    }

    pub(super) fn parse_switch_stmt(&mut self) -> Result<Stmt, CompileError> {
        let tok = self.current().clone();
        self.advance();
        self.expect(TokenKind::LParen)?;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        let body = Box::new(self.parse_stmt()?);
        Ok(Stmt::new(
            StmtKind::Switch { expr, body },
            tok.line,
            tok.col,
        ))
    }
}
