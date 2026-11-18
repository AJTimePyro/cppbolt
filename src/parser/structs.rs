use super::*;

impl Parser {
    pub(super) fn parse_struct(&mut self) -> Result<Decl, CompileError> {
        let mut struct_align = None;
        if self.peek() == &TokenKind::KwAlignas {
            let spec = self.parse_alignas_spec()?;
            struct_align = Some(spec.to_type());
        }
        let start = self.current().clone();
        let (name, is_class, is_final) = self.parse_struct_name(&mut struct_align)?;
        if !self.active_type_params.is_empty() {
            self.register_template_name(&name);
        }

        if self.peek() == &TokenKind::Semi {
            self.advance();
            let mut body = StructBody::new(name.clone(), is_class, is_final, start.line, start.col);
            body.alignment = struct_align;
            return Ok(body.into_decl(Vec::new()));
        }

        let base_classes = self.parse_base_classes(is_class)?;
        self.expect(TokenKind::LBrace)?;

        self.active_struct_scopes.push(name.clone());
        let mut body = StructBody::new(name.clone(), is_class, is_final, start.line, start.col);
        body.alignment = struct_align;
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            self.parse_struct_member(&mut body)?;
        }
        self.active_struct_scopes.pop();

        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::Semi)?;
        for (fname, fty, _) in &body.fields {
            self.member_types
                .insert(format!("{}::{}", name, fname), fty.clone());
            self.member_types
                .entry(fname.clone())
                .or_insert_with(|| fty.clone());
        }

        Ok(body.into_decl(base_classes))
    }

    fn parse_struct_name(
        &mut self,
        struct_align: &mut Option<Type>,
    ) -> Result<(String, bool, bool), CompileError> {
        let is_class = self.current().kind == TokenKind::KwClass;
        self.advance();
        if self.peek() == &TokenKind::KwAlignas {
            let spec = self.parse_alignas_spec()?;
            *struct_align = Some(spec.to_type());
        }
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
        name = self.qualify_name(name);
        if self.peek() == &TokenKind::Lt {
            let args = self.parse_template_args()?;
            let canonical_args = args
                .iter()
                .map(|a| a.to_c_type())
                .collect::<Vec<_>>()
                .join(", ");
            name = format!("{}<{}>", name, canonical_args);
            self.struct_template_args.insert(name.clone(), args);
        }
        let is_final = if matches!(self.peek(), TokenKind::Ident(id) if id == "final") {
            self.advance();
            true
        } else {
            false
        };
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
        Ok((name, is_class, is_final))
    }

    pub(super) fn parse_struct_member(
        &mut self,
        body: &mut StructBody,
    ) -> Result<(), CompileError> {
        if self.parse_struct_access(body)? {
            return Ok(());
        }
        if self.parse_struct_friend()? {
            return Ok(());
        }
        if self.parse_nested_struct_member(body)? {
            return Ok(());
        }
        if self.parse_struct_typedef(body)? {
            return Ok(());
        }
        if self.parse_struct_using(body)? {
            return Ok(());
        }

        let template_params = if self.peek() == &TokenKind::KwTemplate {
            self.advance();
            self.expect(TokenKind::Lt)?;
            let params = self.parse_template_params()?;
            Some(params)
        } else {
            None
        };

        let initial_methods_len = body.methods.len();
        let res = self.parse_struct_member_tail(body);
        if let Some(params) = template_params {
            self.active_type_params.pop();
            if body.methods.len() > initial_methods_len
                && let Some(last) = body.methods.last_mut()
            {
                if last.is_virtual {
                    return Err(CompileError::new(
                        "virtual member function templates are not supported",
                        last.line,
                        last.col,
                    ));
                }
                last.template_params = params;
            }
        }
        res
    }

    fn parse_struct_member_tail(&mut self, body: &mut StructBody) -> Result<(), CompileError> {
        let mut is_explicit = false;
        while matches!(
            self.peek(),
            TokenKind::KwConstexpr | TokenKind::KwInline | TokenKind::KwExplicit
        ) {
            if self.peek() == &TokenKind::KwExplicit {
                is_explicit = true;
            }
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

        if self.parse_struct_special_member(body, &start_tok, is_virtual, is_static, is_explicit)? {
            return Ok(());
        }

        self.parse_struct_function_or_field(body, start_tok, is_virtual, is_static, is_explicit)
    }
}

