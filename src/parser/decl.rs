use super::*;

impl Parser {
    pub(super) fn parse_declaration(
        &mut self,
        declarations: &mut Vec<Decl>,
    ) -> Result<(), CompileError> {
        let _guard = self.enter_declaration()?;
        if self.peek() == &TokenKind::Semi {
            self.advance();
            return Ok(());
        }

        if self.peek() == &TokenKind::KwNamespace {
            return self.parse_namespace_decl(declarations);
        }

        if self.peek() == &TokenKind::KwUsing {
            return self.parse_using_decl(declarations);
        }

        if self.peek() == &TokenKind::KwTypedef {
            return self.parse_typedef_decl();
        }

        if self.peek() == &TokenKind::KwExtern
            && matches!(self.peek_ahead(1), TokenKind::StringLiteral(_))
        {
            return self.parse_extern_block(declarations);
        }

        if self.peek() == &TokenKind::KwEnum {
            let decl = self.parse_enum()?;
            declarations.push(decl);
            return Ok(());
        }

        if self.peek() == &TokenKind::KwStruct || self.peek() == &TokenKind::KwClass {
            let decl = self.parse_struct()?;
            declarations.push(decl);
            return Ok(());
        }

        let decl = self.parse_global_or_func()?;
        declarations.extend(decl);
        Ok(())
    }

