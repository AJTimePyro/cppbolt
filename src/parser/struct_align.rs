use super::*;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::lexer::TokenKind;
use std::collections::HashMap;

pub(super) enum AlignasSpec {
    Dependent(String),
    Concrete(usize),
}

impl AlignasSpec {
    pub fn to_type(&self) -> Type {
        match self {
            AlignasSpec::Dependent(pack) => {
                Type::DependentAlignas(Box::new(Type::Void), pack.clone())
            }
            AlignasSpec::Concrete(align) => {
                Type::Aligned(Box::new(Type::Void), *align)
            }
        }
    }

    pub fn wrap_type(self, ty: Type) -> Type {
        match self {
            AlignasSpec::Dependent(pack) => Type::DependentAlignas(Box::new(ty), pack),
            AlignasSpec::Concrete(align) => Type::Aligned(Box::new(ty), align),
        }
    }
}

impl Parser {
    pub(super) fn is_struct_after_alignas(&self) -> bool {
        if self.peek() != &TokenKind::KwAlignas {
            return false;
        }
        let mut depth = 0;
        let mut idx = 1;
        while let Some(tok) = self.tokens.get(self.pos + idx) {
            match tok.kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        idx += 1;
                        if let Some(next) = self.tokens.get(self.pos + idx) {
                            return matches!(next.kind, TokenKind::KwStruct | TokenKind::KwClass);
                        }
                        return false;
                    }
                }
                _ => {}
            }
            idx += 1;
        }
        false
    }

    pub(super) fn parse_alignas_spec(&mut self) -> Result<AlignasSpec, CompileError> {
        self.expect(TokenKind::KwAlignas)?;
        self.expect(TokenKind::LParen)?;

        if self.peek() == &TokenKind::KwAlignof
            && self.peek_ahead(1) == &TokenKind::LParen
            && let TokenKind::Ident(id) = self.peek_ahead(2)
            && self.peek_ahead(3) == &TokenKind::RParen
            && self.peek_ahead(4) == &TokenKind::Ellipsis
        {
            let pack = id.clone();
            for _ in 0..5 {
                self.advance();
            }
            self.expect(TokenKind::RParen)?;
            return Ok(AlignasSpec::Dependent(pack));
        }

        if let TokenKind::Ident(id) = self.peek() {
            let is_ellipsis = self.peek_ahead(1) == &TokenKind::Ellipsis;
            if is_ellipsis || self.is_active_type_param(id) {
                let pack = id.clone();
                self.advance();
                if self.peek() == &TokenKind::Ellipsis {
                    self.advance();
                }
                self.expect(TokenKind::RParen)?;
                return Ok(AlignasSpec::Dependent(pack));
            }
        }

        if self.is_type_start() {
            let ty = self.parse_type()?;
            self.expect(TokenKind::RParen)?;
            let align = ty.align_of(&HashMap::new());
            return Ok(AlignasSpec::Concrete(align));
        }

        let expr = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        let align = match crate::sema::constexpr_eval::eval_integral_constexpr(&expr, None) {
            Ok(v) if v > 0 => v as usize,
            _ => 1,
        };
        Ok(AlignasSpec::Concrete(align))
    }
}
