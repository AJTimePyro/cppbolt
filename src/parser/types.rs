use super::*;

impl Parser {
    pub(super) fn is_type_start(&self) -> bool {
        match self.peek() {
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
            | TokenKind::KwAuto
            | TokenKind::KwConst
            | TokenKind::KwConstexpr
            | TokenKind::KwDecltype
            | TokenKind::KwTypename
            | TokenKind::KwAlignas => true,
            TokenKind::ColonColon => {
                if let TokenKind::Ident(sub) = self.peek_ahead(1) {
                    sub == "size_t"
                        || sub == "ptrdiff_t"
                        || self.known_types.contains(sub)
                        || self.type_aliases.contains_key(sub)
                } else {
                    false
                }
            }
            TokenKind::Ident(id) => {
                if self.allow_unknown_types {
                    true
                } else if self.peek_ahead(1) == &TokenKind::ColonColon {
                    if let TokenKind::Ident(sub) = self.peek_ahead(2) {
                        let full = format!("{}::{}", id, sub);
                        sub == "size_t"
                            || sub == "ptrdiff_t"
                            || sub == "nullptr_t"
                            || sub == "string"
                            || sub == "ostream"
                            || sub == "istream"
                            || self.known_types.contains(sub)
                            || self.known_types.contains(&full)
                            || self.known_templates.contains(sub)
                            || self.known_templates.contains(&full)
                            || self.type_aliases.contains_key(sub)
                            || self.type_aliases.contains_key(&full)
                            || self.is_active_type_param(id)
                    } else {
                        false
                    }
                } else {
                    id == "nullptr_t"
                        || id == "size_t"
                        || id == "ptrdiff_t"
                        || self.known_types.contains(id)
                        || self.is_known_template(id)
                        || self.is_active_type_param(id)
                        || self.type_aliases.contains_key(id)
                        || self
                            .active_struct_scopes
                            .iter()
                            .rev()
                            .any(|s| self.known_types.contains(&format!("{}::{}", s, id)))
                }
            }
            _ => false,
        }
    }

    pub fn parse_type(&mut self) -> Result<Type, CompileError> {
        let _guard = self.enter_type()?;
        let base_type = self.parse_specifier_type_inner()?;
        self.parse_declarator_prefix(&base_type)
    }
    /// Parses a complete type-id, including parenthesized function- and
    /// member-function-pointer declarators (`int (S::*)(int) const`).
    pub(super) fn parse_type_id(&mut self) -> Result<Type, CompileError> {
        let _guard = self.enter_type()?;
        let base_type = self.parse_specifier_type_inner()?;
        if self.is_function_pointer_declarator_ahead() {
            return Ok(self.parse_function_pointer_declarator(base_type, None)?.0);
        }
        self.parse_declarator_prefix(&base_type)
    }

    /// Parses the type specifiers of a declaration: the builtin or named type
    /// with its `const` qualifiers.
    ///
    /// Pointer, reference and array suffixes bind to an individual declarator
    /// name, so `parse_declarator` applies them separately. That is what makes
    /// `int *a, b;` declare `a` as `int *` and `b` as `int`.
    pub(super) fn parse_specifier_type(&mut self) -> Result<Type, CompileError> {
        let _guard = self.enter_type()?;
        self.parse_specifier_type_inner()
    }

    fn parse_specifier_type_inner(&mut self) -> Result<Type, CompileError> {
        let mut is_const = false;
        let mut is_typename = false;
        let mut alignas_specs = Vec::new();
        loop {
            if self.peek() == &TokenKind::KwConst || self.peek() == &TokenKind::KwConstexpr {
                self.advance();
                is_const = true;
            } else if matches!(self.peek(), TokenKind::Ident(id) if id == "volatile") {
                // Volatile is not a distinct Wasm type; top-level cv is
                // ignored in function parameter types per C++17.
                self.advance();
            } else if self.peek() == &TokenKind::KwTypename {
                self.advance();
                is_typename = true;
            } else if self.peek() == &TokenKind::KwAlignas {
                alignas_specs.push(self.parse_alignas_spec()?);
            } else {
                break;
            }
        }

        if is_typename && !matches!(self.peek(), TokenKind::ColonColon | TokenKind::Ident(_)) {
            return Err(CompileError::new(
                "'typename' must be followed by a qualified name",
                self.current().line,
                self.current().col,
            ));
        }

        let mut base_type = match self.peek() {
            TokenKind::KwVoid
            | TokenKind::KwBool
            | TokenKind::KwChar
            | TokenKind::KwInt
            | TokenKind::KwShort
            | TokenKind::KwLong
            | TokenKind::KwFloat
            | TokenKind::KwDouble
            | TokenKind::KwAuto
            | TokenKind::KwSigned
            | TokenKind::KwUnsigned => self.parse_builtin_type()?,
            TokenKind::KwDecltype | TokenKind::ColonColon | TokenKind::Ident(_) => {
                self.parse_named_type(is_typename)?
            }
            _ => {
                return Err(CompileError::new(
                    format!(
                        "Expected type specifier, found {}",
                        self.peek().diagnostic_name()
                    ),
                    self.current().line,
                    self.current().col,
                ));
            }
        };

        if is_const {
            base_type = Type::Const(Box::new(base_type));
        }

        loop {
            if self.peek() == &TokenKind::KwConst || self.peek() == &TokenKind::KwConstexpr {
                self.advance();
                base_type = Type::Const(Box::new(base_type));
            } else {
                break;
            }
        }
        for spec in alignas_specs {
            base_type = spec.wrap_type(base_type);
        }
        Ok(base_type)
    }