    fn parse_namespace_decl(&mut self, declarations: &mut Vec<Decl>) -> Result<(), CompileError> {
        self.advance();
        if self.peek() == &TokenKind::LBrace {
            self.advance();
            while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
                self.parse_declaration(declarations)?;
            }
            self.expect(TokenKind::RBrace)?;
            return Ok(());
        }
        let ns_name = match self.advance().kind.clone() {
            TokenKind::Ident(id) => id,
            _ => {
                return Err(CompileError::new(
                    "Expected namespace name",
                    self.current().line,
                    self.current().col,
                ));
            }
        };
        self.expect(TokenKind::LBrace)?;
        self.current_namespace.push(ns_name);
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            self.parse_declaration(declarations)?;
        }
        self.expect(TokenKind::RBrace)?;
        self.current_namespace.pop();
        Ok(())
    }

    fn parse_using_decl(&mut self, declarations: &mut Vec<Decl>) -> Result<(), CompileError> {
        self.advance();
        if self.peek() == &TokenKind::KwNamespace {
            self.advance();
            let ns = self.parse_qualified_name()?;
            self.expect(TokenKind::Semi)?;
            self.open_namespaces.push(ns.clone());
            declarations.push(Decl::UsingNamespace(ns));
            return Ok(());
        }
        {
            let mut name = if self.peek() == &TokenKind::KwOperator {
                self.advance();
                self.parse_operator_name()?
            } else {
                self.parse_qualified_name()?
            };
            name = self.qualify_name(name);
            if self.peek() == &TokenKind::Assign {
                self.advance();
                let ty = self.parse_type()?;
                self.expect(TokenKind::Semi)?;
                self.register_type_alias(name, ty);
                return Ok(());
            }
            self.expect(TokenKind::Semi)?;
            declarations.push(Decl::UsingDecl(name));
            Ok(())
        }
    }

    fn parse_typedef_decl(&mut self) -> Result<(), CompileError> {
        self.advance();
        let base_type = self.parse_type()?;
        if self.peek() == &TokenKind::LParen && self.peek_ahead(1) == &TokenKind::Star {
            // Function-pointer typedef: Base (*Name)(args).
            let (final_ty, id) =
                self.parse_function_pointer_declarator(base_type, Some("Expected typedef name"))?;
            self.expect(TokenKind::Semi)?;
            self.register_type_alias(id, final_ty);
            return Ok(());
        }
        {
            let name = match self.advance().kind.clone() {
                TokenKind::Ident(id) => id,
                _ => {
                    return Err(CompileError::new(
                        "Expected typedef alias name",
                        self.current().line,
                        self.current().col,
                    ));
                }
            };
            self.expect(TokenKind::Semi)?;
            self.register_type_alias(name, base_type);
            Ok(())
        }
    }

    fn parse_extern_block(&mut self, declarations: &mut Vec<Decl>) -> Result<(), CompileError> {
        // Linkage and ABI strings have no effect in this single-module
        // compiler; declarations inside the block are parsed normally.
        self.advance();
        self.advance();
        if self.peek() == &TokenKind::LBrace {
            self.advance();
            while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
                self.parse_declaration(declarations)?;
            }
            self.expect(TokenKind::RBrace)?;
            return Ok(());
        }
        Ok(())
    }

    /// Parses a declaration at namespace scope.
    ///
    /// A declaration with several declarators (`int a, b = 2;`) yields one
    /// `Decl` per declarator.
    pub(super) fn parse_global_or_func(&mut self) -> Result<Vec<Decl>, CompileError> {
        let start_tok = self.current().clone();
        while matches!(
            self.peek(),
            TokenKind::KwInline | TokenKind::KwConstexpr | TokenKind::KwExplicit
        ) {
            self.advance();
        }

        let specifiers = self.parse_specifier_type()?;
        let mut declarations = Vec::new();
        loop {
            let ty = self.parse_declarator_prefix(&specifiers)?;
            if declarations.is_empty()
                && self.peek() == &TokenKind::LParen
                && self.peek_ahead(1) == &TokenKind::Star
            {
                return Ok(vec![self.parse_function_pointer_global(ty, &start_tok)?]);
            }
            let mut name = if self.peek() == &TokenKind::KwOperator {
                self.advance();
                self.parse_operator_name()?
            } else {
                self.parse_qualified_name()?
            };
            name = self.qualify_name(name);

            if declarations.is_empty() && self.peek() == &TokenKind::LParen {
                let is_definite_function =
                    self.peek_ahead(1) == &TokenKind::RParen || self.has_function_tail_indicators();
                if is_definite_function {
                    let func = self.parse_function_tail(name, ty, start_tok.line, start_tok.col)?;
                    return Ok(vec![Decl::Function(func)]);
                }

                let saved_pos = self.pos;
                match self.parse_function_tail(
                    name.clone(),
                    ty.clone(),
                    start_tok.line,
                    start_tok.col,
                ) {
                    Ok(func) => return Ok(vec![Decl::Function(func)]),
                    Err(func_err) => {
                        if self.nesting_exceeded.get() {
                            return Err(func_err);
                        }
                        self.pos = saved_pos;
                        let decl = self
                            .parse_global_var(name, ty, &start_tok)
                            .map_err(|_| func_err)?;
                        declarations.push(decl);
                    }
                }
            } else {
                declarations.push(self.parse_global_var(name, ty, &start_tok)?);
            }

            if self.peek() != &TokenKind::Comma {
                break;
            }
            self.advance();
        }
        self.expect(TokenKind::Semi)?;
        Ok(declarations)
    }

    /// Checks if the tokens following the matching closing parenthesis indicate
    /// a function declaration or definition (such as a body, qualifiers, or trailing return).
    fn has_function_tail_indicators(&self) -> bool {
        let mut depth = 0;
        let mut rparen_idx = None;
        for (i, tok) in self.tokens[self.pos..].iter().enumerate() {
            match tok.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        rparen_idx = Some(self.pos + i);
                        break;
                    }
                }
                TokenKind::Semi | TokenKind::Eof => return false,
                _ => {}
            }
        }
        let Some(idx) = rparen_idx else {
            return false;
        };
        match self.tokens.get(idx + 1).map(|t| &t.kind) {
            Some(
                TokenKind::LBrace
                | TokenKind::Colon
                | TokenKind::KwConst
                | TokenKind::KwOverride
                | TokenKind::Amp
                | TokenKind::AndAnd
                | TokenKind::Arrow
                | TokenKind::Assign,
            ) => true,
            Some(TokenKind::Ident(id)) => id == "noexcept",
            _ => false,
        }
    }

    /// Parses one global variable declarator. The caller consumes the `,` that
    /// separates declarators and the terminating `;`.
    fn parse_global_var(
        &mut self,
        name: String,
        ty: Type,
        start_tok: &Token,
    ) -> Result<Decl, CompileError> {
        let final_ty = self.parse_array_suffix(ty)?;
        let init = self.parse_var_init(&final_ty, start_tok.line, start_tok.col)?;
        Ok(Decl::GlobalVar {
            name,
            ty: final_ty,
            init,
            line: start_tok.line,
            col: start_tok.col,
        })
    }

    /// Parses a function-pointer global: Base (*name)(args).
    fn parse_function_pointer_global(
        &mut self,
        ty: Type,
        start_tok: &Token,
    ) -> Result<Decl, CompileError> {
        let (final_ty, name) =
            self.parse_function_pointer_declarator(ty, Some("Expected variable name"))?;
        let init = if self.peek() == &TokenKind::Assign {
            self.advance();
            Some(self.parse_initializer()?)
        } else {
            None
        };
        self.expect(TokenKind::Semi)?;
        Ok(Decl::GlobalVar {
            name,
            ty: final_ty,
            init,
            line: start_tok.line,
            col: start_tok.col,
        })
    }
}
