use super::*;

impl Parser {
    pub(super) fn parse_stmt(&mut self) -> Result<Stmt, CompileError> {
        // A compound block has its own budget, so a nested `if` with its
        // block costs one statement level and one block level.
        if self.peek() == &TokenKind::LBrace {
            return self.parse_compound_stmt();
        }
        let _guard = self.enter_statement()?;
        let tok = self.current().clone();
        let line = tok.line;
        let col = tok.col;
        match self.peek() {
            TokenKind::KwUsing => self.parse_using_stmt(line, col),
            TokenKind::Semi => {
                self.advance();
                Ok(Stmt::new(StmtKind::Empty, line, col))
            }
            TokenKind::KwIf => self.parse_if_stmt(),
            TokenKind::KwWhile => self.parse_while_stmt(),
            TokenKind::KwDo => self.parse_do_while_stmt(),
            TokenKind::KwFor => self.parse_for_stmt(),
            TokenKind::KwSwitch => self.parse_switch_stmt(),
            TokenKind::KwCase => self.parse_case_label(line, col),
            TokenKind::KwDefault => {
                self.advance();
                self.expect(TokenKind::Colon)?;
                Ok(Stmt::new(StmtKind::Default, line, col))
            }
            TokenKind::KwBreak => {
                self.advance();
                self.expect(TokenKind::Semi)?;
                Ok(Stmt::new(StmtKind::Break, line, col))
            }
            TokenKind::KwContinue => {
                self.advance();
                self.expect(TokenKind::Semi)?;
                Ok(Stmt::new(StmtKind::Continue, line, col))
            }
            TokenKind::KwReturn => self.parse_return_stmt(line, col),
            TokenKind::KwStatic => self.parse_static_stmt(),
            _ => self.parse_expr_or_decl_stmt(line, col),
        }
    }

    fn parse_using_stmt(&mut self, line: usize, col: usize) -> Result<Stmt, CompileError> {
        self.advance();
        if self.peek() == &TokenKind::KwNamespace {
            self.advance();
            let ns = self.parse_qualified_name()?;
            self.expect(TokenKind::Semi)?;
            self.open_namespaces.push(ns.clone());
            Ok(Stmt::new(StmtKind::UsingNamespace(ns), line, col))
        } else {
            let name = self.parse_qualified_name()?;
            self.expect(TokenKind::Semi)?;
            Ok(Stmt::new(StmtKind::UsingDecl(name), line, col))
        }
    }

    fn parse_case_label(&mut self, line: usize, col: usize) -> Result<Stmt, CompileError> {
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
        Ok(Stmt::new(StmtKind::Case(val), line, col))
    }

    fn parse_return_stmt(&mut self, line: usize, col: usize) -> Result<Stmt, CompileError> {
        self.advance();
        let expr = if self.peek() != &TokenKind::Semi {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(TokenKind::Semi)?;
        Ok(Stmt::new(StmtKind::Return(expr), line, col))
    }

    fn parse_static_stmt(&mut self) -> Result<Stmt, CompileError> {
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

    fn parse_expr_or_decl_stmt(&mut self, line: usize, col: usize) -> Result<Stmt, CompileError> {
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
                    Ok(Stmt::new(StmtKind::Expr(expr), line, col))
                }
            }
        } else {
            let expr = self.parse_expr()?;
            self.expect(TokenKind::Semi)?;
            Ok(Stmt::new(StmtKind::Expr(expr), line, col))
        }
    }

    pub(super) fn parse_compound_stmt(&mut self) -> Result<Stmt, CompileError> {
        // One compound block costs one level of the block budget.
        let _guard = self.enter_block()?;
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
                self.parse_function_pointer_local(base_type)?
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
            match final_ty.strip_const() {
                // A parenthesized initializer of a class type is a
                // constructor call, not an initializer list.
                Type::Struct(name) => Some(Expr::Call {
                    callee: Box::new(Expr::Identifier(name.clone())),
                    args,
                    pos: ExprPos::new(tok.line, tok.col),
                }),
                // For every other type it stays a parenthesized list.
                _ => Some(Expr::InitializerList(args)),
            }
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

    /// Parses a function-pointer local: Base (*name)(args).
    fn parse_function_pointer_local(
        &mut self,
        base_type: Type,
    ) -> Result<(Type, String), CompileError> {
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
        Ok((
            Type::Pointer(Box::new(Type::Function {
                return_type: Box::new(base_type),
                param_types: fn_params,
            })),
            id,
        ))
    }

    pub(super) fn parse_initializer(&mut self) -> Result<Expr, CompileError> {
        let _guard = self.enter_expression()?;
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