pub(super) struct StructBody {
    pub(super) name: String,
    pub(super) is_class: bool,
    pub(super) is_final: bool,
    pub(super) fields: Vec<StructFieldDecl>,
    pub(super) methods: Vec<FunctionDecl>,
    pub(super) static_fields: Vec<(String, Type, Option<Expr>)>,
    pub(super) private_members: Vec<String>,
    pub(super) nested_structs: Vec<Decl>,
    pub(super) is_private: bool,
    pub(super) alignment: Option<Type>,
    pub(super) line: usize,
    pub(super) col: usize,
}

impl StructBody {
    pub(super) fn new(
        name: String,
        is_class: bool,
        is_final: bool,
        line: usize,
        col: usize,
    ) -> Self {
        Self {
            name,
            is_class,
            is_final,
            fields: Vec::new(),
            methods: Vec::new(),
            static_fields: Vec::new(),
            private_members: Vec::new(),
            nested_structs: Vec::new(),
            is_private: is_class,
            alignment: None,
            line,
            col,
        }
    }

    pub(super) fn into_decl(self, bases: Vec<(String, BaseAccess)>) -> Decl {
        let (base_classes, base_access) = bases.into_iter().unzip();
        Decl::StructDecl {
            name: self.name,
            is_class: self.is_class,
            is_final: self.is_final,
            base_classes,
            base_access,
            fields: self.fields,
            methods: self.methods,
            static_fields: self.static_fields,
            private_members: self.private_members,
            nested_structs: self.nested_structs,
            alignment: self.alignment,
            line: self.line,
            col: self.col,
        }
    }
}

impl Parser {
    /// Parses the conversion operator, destructor, and constructor forms of a
    /// class member. Returns `true` when one of them was parsed.
    fn parse_struct_special_member(
        &mut self,
        body: &mut StructBody,
        start_tok: &Token,
        is_virtual: bool,
        is_static: bool,
        is_explicit: bool,
    ) -> Result<bool, CompileError> {
        if self.parse_struct_conversion_operator(
            body,
            start_tok,
            is_virtual,
            is_static,
            is_explicit,
        )? {
            return Ok(true);
        }

        if self.peek() == &TokenKind::Tilde {
            self.advance();
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
            dtor.is_virtual |= is_virtual;
            body.methods.push(dtor);
            return Ok(true);
        }

        let base_name = body.name.split('<').next().unwrap_or(&body.name);
        let short_name = base_name.split("::").last().unwrap_or(base_name);
        if let TokenKind::Ident(id) = self.peek().clone()
            && (id == body.name || id == base_name || id == short_name)
            && self.peek_ahead(1) == &TokenKind::LParen
        {
            self.advance();
            let mut ctor = self.parse_function_tail(
                short_name.to_string(),
                Type::Void,
                start_tok.line,
                start_tok.col,
            )?;
            ctor.is_virtual |= is_virtual;
            ctor.is_explicit = is_explicit;
            body.methods.push(ctor);
            return Ok(true);
        }

        Ok(false)
    }

