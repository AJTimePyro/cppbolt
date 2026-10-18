use super::*;

impl Parser {
    pub(super) fn parse_struct(&mut self) -> Result<Decl, CompileError> {
        let is_class = self.current().kind == TokenKind::KwClass;
        self.advance(); // consume struct or class
        let mut name = match self.advance().kind.clone() {
            TokenKind::Ident(id) => id,
            _ => {
                return Err(CompileError::new(
                    "Expected struct/class name",
                    self.current().line,
                    self.current().col,
                ));
            }
        };
        if !self.current_namespace.is_empty() && !name.contains("::") {
            name = format!("{}::{}", self.current_namespace.join("::"), name);
        }
        self.known_types.insert(name.clone());
        if let Some(short) = name.split("::").last() {
            self.known_types.insert(short.to_string());
        }
        let qualified = QualifiedName::new(&name);
        if qualified.is_global() {
            self.known_types.insert(qualified.std_name().into_owned());
        } else if qualified.in_std() {
            self.known_types.insert(qualified.plain().to_string());
        }

        if self.peek() == &TokenKind::Semi {
            self.advance();
            return Ok(Decl::StructDecl {
                name,
                is_class,
                base_classes: Vec::new(),
                fields: Vec::new(),
                methods: Vec::new(),
                static_fields: Vec::new(),
                private_members: Vec::new(),
            });
        }

        let mut base_classes = Vec::new();
        if self.peek() == &TokenKind::Colon {
            self.advance();
            loop {
                if matches!(
                    self.peek(),
                    TokenKind::KwPublic
                        | TokenKind::KwPrivate
                        | TokenKind::KwProtected
                        | TokenKind::KwVirtual
                ) {
                    self.advance();
                }
                match self.advance().kind.clone() {
                    TokenKind::Ident(base) => base_classes.push(base),
                    _ => {
                        return Err(CompileError::new(
                            "Expected base class name",
                            self.current().line,
                            self.current().col,
                        ));
                    }
                }
                if self.peek() == &TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut static_fields = Vec::new();
        let mut is_private = is_class;
        let mut private_members = Vec::new();

        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            // Public/private/protected labels
            if self.peek() == &TokenKind::KwPublic {
                self.advance();
                self.expect(TokenKind::Colon)?;
                is_private = false;
                continue;
            }
            if matches!(self.peek(), TokenKind::KwPrivate | TokenKind::KwProtected) {
                self.advance();
                self.expect(TokenKind::Colon)?;
                is_private = true;
                continue;
            }

            if self.peek() == &TokenKind::KwFriend {
                self.advance();
                while self.peek() != &TokenKind::Semi && self.peek() != &TokenKind::Eof {
                    self.advance();
                }
                if self.peek() == &TokenKind::Semi {
                    self.advance();
                }
                continue;
            }

            if self.peek() == &TokenKind::KwTypedef {
                self.advance();
                let ty = self.parse_type()?;
                let tname = match self.advance().kind.clone() {
                    TokenKind::Ident(id) => id,
                    _ => {
                        return Err(CompileError::new(
                            "Expected typedef alias identifier",
                            self.current().line,
                            self.current().col,
                        ));
                    }
                };
                self.expect(TokenKind::Semi)?;
                self.type_aliases
                    .insert(format!("{}::{}", name, tname), ty.clone());
                self.type_aliases.insert(tname.clone(), ty);
                self.known_types.insert(format!("{}::{}", name, tname));
                self.known_types.insert(tname);
                continue;
            }

            if self.peek() == &TokenKind::KwUsing {
                self.advance();
                let alias = match self.advance().kind.clone() {
                    TokenKind::Ident(id) => id,
                    _ => {
                        return Err(CompileError::new(
                            "Expected alias identifier in using",
                            self.current().line,
                            self.current().col,
                        ));
                    }
                };
                if self.peek() == &TokenKind::Assign {
                    self.advance();
                    let ty = self.parse_type()?;
                    self.expect(TokenKind::Semi)?;
                    self.type_aliases
                        .insert(format!("{}::{}", name, alias), ty.clone());
                    self.type_aliases.insert(alias.clone(), ty);
                    self.known_types.insert(format!("{}::{}", name, alias));
                    self.known_types.insert(alias);
                    continue;
                } else {
                    self.expect(TokenKind::Semi)?;
                    continue;
                }
            }

            while matches!(
                self.peek(),
                TokenKind::KwConstexpr | TokenKind::KwInline | TokenKind::KwExplicit
            ) {
                self.advance();
            }

            let start_tok = self.current().clone();
            let mut is_virtual = false;
            let mut is_static = false;

            if self.peek() == &TokenKind::KwVirtual {
                self.advance();
                is_virtual = true;
            }
            if self.peek() == &TokenKind::KwStatic {
                self.advance();
                is_static = true;
            }

            if self.peek() == &TokenKind::KwOperator && self.peek_ahead(1) == &TokenKind::KwBool {
                self.advance();
                self.advance();
                let fname = "operator bool".to_string();
                let mut func = self.parse_function_tail(
                    fname.clone(),
                    Type::Bool,
                    start_tok.line,
                    start_tok.col,
                )?;
                func.is_virtual = is_virtual;
                func.is_static = is_static;
                if is_private {
                    private_members.push(fname);
                }
                methods.push(func);
                continue;
            }

            // Destructor: ~ClassName()
            if self.peek() == &TokenKind::Tilde {
                self.advance(); // consume ~
                let dname = match self.advance().kind.clone() {
                    TokenKind::Ident(id) => format!("~{}", id),
                    _ => {
                        return Err(CompileError::new(
                            "Expected destructor name after ~",
                            start_tok.line,
                            start_tok.col,
                        ));
                    }
                };
                let mut dtor =
                    self.parse_function_tail(dname, Type::Void, start_tok.line, start_tok.col)?;
                dtor.is_virtual = is_virtual;
                methods.push(dtor);
                continue;
            }

            // Constructor: ClassName(...)
            let short_name = name.split("::").last().unwrap_or(&name);
            if let TokenKind::Ident(id) = self.peek().clone()
                && (id == name || id == short_name)
                && self.peek_ahead(1) == &TokenKind::LParen
            {
                self.advance(); // consume ctor name
                let mut ctor = self.parse_function_tail(
                    short_name.to_string(),
                    Type::Void,
                    start_tok.line,
                    start_tok.col,
                )?;
                ctor.is_virtual = is_virtual;
                methods.push(ctor);
                continue;
            }

            if self.is_type_start() {
                let ty = self.parse_type()?;
                let fname = if self.peek() == &TokenKind::KwOperator {
                    self.advance();
                    self.parse_operator_name()?
                } else {
                    match self.advance().kind.clone() {
                        TokenKind::Ident(id) => id,
                        _ => {
                            return Err(CompileError::new(
                                "Expected member identifier",
                                self.current().line,
                                self.current().col,
                            ));
                        }
                    }
                };

                // Check if member function
                if self.peek() == &TokenKind::LParen {
                    let mut func =
                        self.parse_function_tail(fname.clone(), ty, start_tok.line, start_tok.col)?;
                    func.is_virtual = is_virtual;
                    func.is_static = is_static;
                    if is_private {
                        private_members.push(fname);
                    }
                    methods.push(func);
                } else {
                    // Field
                    let final_ty = self.parse_array_suffix(ty.clone())?;
                    if is_static {
                        let init = if self.peek() == &TokenKind::Assign {
                            self.advance();
                            Some(self.parse_initializer()?)
                        } else {
                            None
                        };
                        self.expect(TokenKind::Semi)?;
                        if is_private {
                            private_members.push(fname.clone());
                        }
                        static_fields.push((fname, final_ty, init));
                    } else {
                        if is_private {
                            private_members.push(fname.clone());
                        }
                        fields.push((fname, final_ty));
                        while self.peek() == &TokenKind::Comma {
                            self.advance();
                            let next_fname = match self.advance().kind.clone() {
                                TokenKind::Ident(id) => id,
                                _ => {
                                    return Err(CompileError::new(
                                        "Expected member identifier after comma",
                                        start_tok.line,
                                        start_tok.col,
                                    ));
                                }
                            };
                            let next_ty = self.parse_array_suffix(ty.clone())?;
                            if is_private {
                                private_members.push(next_fname.clone());
                            }
                            fields.push((next_fname, next_ty));
                        }
                        self.expect(TokenKind::Semi)?;
                    }
                }
            } else {
                return Err(CompileError::new(
                    "Unexpected token in struct body",
                    self.current().line,
                    self.current().col,
                ));
            }
        }

        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::Semi)?;

        Ok(Decl::StructDecl {
            name,
            is_class,
            base_classes,
            fields,
            methods,
            static_fields,
            private_members,
        })
    }
}
