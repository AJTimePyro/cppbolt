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
        let quals = self.parse_function_qualifiers()?;
        let (is_const, ref_qualifier) = (quals.is_const, quals.ref_qualifier);
        let ctor_initializers = self.parse_ctor_initializers()?;
        let (body, is_pure_virtual) = self.parse_function_body(line, col)?;

        Ok(FunctionDecl {
            qualifiers: quals,
            name,
            return_type,
            params,
            body,
            is_virtual: false,
            is_pure_virtual,
            is_const,
            is_explicit: false,
            ref_qualifier,
            is_static: false,
            ctor_initializers,
            template_params: Vec::new(),
            line,
            col,
        })
    }

    pub(super) fn parse_function_params(&mut self) -> Result<Vec<Param>, CompileError> {
        let mut params = Vec::new();

        while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
            if self.peek() == &TokenKind::Ellipsis {
                self.advance();
                break;
            }
            params.push(self.parse_single_function_param()?);
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

    fn parse_single_function_param(&mut self) -> Result<Param, CompileError> {
        let base_type = self.parse_type()?;
        let is_pack = if self.peek() == &TokenKind::Ellipsis {
            self.advance();
            true
        } else {
            false
        };
        let (ptype, pname) = if self.is_function_pointer_declarator_ahead() {
            self.parse_function_pointer_declarator(base_type, None)?
        } else {
            let name = if let TokenKind::Ident(id) = self.peek().clone() {
                self.advance();
                id
            } else {
                String::new()
            };
            let ptype = if self.peek() == &TokenKind::LBracket {
                let arr_type = self.parse_array_suffix(base_type)?;
                match arr_type {
                    Type::Array(inner, _) => Type::Pointer(inner),
                    other => other,
                }
            } else {
                base_type
            };
            (ptype, name)
        };

        let default_val = if self.peek() == &TokenKind::Assign {
            if is_pack {
                return Err(CompileError::new(
                    "parameter pack cannot have a default argument",
                    self.current().line,
                    self.current().col,
                ));
            }
            self.advance();
            Some(self.parse_expr_prec(2)?)
        } else {
            None
        };

        Ok(Param {
            name: pname,
            ty: ptype,
            default_val,
            is_pack,
        })
    }

    pub(super) fn is_member_pointer_prefix_ahead_from(&self, mut offset: usize) -> bool {
        if self.peek_ahead(offset) == &TokenKind::ColonColon {
            offset += 1;
        }
        match self.peek_ahead(offset) {
            TokenKind::Ident(_) => offset += 1,
            _ => return false,
        }
        while self.peek_ahead(offset) == &TokenKind::ColonColon {
            offset += 1;
            if self.peek_ahead(offset) == &TokenKind::Star {
                return true;
            }
            match self.peek_ahead(offset) {
                TokenKind::Ident(_) => offset += 1,
                _ => return false,
            }
        }
        false
    }

    pub(super) fn parse_member_pointer_class_prefix(&mut self) -> Result<String, CompileError> {
        let mut class_name = String::new();
        if self.peek() == &TokenKind::ColonColon {
            self.advance();
            class_name.push_str("::");
        }
        match &self.advance().kind {
            TokenKind::Ident(id) => class_name.push_str(id),
            _ => {
                return Err(CompileError::new(
                    "Expected class name in member pointer",
                    self.current().line,
                    self.current().col,
                ));
            }
        }
        while self.peek() == &TokenKind::ColonColon && self.peek_ahead(1) != &TokenKind::Star {
            self.advance();
            match &self.advance().kind {
                TokenKind::Ident(sub) => {
                    class_name.push_str("::");
                    class_name.push_str(sub);
                }
                _ => {
                    return Err(CompileError::new(
                        "Expected identifier in qualified class name",
                        self.current().line,
                        self.current().col,
                    ));
                }
            }
        }
        self.expect(TokenKind::ColonColon)?;
        self.expect(TokenKind::Star)?;
        Ok(class_name)
    }

    pub(super) fn is_function_pointer_declarator_ahead(&self) -> bool {
        self.peek() == &TokenKind::LParen
            && (self.peek_ahead(1) == &TokenKind::Star
                || self.is_member_pointer_prefix_ahead_from(1))
    }

    /// Parses `(*name)(args)` or `(Class::*name)(args)` after the base type of a function pointer and
    /// returns the pointer-to-function type together with the name.
    ///
    /// `name_error` is the diagnostic for a missing name; `None` accepts an
    /// unnamed declarator, as in a function-pointer parameter.
    /// Parses the parameter list of a function-pointer type-id (already
    /// positioned at the opening `(`).
    fn parse_abstract_fn_params(&mut self) -> Result<Vec<Type>, CompileError> {
        self.advance();
        let mut fn_params = Vec::new();
        while self.peek() != &TokenKind::RParen && self.peek() != &TokenKind::Eof {
            let mut pt = self.parse_type()?;
            if self.peek() == &TokenKind::Ellipsis {
                self.advance();
                pt = Type::PackExpansion(Box::new(pt));
            }
            fn_params.push(pt);
            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(fn_params)
    }

    pub(super) fn parse_function_pointer_declarator(
        &mut self,
        base_type: Type,
        name_error: Option<&str>,
    ) -> Result<(Type, String), CompileError> {
        self.advance();
        let class_name = if self.is_member_pointer_prefix_ahead_from(0) {
            Some(self.parse_member_pointer_class_prefix()?)
        } else {
            self.expect(TokenKind::Star)?;
            None
        };
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
        // `getpmf()` inside `(S::*getpmf())` is the declared function's own
        // parameter list; the suffix after the closing parenthesis belongs to
        // the pointed-to type.
        let mut entity_params: Option<Vec<crate::ast::Param>> = None;
        if self.peek() == &TokenKind::LParen {
            self.advance();
            entity_params = Some(self.parse_function_params()?);
        }
        self.expect(TokenKind::RParen)?;
        // A parenthesized declarator may denote a pointer to function, a
        // pointer to array, or just a grouped pointer. The trailing `)`, `[`,
        // or `(` decides which derivation applies.
        let inner = if self.peek() == &TokenKind::LParen {
            let fn_params = self.parse_abstract_fn_params()?;
            let qualifiers = self.parse_function_qualifiers()?;
            Type::Function {
                return_type: Box::new(base_type),
                param_types: fn_params,
                qualifiers,
            }
        } else if self.peek() == &TokenKind::LBracket {
            self.parse_array_suffix(base_type)?
        } else {
            base_type
        };
        let final_type = match class_name {
            Some(cls) => Type::MemberPointer {
                class_name: cls,
                member_type: Box::new(inner),
            },
            None => Type::Pointer(Box::new(inner)),
        };
        let entity_type = match entity_params {
            Some(params) => Type::Function {
                return_type: Box::new(final_type),
                param_types: params.iter().map(|p| p.ty.clone()).collect(),
                qualifiers: FunctionQualifiers::default(),
            },
            None => final_type,
        };
        Ok((entity_type, name))
    }

    pub(super) fn parse_function_qualifiers(&mut self) -> Result<FunctionQualifiers, CompileError> {
        let mut quals = FunctionQualifiers::default();
        while matches!(
            self.peek(),
            TokenKind::KwConst
                | TokenKind::KwOverride
                | TokenKind::Amp
                | TokenKind::AndAnd
                | TokenKind::KwThrow
        ) || matches!(self.peek(), TokenKind::Ident(id) if id == "noexcept" || id == "volatile" || id == "final")
        {
            match self.peek().clone() {
                TokenKind::KwConst => {
                    self.advance();
                    quals.is_const = true;
                }
                TokenKind::Amp => {
                    self.advance();
                    quals.ref_qualifier = RefQualifier::LValue;
                }
                TokenKind::AndAnd => {
                    self.advance();
                    quals.ref_qualifier = RefQualifier::RValue;
                }
                TokenKind::KwOverride => {
                    self.advance();
                }
                TokenKind::Ident(id) if id == "volatile" => {
                    self.advance();
                    quals.is_volatile = true;
                }
                TokenKind::Ident(id) if id == "final" => {
                    self.advance();
                }
                TokenKind::KwThrow | TokenKind::Ident(_) => {
                    let is_noexcept =
                        matches!(&self.peek(), TokenKind::Ident(id) if id == "noexcept");
                    let is_throw = matches!(self.peek(), TokenKind::KwThrow);
                    self.advance();
                    let mut value = true;
                    if self.peek() == &TokenKind::LParen {
                        self.advance();
                        if is_throw {
                            // Dynamic exception specifications are not a type
                            // qualifier; skip their (possibly dependent) list.
                            while self.peek() != &TokenKind::RParen
                                && self.peek() != &TokenKind::Eof
                            {
                                self.advance();
                            }
                        } else if self.peek() != &TokenKind::RParen
                            && let Expr::BoolLiteral(b) = self.parse_expr()?
                        {
                            value = b;
                        }
                        self.expect(TokenKind::RParen)?;
                    }
                    if is_noexcept {
                        quals.is_noexcept = value;
                    }
                }
                _ => break,
            }
        }

        Ok(quals)
    }

    pub(super) fn parse_ctor_initializers(
        &mut self,
    ) -> Result<Vec<(String, Vec<Expr>)>, CompileError> {
        let mut ctor_initializers = Vec::new();
        if self.peek() == &TokenKind::Colon {
            self.advance();
            loop {
                let mut iname = match self.advance().kind.clone() {
                    TokenKind::Ident(id) => id,
                    _ => {
                        return Err(CompileError::new(
                            "Expected member name in initializer list",
                            self.current().line,
                            self.current().col,
                        ));
                    }
                };
                while self.peek() == &TokenKind::ColonColon {
                    self.advance();
                    if let TokenKind::Ident(ref sub) = self.advance().kind {
                        iname = format!("{}::{}", iname, sub);
                    }
                }
                if self.peek() == &TokenKind::Lt {
                    let args = self.parse_template_args()?;
                    let canonical = args
                        .iter()
                        .map(|a| a.to_c_type())
                        .collect::<Vec<_>>()
                        .join(", ");
                    iname = format!("{}<{}>", iname, canonical);
                }
                let args = if self.peek() == &TokenKind::LBrace {
                    self.parse_brace_expr_list()?.0
                } else {
                    self.expect(TokenKind::LParen)?;
                    self.parse_paren_expr_list()?
                };
                if self.peek() == &TokenKind::Ellipsis {
                    self.advance();
                    iname.push_str("...");
                }
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
