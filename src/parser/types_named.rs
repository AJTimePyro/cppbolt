//! Named-type and decltype resolution helpers.

use super::*;

impl Parser {
    pub(super) fn parse_decltype_specifier(&mut self) -> Result<Type, CompileError> {
        self.advance();
        self.expect(TokenKind::LParen)?;
        if self.peek() == &TokenKind::KwNullptr {
            self.advance();
            self.expect(TokenKind::RParen)?;
            return Ok(Type::Nullptr);
        }

        // A second `(` may wrap the whole operand. C++17 gives an
        // unparenthesized id-expression or member access the declared type,
        // so record which form we saw.
        let start = self.pos;
        let mut parenthesized = None;
        if self.peek() == &TokenKind::LParen {
            self.advance();
            if let Ok(expr) = self.parse_expr()
                && self.peek() == &TokenKind::RParen
            {
                self.advance();
                if self.peek() == &TokenKind::RParen {
                    parenthesized = Some(expr);
                }
            }
            if parenthesized.is_none() {
                self.pos = start;
            }
        }
        let is_parenthesized = parenthesized.is_some();
        let expr = match parenthesized {
            Some(expr) => expr,
            None => self.parse_expr()?,
        };
        self.expect(TokenKind::RParen)?;
        Ok(Type::Decltype {
            expr: Box::new(expr),
            parenthesized: is_parenthesized,
        })
    }

    pub(super) fn resolve_named_struct_fallback(
        &self,
        id: &str,
        is_typename: bool,
        pos: (usize, usize),
    ) -> Result<Type, CompileError> {
        let (line, col) = pos;
        if QualifiedName::new(id).in_std() {
            let plain = QualifiedName::new(id).plain();
            if self.known_types.contains(plain) {
                return Ok(Type::Struct(plain.to_string()));
            }
            return Err(CompileError::new(
                format!("Unknown type '{}'", id),
                line,
                col,
            ));
        }
        if self.allow_unknown_types
            || is_typename
            || self.known_types.contains(&format!("std::{}", id))
            || self.is_template_member_type(id)
        {
            return Ok(Type::Struct(id.to_string()));
        }
        Err(CompileError::new(
            format!("Unknown type '{}'", id),
            line,
            col,
        ))
    }
}
