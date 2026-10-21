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
        declarations.push(decl);
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
            self.type_aliases.insert(name.clone(), base_type);
            self.known_types.insert(name);
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
            return self.parse_function_pointer_global(ty, &start_tok);
        }
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
            self.parse_global_var(name, ty, &start_tok)
        }
    }

    /// Parses a global variable declaration.
    fn parse_global_var(
        &mut self,
        name: String,
        ty: Type,
        start_tok: &Token,
    ) -> Result<Decl, CompileError> {
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

    /// Parses a function-pointer global: Base (*name)(args).
    fn parse_function_pointer_global(
        &mut self,
        ty: Type,
        start_tok: &Token,
    ) -> Result<Decl, CompileError> {
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
        Ok(Decl::GlobalVar {
            name,
            ty: final_ty,
            init,
            line: start_tok.line,
            col: start_tok.col,
        })
    }
}