    fn parse_struct_conversion_operator(
        &mut self,
        body: &mut StructBody,
        start_tok: &Token,
        is_virtual: bool,
        is_static: bool,
        is_explicit: bool,
    ) -> Result<bool, CompileError> {
        if self.peek() != &TokenKind::KwOperator {
            return Ok(false);
        }
        let next_tok = self.peek_ahead(1);
        let is_conv_type = matches!(
            next_tok,
            TokenKind::KwInt
                | TokenKind::KwLong
                | TokenKind::KwChar
                | TokenKind::KwBool
                | TokenKind::KwFloat
                | TokenKind::KwDouble
                | TokenKind::KwUnsigned
                | TokenKind::KwSigned
                | TokenKind::KwShort
                | TokenKind::KwVoid
        ) || matches!(next_tok, TokenKind::Ident(id) if self.known_types.contains(id) || self.type_aliases.contains_key(id) || self.is_active_type_param(id));
        if !is_conv_type {
            return Ok(false);
        }
        self.advance();
        let target_type = self.parse_type()?;
        let fname = format!("operator {}", target_type.to_c_type());
        let mut func =
            self.parse_function_tail(fname.clone(), target_type, start_tok.line, start_tok.col)?;
        func.is_virtual |= is_virtual;
        func.is_static = is_static;
        func.is_explicit = is_explicit;
        if body.is_private {
            body.private_members.push(fname);
        }
        body.methods.push(func);
        Ok(true)
    }

    fn parse_struct_function_or_field(
        &mut self,
        body: &mut StructBody,
        start_tok: Token,
        is_virtual: bool,
        is_static: bool,
        is_explicit: bool,
    ) -> Result<(), CompileError> {
        if self.is_type_start() {
            let specifiers = self.parse_specifier_type()?;
            let (ty, fname, is_fn_pointer_field) = if self.is_function_pointer_declarator_ahead() {
                let (t, n) = self.parse_function_pointer_declarator(
                    specifiers.clone(),
                    Some("Expected member identifier"),
                )?;
                (t, n, true)
            } else {
                let ty = self.parse_declarator_prefix(&specifiers)?;
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
                (ty, fname, false)
            };

            // A parameter list after the name marks a member function;
            // otherwise this is a field declaration.
            if !is_fn_pointer_field && self.peek() == &TokenKind::LParen {
                let mut func =
                    self.parse_function_tail(fname.clone(), ty, start_tok.line, start_tok.col)?;
                func.is_virtual |= is_virtual;
                func.is_static = is_static;
                func.is_explicit = is_explicit;
                if body.is_private {
                    body.private_members.push(fname);
                }
                body.methods.push(func);
            } else {
                self.parse_struct_field(body, fname, &ty, &specifiers, is_static)?;
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
    fn parse_optional_member_init(&mut self) -> Result<FieldInit, CompileError> {
        if matches!(self.peek(), TokenKind::Assign | TokenKind::LBrace) {
            let tok = if self.peek() == &TokenKind::Assign {
                self.advance().clone()
            } else {
                self.current().clone()
            };
            Ok(Some((self.parse_initializer()?, tok.line, tok.col)))
        } else {
            Ok(None)
        }
    }

    /// Parses a field declaration of a class, including comma-separated
    /// declarators and static fields with initializers.
    ///
    /// `specifiers` is the declaration's specifier type. Each declarator
    /// applies its own pointer, reference and array suffixes to it, so
    /// `int *p, q;` declares `p` as `int *` and `q` as `int`.
    fn parse_struct_field(
        &mut self,
        body: &mut StructBody,
        fname: String,
        ty: &Type,
        specifiers: &Type,
        is_static: bool,
    ) -> Result<(), CompileError> {
        let final_ty = self.parse_array_suffix(ty.clone())?;
        if body.is_private {
            body.private_members.push(fname.clone());
        }
        if is_static {
            let init = if self.peek() == &TokenKind::Assign {
                self.advance();
                Some(self.parse_initializer()?)
            } else {
                None
            };
            self.expect(TokenKind::Semi)?;
            body.static_fields.push((fname, final_ty, init));
        } else {
            let init = self.parse_optional_member_init()?;
            body.fields.push((fname, final_ty, init));
            while self.peek() == &TokenKind::Comma {
                self.advance();
                let (next_ty, next_fname) =
                    self.parse_declarator(specifiers, "Expected member identifier after comma")?;
                if body.is_private {
                    body.private_members.push(next_fname.clone());
                }
                let next_init = self.parse_optional_member_init()?;
                body.fields.push((next_fname, next_ty, next_init));
            }
            self.expect(TokenKind::Semi)?;
        }
        Ok(())
    }
}