    /// Parses one declarator: the pointer, reference and `const` prefixes that
    /// bind to the name, the name, and its array suffix.
    ///
    /// `specifiers` is the declaration's specifier type, so every declarator of
    /// a declaration starts from the same base type.
    pub(super) fn parse_declarator(
        &mut self,
        specifiers: &Type,
        name_error: &str,
    ) -> Result<(Type, String), CompileError> {
        let ty = self.parse_declarator_prefix(specifiers)?;
        if self.is_function_pointer_declarator_ahead() {
            return self.parse_function_pointer_declarator(ty, Some(name_error));
        }
        let tok = self.advance().clone();
        let name = match tok.kind {
            TokenKind::Ident(id) => id,
            _ => {
                return Err(CompileError::new(name_error, tok.line, tok.col));
            }
        };
        Ok((self.parse_array_suffix(ty)?, name))
    }

    /// Parses the pointer, reference and `const` prefixes of one declarator.
    pub(super) fn parse_declarator_prefix(
        &mut self,
        specifiers: &Type,
    ) -> Result<Type, CompileError> {
        let mut ty = specifiers.clone();
        loop {
            if self.is_member_pointer_prefix_ahead_from(0) {
                let class_name = self.parse_member_pointer_class_prefix()?;
                ty = Type::MemberPointer {
                    class_name,
                    member_type: Box::new(ty),
                };
                continue;
            }
            match self.peek() {
                TokenKind::Star => {
                    self.advance();
                    ty = Type::Pointer(Box::new(ty));
                }
                TokenKind::AndAnd => {
                    self.advance();
                    ty = Type::RvalueReference(Box::new(ty));
                }
                TokenKind::Amp => {
                    self.advance();
                    if self.peek() == &TokenKind::Amp {
                        self.advance();
                        ty = Type::RvalueReference(Box::new(ty));
                    } else {
                        ty = Type::Reference(Box::new(ty));
                    }
                }
                TokenKind::KwConst => {
                    self.advance();
                    ty = Type::Const(Box::new(ty));
                }
                _ => break,
            }
        }
        Ok(ty)
    }

    fn parse_builtin_type(&mut self) -> Result<Type, CompileError> {
        let ty = match self.peek() {
            TokenKind::KwVoid => {
                self.advance();
                Type::Void
            }
            TokenKind::KwBool => {
                self.advance();
                Type::Bool
            }
            TokenKind::KwChar => {
                self.advance();
                Type::Char
            }
            TokenKind::KwInt => {
                self.advance();
                Type::Int
            }
            TokenKind::KwShort => {
                self.advance();
                if self.peek() == &TokenKind::KwInt {
                    self.advance();
                }
                Type::Short
            }
            TokenKind::KwLong => {
                self.advance();
                if self.peek() == &TokenKind::KwLong {
                    self.advance();
                    if self.peek() == &TokenKind::KwInt {
                        self.advance();
                    }
                    Type::LongLong
                } else {
                    if self.peek() == &TokenKind::KwInt {
                        self.advance();
                    }
                    Type::Long
                }
            }
            TokenKind::KwFloat => {
                self.advance();
                Type::Float
            }
            TokenKind::KwDouble => {
                self.advance();
                Type::Double
            }
            TokenKind::KwAuto => {
                self.advance();
                Type::Auto
            }
            TokenKind::KwSigned | TokenKind::KwUnsigned => self.parse_signed_type()?,
            _ => unreachable!(),
        };
        Ok(ty)
    }

