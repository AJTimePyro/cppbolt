use super::*;

impl Parser {
    pub(super) fn parse_struct(&mut self) -> Result<Decl, CompileError> {
        let start = self.current().clone();
        let (name, is_class) = self.parse_struct_name()?;

        if self.peek() == &TokenKind::Semi {
            self.advance();
            return Ok(
                StructBody::new(name, is_class, start.line, start.col).into_decl(Vec::new())
            );
        }

        let base_classes = self.parse_base_classes()?;
        self.expect(TokenKind::LBrace)?;

        let mut body = StructBody::new(name, is_class, start.line, start.col);
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            self.parse_struct_member(&mut body)?;
        }

        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::Semi)?;

        Ok(body.into_decl(base_classes))
    }

    fn parse_struct_name(&mut self) -> Result<(String, bool), CompileError> {
        let is_class = self.current().kind == TokenKind::KwClass;
        self.advance();
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
        Ok((name, is_class))
    }

    fn parse_base_classes(&mut self) -> Result<Vec<String>, CompileError> {
        let mut base_classes = Vec::new();
        if self.peek() == &TokenKind::Colon {
            self.advance();
            loop {
                loop {
                    if self.peek() == &TokenKind::KwVirtual {
                        let token = self.current();
                        return Err(CompileError::new(
                            "virtual inheritance is not supported",
                            token.line,
                            token.col,
                        ));
                    }
                    if matches!(
                        self.peek(),
                        TokenKind::KwPublic | TokenKind::KwPrivate | TokenKind::KwProtected
                    ) {
                        self.advance();
                    } else {
                        break;
                    }
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
        Ok(base_classes)
    }

    fn parse_struct_member(&mut self, body: &mut StructBody) -> Result<(), CompileError> {
        if self.parse_struct_access(body)? {
            return Ok(());
        }
        if self.parse_struct_friend()? {
            return Ok(());
        }
        if self.parse_struct_typedef(body)? {
            return Ok(());
        }
        if self.parse_struct_using(body)? {
            return Ok(());
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

        if self.parse_struct_special_member(body, &start_tok, is_virtual, is_static)? {
            return Ok(());
        }

        self.parse_struct_function_or_field(body, start_tok, is_virtual, is_static)
    }
}

struct StructBody {
    name: String,
    is_class: bool,
    fields: Vec<(String, Type)>,
    methods: Vec<FunctionDecl>,
    static_fields: Vec<(String, Type, Option<Expr>)>,
    private_members: Vec<String>,
    is_private: bool,
    line: usize,
    col: usize,
}

impl StructBody {
    fn new(name: String, is_class: bool, line: usize, col: usize) -> Self {
        Self {
            name,
            is_class,
            fields: Vec::new(),
            methods: Vec::new(),
            static_fields: Vec::new(),
            private_members: Vec::new(),
            is_private: is_class,
            line,
            col,
        }
    }

    fn into_decl(self, base_classes: Vec<String>) -> Decl {
        Decl::StructDecl {
            name: self.name,
            is_class: self.is_class,
            base_classes,
            fields: self.fields,
            methods: self.methods,
            static_fields: self.static_fields,
            private_members: self.private_members,
            line: self.line,
            col: self.col,
        }
    }
}

impl Parser {
    fn parse_struct_access(&mut self, body: &mut StructBody) -> Result<bool, CompileError> {
        // Public/private/protected labels
        if self.peek() == &TokenKind::KwPublic {
            self.advance();
            self.expect(TokenKind::Colon)?;
            body.is_private = false;
            return Ok(true);
        }
        if matches!(self.peek(), TokenKind::KwPrivate | TokenKind::KwProtected) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            body.is_private = true;
            return Ok(true);
        }
        Ok(false)
    }

    fn parse_struct_friend(&mut self) -> Result<bool, CompileError> {
        if self.peek() == &TokenKind::KwFriend {
            self.advance();
            while self.peek() != &TokenKind::Semi && self.peek() != &TokenKind::Eof {
                self.advance();
            }
            if self.peek() == &TokenKind::Semi {
                self.advance();
            }
            return Ok(true);
        }
        Ok(false)
    }

    fn parse_struct_typedef(&mut self, body: &mut StructBody) -> Result<bool, CompileError> {
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
                .insert(format!("{}::{}", body.name, tname), ty.clone());
            self.type_aliases.insert(tname.clone(), ty);
            self.known_types.insert(format!("{}::{}", body.name, tname));
            self.known_types.insert(tname);
            return Ok(true);
        }
        Ok(false)
    }

    fn parse_struct_using(&mut self, body: &mut StructBody) -> Result<bool, CompileError> {
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
                    .insert(format!("{}::{}", body.name, alias), ty.clone());
                self.type_aliases.insert(alias.clone(), ty);
                self.known_types.insert(format!("{}::{}", body.name, alias));
                self.known_types.insert(alias);
                return Ok(true);
            } else {
                self.expect(TokenKind::Semi)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Parses the conversion operator, destructor, and constructor forms of a
    /// class member. Returns `true` when one of them was parsed.
    fn parse_struct_special_member(
        &mut self,
        body: &mut StructBody,
        start_tok: &Token,
        is_virtual: bool,
        is_static: bool,
    ) -> Result<bool, CompileError> {
        if self.peek() == &TokenKind::KwOperator && self.peek_ahead(1) == &TokenKind::KwBool {
            self.advance();
            self.advance();
            let fname = "operator bool".to_string();
            let mut func =
                self.parse_function_tail(fname.clone(), Type::Bool, start_tok.line, start_tok.col)?;
            func.is_virtual = is_virtual;
            func.is_static = is_static;
            if body.is_private {
                body.private_members.push(fname);
            }
            body.methods.push(func);
            return Ok(true);
        }

        // Destructor: ~ClassName()
        if self.peek() == &TokenKind::Tilde {
            self.advance();
            let dname = match self.advance().kind.clone() {
                TokenKind::Ident(id) => format!("~{}", id),
                _ => {
                    return Err(CompileError::new(
                        "Expected destructor body.name after ~",
                        start_tok.line,
                        start_tok.col,
                    ));
                }
            };
            let mut dtor =
                self.parse_function_tail(dname, Type::Void, start_tok.line, start_tok.col)?;
            dtor.is_virtual = is_virtual;
            body.methods.push(dtor);
            return Ok(true);
        }

        // Constructor: ClassName(...)
        let short_name = body.name.split("::").last().unwrap_or(&body.name);
        if let TokenKind::Ident(id) = self.peek().clone()
            && (id == body.name || id == short_name)
            && self.peek_ahead(1) == &TokenKind::LParen
        {
            self.advance();
            let mut ctor = self.parse_function_tail(
                short_name.to_string(),
                Type::Void,
                start_tok.line,
                start_tok.col,
            )?;
            ctor.is_virtual = is_virtual;
            body.methods.push(ctor);
            return Ok(true);
        }

        Ok(false)
    }

    fn parse_struct_function_or_field(
        &mut self,
        body: &mut StructBody,
        start_tok: Token,
        is_virtual: bool,
        is_static: bool,
    ) -> Result<(), CompileError> {
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

            // A parameter list after the body.name marks a member function;
            // otherwise this is a field declaration.
            if self.peek() == &TokenKind::LParen {
                let mut func =
                    self.parse_function_tail(fname.clone(), ty, start_tok.line, start_tok.col)?;
                func.is_virtual = is_virtual;
                func.is_static = is_static;
                if body.is_private {
                    body.private_members.push(fname);
                }
                body.methods.push(func);
            } else {
                self.parse_struct_field(body, fname, &ty, is_static, &start_tok)?;
            }
        } else {
            return Err(CompileError::new(
                "Unexpected token in struct body",
                self.current().line,
                self.current().col,
            ));
        }
        Ok(())
    }
    /// Parses a field declaration of a class, including comma-separated
    /// declarators and static fields with initializers.
    fn parse_struct_field(
        &mut self,
        body: &mut StructBody,
        fname: String,
        ty: &Type,
        is_static: bool,
        start_tok: &Token,
    ) -> Result<(), CompileError> {
        let final_ty = self.parse_array_suffix(ty.clone())?;
        if is_static {
            let init = if self.peek() == &TokenKind::Assign {
                self.advance();
                Some(self.parse_initializer()?)
            } else {
                None
            };
            self.expect(TokenKind::Semi)?;
            if body.is_private {
                body.private_members.push(fname.clone());
            }
            body.static_fields.push((fname, final_ty, init));
        } else {
            if body.is_private {
                body.private_members.push(fname.clone());
            }
            body.fields.push((fname, final_ty));
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
                if body.is_private {
                    body.private_members.push(next_fname.clone());
                }
                body.fields.push((next_fname, next_ty));
            }
            self.expect(TokenKind::Semi)?;
        }
        Ok(())
    }
}
