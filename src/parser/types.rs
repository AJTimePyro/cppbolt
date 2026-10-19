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
            | TokenKind::KwDecltype => true,
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
                if self.peek_ahead(1) == &TokenKind::ColonColon {
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
                            || self.type_aliases.contains_key(sub)
                            || self.type_aliases.contains_key(&full)
                    } else {
                        false
                    }
                } else {
                    id == "nullptr_t"
                        || id == "size_t"
                        || id == "ptrdiff_t"
                        || self.known_types.contains(id)
                        || self.type_aliases.contains_key(id)
                }
            }
            _ => false,
        }
    }

    pub fn parse_type(&mut self) -> Result<Type, CompileError> {
        let mut is_const = false;
        while self.peek() == &TokenKind::KwConst {
            self.advance();
            is_const = true;
        }

        let mut base_type = match self.peek() {
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
            TokenKind::KwDecltype => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                if self.peek() == &TokenKind::KwNullptr {
                    self.advance();
                } else {
                    let _ = self.parse_expr()?;
                }
                self.expect(TokenKind::RParen)?;
                Type::Pointer(Box::new(Type::Void))
            }
            TokenKind::ColonColon | TokenKind::Ident(_) => {
                let id = self.parse_qualified_name()?;
                if id == "size_t" {
                    Type::UnsignedInt
                } else if id == "ptrdiff_t" {
                    Type::Int
                } else if let Some(alias) = self.type_aliases.get(&id).cloned() {
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
                } else if QualifiedName::new(&id).in_std() {
                    let plain = QualifiedName::new(&id).plain();
                    if self.known_types.contains(plain) {
                        Type::Struct(plain.to_string())
                    } else {
                        return Err(CompileError::new(
                            format!("Unknown type '{}'", id),
                            self.current().line,
                            self.current().col,
                        ));
                    }
                } else if self.known_types.contains(&format!("std::{}", id)) {
                    Type::Struct(id)
                } else {
                    return Err(CompileError::new(
                        format!("Unknown type '{}'", id),
                        self.current().line,
                        self.current().col,
                    ));
                }
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

        // Pointer / Reference qualifiers
        loop {
            if self.peek() == &TokenKind::KwConst {
                self.advance();
                base_type = Type::Const(Box::new(base_type));
                continue;
            }
            if self.peek() == &TokenKind::Star {
                self.advance();
                base_type = Type::Pointer(Box::new(base_type));
            } else if self.peek() == &TokenKind::Amp {
                self.advance();
                base_type = Type::Reference(Box::new(base_type));
            } else {
                break;
            }
        }
        Ok(base_type)
    }

    pub(super) fn parse_array_suffix(&mut self, elem_type: Type) -> Result<Type, CompileError> {
        let mut dimensions = Vec::new();
        while self.peek() == &TokenKind::LBracket {
            self.advance();
            let size = if self.peek() != &TokenKind::RBracket {
                let expr = self.parse_expr()?;
                match expr {
                    Expr::IntLiteral(n, _) if n >= 0 => n as usize,
                    _ => {
                        return Err(CompileError::new(
                            "Array dimension must be non-negative integer constant",
                            self.current().line,
                            self.current().col,
                        ));
                    }
                }
            } else {
                0
            };
            self.expect(TokenKind::RBracket)?;
            dimensions.push(size);
        }
        // C declarators nest left to right, so `int m[2][3]` is two rows of
        // three ints: wrap the element type from the innermost dimension out.
        let mut cur_type = elem_type;
        for size in dimensions.into_iter().rev() {
            cur_type = Type::Array(Box::new(cur_type), size);
        }
        Ok(cur_type)
    }
}
