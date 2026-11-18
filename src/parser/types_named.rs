//! Named-type and decltype resolution helpers.

use super::*;

impl Parser {
    pub(super) fn parse_decltype_specifier(&mut self) -> Result<Type, CompileError> {
        self.advance();
        self.expect(TokenKind::LParen)?;
        let res = if self.peek() == &TokenKind::KwNullptr {
            self.advance();
            Type::Nullptr
        } else {
            let expr = self.parse_expr()?;
            if let Expr::Unary {
                op: UnaryOp::AddrOf,
                expr: inner,
                ..
            } = &expr
                && let Expr::Identifier(name) = &**inner
                && let Some(member_ty) = self.member_types.get(name)
            {
                let (class_name, _) = name.rsplit_once("::").unwrap_or(("", ""));
                Type::MemberPointer {
                    class_name: class_name.to_string(),
                    member_type: Box::new(member_ty.clone()),
                }
            } else {
                Type::Pointer(Box::new(Type::Void))
            }
        };
        self.expect(TokenKind::RParen)?;
        Ok(res)
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
