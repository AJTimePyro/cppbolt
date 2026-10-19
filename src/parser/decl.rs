use super::*;

impl Parser {
    pub(super) fn parse_declaration(
        &mut self,
        declarations: &mut Vec<Decl>,
    ) -> Result<(), CompileError> {
        let _guard = self.enter_nesting()?;
        if self.peek() == &TokenKind::Semi {
            self.advance();
            return Ok(());
        }

        if self.peek() == &TokenKind::KwNamespace {
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
            return Ok(());
        }

        if self.peek() == &TokenKind::KwUsing {
            self.advance();
            if self.peek() == &TokenKind::KwNamespace {
                self.advance();
                let ns = self.parse_qualified_name()?;
                self.expect(TokenKind::Semi)?;
                self.open_namespaces.push(ns.clone());
                declarations.push(Decl::UsingNamespace(ns));
                return Ok(());
            } else {
                let mut name = if self.peek() == &TokenKind::KwOperator {
                    self.advance();
                    self.parse_operator_name()?
                } else {
                    self.parse_qualified_name()?
                };
                if !self.current_namespace.is_empty() && !name.contains("::") {
                    name = format!("{}::{}", self.current_namespace.join("::"), name);
                }
                if self.peek() == &TokenKind::Assign {
                    self.advance();
                    let ty = self.parse_type()?;
                    self.expect(TokenKind::Semi)?;
                    self.type_aliases.insert(name.clone(), ty);
                    self.known_types.insert(name);
                    return Ok(());
                } else {
                    self.expect(TokenKind::Semi)?;
                    declarations.push(Decl::UsingDecl(name));
                    return Ok(());
                }
            }
        }

        if self.peek() == &TokenKind::KwTypedef {
            self.advance();
            let base_type = self.parse_type()?;
            if self.peek() == &TokenKind::LParen && self.peek_ahead(1) == &TokenKind::Star {
                // Function-pointer typedef: Base (*Name)(args).
                self.advance();
                self.advance();
                let id = match self.advance().kind.clone() {
                    TokenKind::Ident(id) => id,
                    _ => {
                        return Err(CompileError::new(
                            "Expected typedef name",
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
                self.expect(TokenKind::Semi)?;
                let final_ty = Type::Pointer(Box::new(Type::Function {
                    return_type: Box::new(base_type),
                    param_types: fn_params,
                }));
                self.type_aliases.insert(id.clone(), final_ty);
                self.known_types.insert(id);
                return Ok(());
            } else {
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
                self.type_aliases.insert(name.clone(), base_type);
                self.known_types.insert(name);
                return Ok(());
            }
        }

        if self.peek() == &TokenKind::KwExtern
            && matches!(self.peek_ahead(1), TokenKind::StringLiteral(_))
        {
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
        declarations.push(decl);
        Ok(())
    }

    pub(super) fn parse_global_or_func(&mut self) -> Result<Decl, CompileError> {
        let start_tok = self.current().clone();
        while matches!(
            self.peek(),
            TokenKind::KwInline | TokenKind::KwConstexpr | TokenKind::KwExplicit
        ) {
            self.advance();
        }

        let ty = self.parse_type()?;
        if self.peek() == &TokenKind::LParen && self.peek_ahead(1) == &TokenKind::Star {
            // Function-pointer declarator: Base (*name)(args).
            self.advance();
            self.advance();
            let name = match self.advance().kind.clone() {
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
            let final_ty = Type::Pointer(Box::new(Type::Function {
                return_type: Box::new(ty),
                param_types: fn_params,
            }));
            let init = if self.peek() == &TokenKind::Assign {
                self.advance();
                Some(self.parse_initializer()?)
            } else {
                None
            };
            self.expect(TokenKind::Semi)?;
            return Ok(Decl::GlobalVar {
                name,
                ty: final_ty,
                init,
                line: start_tok.line,
                col: start_tok.col,
            });
        }
        let mut name = if self.peek() == &TokenKind::KwOperator {
            self.advance();
            self.parse_operator_name()?
        } else {
            self.parse_qualified_name()?
        };
        if !self.current_namespace.is_empty() && !name.contains("::") {
            name = format!("{}::{}", self.current_namespace.join("::"), name);
        }

        if self.peek() == &TokenKind::LParen {
            let func = self.parse_function_tail(name, ty, start_tok.line, start_tok.col)?;
            Ok(Decl::Function(func))
        } else {
            let final_ty = self.parse_array_suffix(ty.clone())?;
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

    pub(super) fn parse_function_tail(
        &mut self,
        name: String,
        return_type: Type,
        line: usize,
        col: usize,
    ) -> Result<FunctionDecl, CompileError> {
        self.expect(TokenKind::LParen)?;
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
                    // Function-pointer parameter: Base (*name)(args).
                    self.advance();
                    self.advance();
                    let id = if let TokenKind::Ident(id) = self.peek().clone() {
                        self.advance();
                        id
                    } else {
                        String::new()
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
                ctor_initializers.push((iname, args));
                if self.peek() == &TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }

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
}
