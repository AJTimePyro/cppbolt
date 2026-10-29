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
            TokenKind::KwThrow => self.parse_throw_stmt(line, col),
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
        self.expect(TokenKind::Colon)?;
        Ok(Stmt::new(StmtKind::Case(expr, 0), line, col))
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

    fn parse_throw_stmt(&mut self, line: usize, col: usize) -> Result<Stmt, CompileError> {
        self.advance();
        if self.peek() != &TokenKind::Semi && self.peek() != &TokenKind::Eof {
            let _ = self.parse_expr()?;
        }
        if self.peek() == &TokenKind::Semi {
            self.advance();
        }
        Err(CompileError::new(
            "C++ exception handling ('throw') is not supported on this WebAssembly target",
            line,
            col,
        ))
    }

    fn parse_static_stmt(&mut self) -> Result<Stmt, CompileError> {
        self.advance();
        let mut stmt = self.parse_var_decl()?;
        mark_static(&mut stmt.kind);
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
        let specifiers = self.parse_specifier_type()?;
        if specifiers.strip_const() == &Type::Auto {
            if self.peek() == &TokenKind::LBracket {
                return self.parse_structured_binding(tok.line, tok.col, specifiers);
            } else if matches!(self.peek(), TokenKind::Amp | TokenKind::AndAnd)
                && self.peek_ahead(1) == &TokenKind::LBracket
            {
                self.advance();
                return self.parse_structured_binding(
                    tok.line,
                    tok.col,
                    Type::Reference(Box::new(specifiers)),
                );
            }
        }
        let first = self.parse_declarator(&specifiers, "Expected variable name")?;
        self.finish_var_decl(&tok, &specifiers, first)
    }

    fn parse_structured_binding(
        &mut self,
        line: usize,
        col: usize,
        ty: Type,
    ) -> Result<Stmt, CompileError> {
        self.expect(TokenKind::LBracket)?;
        let mut bindings = Vec::new();
        while self.peek() != &TokenKind::RBracket && self.peek() != &TokenKind::Eof {
            let tok = self.advance().clone();
            let name = match tok.kind {
                TokenKind::Ident(id) => id,
                _ => {
                    return Err(CompileError::new(
                        "Expected identifier in structured binding",
                        tok.line,
                        tok.col,
                    ));
                }
            };
            bindings.push(name);
            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RBracket)?;
        self.expect(TokenKind::Assign)?;
        let init = self.parse_expr()?;
        self.expect(TokenKind::Semi)?;

        Ok(Stmt::new(
            StmtKind::StructuredBinding { bindings, ty, init },
            line,
            col,
        ))
    }

    fn finish_var_decl(
        &mut self,
        tok: &Token,
        specifiers: &Type,
        first: (Type, String),
    ) -> Result<Stmt, CompileError> {
        let mut declarators = Vec::new();
        let (first_ty, first_name) = first;
        let first_init = self.parse_var_init(&first_ty, tok.line, tok.col)?;
        declarators.push(Stmt::new(
            StmtKind::VarDecl {
                name: first_name,
                ty: first_ty,
                init: first_init,
                is_static: false,
            },
            tok.line,
            tok.col,
        ));
        while self.peek() == &TokenKind::Comma {
            self.advance();
            let (ty, name) = self.parse_declarator(specifiers, "Expected variable name")?;
            let init = self.parse_var_init(&ty, tok.line, tok.col)?;
            declarators.push(Stmt::new(
                StmtKind::VarDecl {
                    name,
                    ty,
                    init,
                    is_static: false,
                },
                tok.line,
                tok.col,
            ));
        }
        self.expect(TokenKind::Semi)?;
        match declarators.len() {
            1 => declarators
                .pop()
                .ok_or_else(|| crate::internal_error!(tok.line, tok.col, "missing declarator")),
            _ => Ok(Stmt::new(
                StmtKind::VarDeclGroup(declarators),
                tok.line,
                tok.col,
            )),
        }
    }

    /// Parses the initializer of one declarator of type `ty`.
    pub(super) fn parse_var_init(
        &mut self,
        ty: &Type,
        line: usize,
        col: usize,
    ) -> Result<Option<Expr>, CompileError> {
        if self.peek() == &TokenKind::Assign {
            self.advance();
            Ok(Some(self.parse_initializer()?))
        } else if self.peek() == &TokenKind::LParen {
            self.advance();
            let args = self.parse_paren_expr_list()?;
            match ty.strip_const() {
                // A parenthesized initializer of a class type is a
                // constructor call, not an initializer list.
                Type::Struct(name) => Ok(Some(Expr::Call {
                    callee: Box::new(Expr::Identifier(name.clone())),
                    args,
                    pos: ExprPos::new(line, col),
                })),
                // Scalar types unpack a single expression; arrays stay an initializer list.
                _ if args.len() == 1 && !matches!(ty.strip_const(), Type::Array(_, _)) => {
                    Ok(args.into_iter().next())
                }
                _ => Ok(Some(Expr::InitializerList {
                    elements: args,
                    pos: ExprPos::new(line, col),
                })),
            }
        } else if self.peek() == &TokenKind::LBrace {
            Ok(Some(self.parse_initializer()?))
        } else {
            Ok(None)
        }
    }

    pub(super) fn parse_initializer(&mut self) -> Result<Expr, CompileError> {
        let _guard = self.enter_expression()?;
        if self.peek() == &TokenKind::LBrace {
            let tok = self.advance().clone();
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
            Ok(Expr::InitializerList {
                elements: list,
                pos: ExprPos::new(tok.line, tok.col),
            })
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
            let decl_tok = self.current().clone();
            let specifiers = self.parse_specifier_type()?;
            let first = self.parse_declarator(&specifiers, "Expected variable name")?;
            if self.peek() == &TokenKind::Colon {
                self.advance();
                let range = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                let body = Box::new(self.parse_stmt()?);
                let decl = Box::new(Stmt::new(
                    StmtKind::VarDecl {
                        name: first.1,
                        ty: first.0,
                        init: None,
                        is_static: false,
                    },
                    decl_tok.line,
                    decl_tok.col,
                ));
                return Ok(Stmt::new(
                    StmtKind::RangeFor { decl, range, body },
                    tok.line,
                    tok.col,
                ));
            }
            Some(Box::new(self.finish_var_decl(
                &decl_tok,
                &specifiers,
                first,
            )?))
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

/// Marks every declarator of one declaration as `static`.
fn mark_static(kind: &mut StmtKind) {
    match kind {
        StmtKind::VarDecl { is_static, .. } => *is_static = true,
        StmtKind::VarDeclGroup(stmts) => {
            for stmt in stmts {
                mark_static(&mut stmt.kind);
            }
        }
        _ => {}
    }
}