    fn parse_signed_type(&mut self) -> Result<Type, CompileError> {
        let ty = match self.peek() {
            TokenKind::KwSigned => {
                self.advance();
                if self.peek() == &TokenKind::KwChar {
                    self.advance();
                    Type::SignedChar
                } else if self.peek() == &TokenKind::KwShort {
                    self.advance();
                    Type::Short
                } else if self.peek() == &TokenKind::KwLong {
                    self.advance();
                    if self.peek() == &TokenKind::KwLong {
                        self.advance();
                        Type::LongLong
                    } else {
                        Type::Long
                    }
                } else {
                    if self.peek() == &TokenKind::KwInt {
                        self.advance();
                    }
                    Type::Int
                }
            }
            TokenKind::KwUnsigned => {
                self.advance();
                if self.peek() == &TokenKind::KwChar {
                    self.advance();
                    Type::UnsignedChar
                } else if self.peek() == &TokenKind::KwShort {
                    self.advance();
                    Type::UnsignedShort
                } else if self.peek() == &TokenKind::KwLong {
                    self.advance();
                    if self.peek() == &TokenKind::KwLong {
                        self.advance();
                        Type::UnsignedLongLong
                    } else {
                        Type::UnsignedLong
                    }
                } else {
                    if self.peek() == &TokenKind::KwInt {
                        self.advance();
                    }
                    Type::UnsignedInt
                }
            }
            _ => unreachable!(),
        };
        Ok(ty)
    }

    fn parse_named_type(&mut self, is_typename: bool) -> Result<Type, CompileError> {
        let ty = match self.peek() {
            TokenKind::KwDecltype => self.parse_decltype_specifier()?,
            TokenKind::ColonColon | TokenKind::Ident(_) => {
                let raw_id = self.parse_qualified_name()?;
                if is_typename && !raw_id.contains("::") {
                    return Err(CompileError::new(
                        "'typename' must be followed by a qualified name",
                        self.current().line,
                        self.current().col,
                    ));
                }
                let id = self.resolve_namespace_alias(&raw_id);
                if (self.is_active_type_param(&id)
                    || self.is_known_template(&id)
                    || self.is_template_member_type(&id)
                    || is_typename
                    || self.allow_unknown_types)
                    && self.peek() == &TokenKind::Lt
                {
                    let args = self.parse_template_args()?;
                    Type::TemplateInstance { name: id, args }
                } else if self.is_active_type_param(&id) {
                    Type::TypeParam(id)
                } else if self.is_known_template(&id) {
                    Type::Struct(id)
                } else if id == "size_t" {
                    Type::UnsignedInt
                } else if id == "ptrdiff_t" {
                    Type::Int
                } else if let Some(scoped) = self.resolve_active_struct_scope_type(&id) {
                    scoped
                } else if let Some(alias) = self
                    .scoped_aliases
                    .iter()
                    .rev()
                    .find_map(|scope| scope.get(&id).cloned())
                    .or_else(|| self.type_aliases.get(&id).cloned())
                {
                    alias
                } else if self.known_enums.contains(&id) {
                    Type::Enum(id)
                } else if !self.current_namespace.is_empty()
                    && self.known_types.contains(&format!(
                        "{}::{}",
                        self.current_namespace.join("::"),
                        id
                    ))
                {
                    Type::Struct(format!("{}::{}", self.current_namespace.join("::"), id))
                } else if let Some(ns) = self
                    .open_namespaces
                    .iter()
                    .find(|ns| self.known_types.contains(&format!("{}::{}", ns, id)))
                {
                    Type::Struct(format!("{}::{}", ns, id))
                } else if self.known_types.contains(&id) {
                    Type::Struct(id)
                } else {
                    self.resolve_named_struct_fallback(
                        &id,
                        is_typename,
                        (self.current().line, self.current().col),
                    )?
                }
            }
            _ => unreachable!(),
        };
        Ok(ty)
    }

    pub(super) fn is_template_member_type(&self, id: &str) -> bool {
        let plain = QualifiedName::new(id).plain();
        if let Some(gt) = plain.rfind('>')
            && plain[gt..].starts_with(">::")
        {
            let base = plain.split('<').next().unwrap_or(plain);
            return self.is_known_template(base);
        }
        if let Some((base, _member)) = plain.split_once("::") {
            return self.is_active_type_param(base) || self.type_aliases.contains_key(base);
        }
        false
    }

    fn resolve_active_struct_scope_type(&self, id: &str) -> Option<Type> {
        for scope in self.active_struct_scopes.iter().rev() {
            let scoped = format!("{}::{}", scope, id);
            if let Some(alias) = self.type_aliases.get(&scoped).cloned() {
                return Some(alias);
            }
            if self.known_types.contains(&scoped) {
                return Some(Type::Struct(scoped));
            }
        }
        None
    }
}
