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

        if self.peek() == &TokenKind::KwTemplate {
            return self.parse_template_decl(declarations);
        }

        if self.peek() == &TokenKind::KwNamespace {
            return self.parse_namespace_decl(declarations);
        }
        if self.peek() == &TokenKind::KwStaticAssert {
            let decl = self.parse_static_assert_decl()?;
            declarations.push(decl);
            return Ok(());
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

        if self.peek() == &TokenKind::KwStruct
            || self.peek() == &TokenKind::KwClass
            || self.is_struct_after_alignas()
        {
            for decl in self.parse_struct()? {
                flatten_struct_decl(decl, declarations);
            }
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
        let line = self.current().line;
        let col = self.current().col;
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
        if self.peek() == &TokenKind::Assign {
            self.advance();
            let raw_target = self.parse_qualified_name()?;
            self.expect(TokenKind::Semi)?;
            let qualified_alias = self.qualify_name(ns_name);
            let target = if !self.current_namespace.is_empty() {
                let candidate = self.qualify_name(raw_target.clone());
                if self.known_namespaces.contains(&candidate) {
                    candidate
                } else {
                    raw_target
                }
            } else {
                raw_target
            };
            let canonical = self.resolve_namespace_alias(&target);
            self.namespace_aliases
                .insert(qualified_alias.clone(), canonical.clone());
            self.known_namespaces.insert(qualified_alias.clone());
            declarations.push(Decl::NamespaceAlias {
                alias: qualified_alias,
                target: canonical,
                line,
                col,
            });
            return Ok(());
        }
        self.expect(TokenKind::LBrace)?;
        let full_ns = self.qualify_name(ns_name.clone());
        self.known_namespaces.insert(full_ns);
        self.current_namespace.push(ns_name);
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            self.parse_declaration(declarations)?;
        }
        self.expect(TokenKind::RBrace)?;
        self.current_namespace.pop();
        Ok(())
    }

    fn parse_using_decl(&mut self, declarations: &mut Vec<Decl>) -> Result<(), CompileError> {
        let pos = self.current_pos();
        self.advance();
        if self.peek() == &TokenKind::KwNamespace {
            self.advance();
            let raw_ns = self.parse_qualified_name()?;
            let ns = self.resolve_namespace_alias(&raw_ns);
            self.expect(TokenKind::Semi)?;
            self.open_namespaces.push(ns.clone());
            declarations.push(Decl::UsingNamespace {
                name: ns,
                line: pos.line,
                col: pos.col,
            });
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
                let ty = self.parse_type_id()?;
                self.expect(TokenKind::Semi)?;
                let name = self.qualify_name(name);
                self.register_type_alias(name, ty, (pos.line, pos.col));
                return Ok(());
            }
            self.expect(TokenKind::Semi)?;
            declarations.push(Decl::UsingDecl {
                name,
                line: pos.line,
                col: pos.col,
            });
            Ok(())
        }
    }

    fn parse_typedef_decl(&mut self) -> Result<(), CompileError> {
        let pos = self.current_pos();
        self.advance();
        let base_type = self.parse_type()?;
        if self.is_function_pointer_declarator_ahead() {
            // Function-pointer typedef: Base (*Name)(args).
            let (final_ty, id) =
                self.parse_function_pointer_declarator(base_type, Some("Expected typedef name"))?;
            self.expect(TokenKind::Semi)?;
            self.register_type_alias(id, final_ty, (pos.line, pos.col));
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
            self.register_type_alias(name, base_type, (pos.line, pos.col));
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
        let mut is_constexpr = false;
        while matches!(
            self.peek(),
            TokenKind::KwInline
                | TokenKind::KwConstexpr
                | TokenKind::KwExplicit
                | TokenKind::KwStatic
        ) {
            if self.peek() == &TokenKind::KwConstexpr {
                is_constexpr = true;
            }
            self.advance();
        }

        if self.is_out_of_class_ctor_or_dtor() {
            let name = self.parse_qualified_name()?;
            let decl = self.parse_function_tail(name, Type::Void, start_tok.line, start_tok.col)?;
            return Ok(vec![Decl::Function(decl)]);
        }

        let specifiers = self.parse_specifier_type()?;
        let mut declarations = Vec::new();
        loop {
            let ty = self.parse_declarator_prefix(&specifiers)?;
            if declarations.is_empty() && self.is_function_pointer_declarator_ahead() {
                return Ok(vec![self.parse_function_pointer_global(
                    ty,
                    &start_tok,
                    is_constexpr,
                )?]);
            }
            let mut name = if self.peek() == &TokenKind::KwOperator {
                self.advance();
                self.parse_operator_name()?
            } else {
                self.parse_qualified_name()?
            };
            name = self.qualify_name(name);

            if declarations.is_empty() && self.peek() == &TokenKind::LParen {
                let decl =
                    self.parse_ambiguous_global_or_func(name, ty, &start_tok, is_constexpr)?;
                if matches!(decl, Decl::Function(_)) {
                    return Ok(vec![decl]);
                }
                declarations.push(self.mark_constexpr(decl, is_constexpr));
            } else {
                declarations.push(self.parse_global_var(name, ty, &start_tok, is_constexpr)?);
            }

            if self.peek() != &TokenKind::Comma {
                break;
            }
            self.advance();
        }
        self.expect(TokenKind::Semi)?;
        Ok(declarations)
    }

    /// Parses a declarator when followed by `(`, deciding between a function
    /// and a global variable with parenthesized initialization.
    fn parse_ambiguous_global_or_func(
        &mut self,
        name: String,
        ty: Type,
        start_tok: &Token,
        is_constexpr: bool,
    ) -> Result<Decl, CompileError> {
        let is_definite_function =
            self.peek_ahead(1) == &TokenKind::RParen || self.has_function_tail_indicators();
        if is_definite_function {
            let func = self.parse_function_tail(name, ty, start_tok.line, start_tok.col)?;
            return Ok(Decl::Function(func));
        }

        let starts_with_expr = token_starts_expr(self.peek_ahead(1));
        let saved_pos = self.pos;
        let saved_splits = self.shr_splits.len();
        match self.parse_function_tail(name.clone(), ty.clone(), start_tok.line, start_tok.col) {
            Ok(func) => Ok(Decl::Function(func)),
            Err(func_err) => {
                if self.nesting_exceeded.get() {
                    return Err(func_err);
                }
                self.rollback_pos(saved_pos, saved_splits);
                self.parse_global_var(name, ty, start_tok, is_constexpr)
                    .map_err(|var_err| {
                        if starts_with_expr || self.pos > saved_pos + 1 {
                            var_err
                        } else {
                            func_err
                        }
                    })
            }
        }
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

    /// Marks a global variable declaration as constexpr by wrapping its type
    /// in `Const`. Functions are returned unchanged.
    fn mark_constexpr(&self, decl: Decl, is_constexpr: bool) -> Decl {
        if !is_constexpr {
            return decl;
        }
        match decl {
            Decl::GlobalVar {
                name,
                ty,
                init,
                line,
                col,
            } => Decl::GlobalVar {
                name,
                ty: Type::Const(Box::new(ty)),
                init,
                line,
                col,
            },
            other => other,
        }
    }

    /// Parses one global variable declarator. The caller consumes the `,` that
    /// separates declarators and the terminating `;`.
    fn parse_global_var(
        &mut self,
        name: String,
        ty: Type,
        start_tok: &Token,
        is_constexpr: bool,
    ) -> Result<Decl, CompileError> {
        let final_ty = self.parse_array_suffix(ty)?;
        let init = self.parse_var_init(&final_ty, start_tok.line, start_tok.col)?;
        let final_ty = if is_constexpr {
            Type::Const(Box::new(final_ty))
        } else {
            final_ty
        };
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
        is_constexpr: bool,
    ) -> Result<Decl, CompileError> {
        let (final_ty, name) =
            self.parse_function_pointer_declarator(ty, Some("Expected variable name"))?;
        if let Type::Function {
            return_type,
            param_types,
            ..
        } = final_ty
        {
            let ctor_initializers = self.parse_ctor_initializers()?;
            let (body, is_pure_virtual) =
                self.parse_function_body(start_tok.line, start_tok.col)?;
            let params = param_types
                .into_iter()
                .map(|ty| Param {
                    name: String::new(),
                    ty,
                    default_val: None,
                    is_pack: false,
                })
                .collect();
            return Ok(Decl::Function(FunctionDecl {
                qualifiers: FunctionQualifiers::default(),
                name,
                return_type: *return_type,
                params,
                body,
                is_virtual: false,
                is_pure_virtual,
                is_const: false,
                is_explicit: false,
                ref_qualifier: RefQualifier::None,
                is_static: false,
                ctor_initializers,
                template_params: Vec::new(),
                line: start_tok.line,
                col: start_tok.col,
            }));
        }
        let init = if self.peek() == &TokenKind::Assign {
            self.advance();
            Some(self.parse_initializer()?)
        } else {
            None
        };
        self.expect(TokenKind::Semi)?;
        let final_ty = if is_constexpr {
            Type::Const(Box::new(final_ty))
        } else {
            final_ty
        };
        Ok(Decl::GlobalVar {
            name,
            ty: final_ty,
            init,
            line: start_tok.line,
            col: start_tok.col,
        })
    }
}

fn token_starts_expr(tok: &TokenKind) -> bool {
    matches!(
        tok,
        TokenKind::IntLiteral(..)
            | TokenKind::FloatLiteral(..)
            | TokenKind::StringLiteral(_)
            | TokenKind::CharLiteral(_)
            | TokenKind::BoolLiteral(_)
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Amp
            | TokenKind::Bang
            | TokenKind::Tilde
    )
}

fn flatten_struct_decl(decl: Decl, declarations: &mut Vec<Decl>) {
    if let Decl::StructDecl { nested_structs, .. } = &decl {
        for nested in nested_structs {
            flatten_struct_decl(nested.clone(), declarations);
        }
    }
    declarations.push(decl);
}
