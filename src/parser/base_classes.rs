//! Base-clause parsing for structs and classes.

use super::*;

impl Parser {
    fn parse_base_access(&mut self, is_class: bool) -> Result<BaseAccess, CompileError> {
        let mut access = if is_class {
            BaseAccess::Private
        } else {
            BaseAccess::Public
        };
        while matches!(
            self.peek(),
            TokenKind::KwPublic
                | TokenKind::KwPrivate
                | TokenKind::KwProtected
                | TokenKind::KwVirtual
        ) {
            if self.peek() == &TokenKind::KwVirtual {
                return Err(CompileError::new(
                    "virtual inheritance is not supported",
                    self.current().line,
                    self.current().col,
                ));
            }
            access = match self.peek() {
                TokenKind::KwPublic => BaseAccess::Public,
                TokenKind::KwProtected => BaseAccess::Protected,
                _ => BaseAccess::Private,
            };
            self.advance();
        }
        Ok(access)
    }

    fn parse_single_base_name(&mut self) -> Result<String, CompileError> {
        match self.advance().kind.clone() {
            TokenKind::Ident(mut base) => {
                while self.peek() == &TokenKind::ColonColon {
                    self.advance();
                    if let TokenKind::Ident(ref sub) = self.advance().kind {
                        base = format!("{}::{}", base, sub);
                    }
                }
                if self.peek() == &TokenKind::Lt {
                    let args = self.parse_template_args()?;
                    let canonical = args
                        .iter()
                        .map(|a| a.to_c_type())
                        .collect::<Vec<_>>()
                        .join(", ");
                    base = format!("{}<{}>", base, canonical);
                }
                if self.peek() == &TokenKind::Ellipsis {
                    self.advance();
                    base.push_str("...");
                }
                Ok(base)
            }
            _ => Err(CompileError::new(
                "Expected base class name",
                self.current().line,
                self.current().col,
            )),
        }
    }

    pub(super) fn parse_base_classes(
        &mut self,
        is_class: bool,
    ) -> Result<Vec<(String, BaseAccess)>, CompileError> {
        let mut base_classes = Vec::new();
        if self.peek() == &TokenKind::Colon {
            self.advance();
            loop {
                let access = self.parse_base_access(is_class)?;
                let base_token = self.current().clone();
                let base = self.parse_single_base_name()?;
                if base_classes.iter().any(|(name, _)| name == &base) {
                    return Err(CompileError::new(
                        format!(
                            "base class '{base}' specified more than once as a direct base class"
                        ),
                        base_token.line,
                        base_token.col,
                    ));
                }
                base_classes.push((base, access));
                if self.peek() == &TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        Ok(base_classes)
    }
}
