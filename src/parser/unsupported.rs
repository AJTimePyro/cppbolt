use super::*;

impl Parser {
    pub fn validate_supported_features(&self) -> Result<(), CompileError> {
        if let Some(error) = self.reject_operator_overloads() {
            return Err(error);
        }
        if let Some(error) = self.reject_namespace_aliases() {
            return Err(error);
        }
        if let Some(error) = self.reject_const_overloading() {
            return Err(error);
        }
        Ok(())
    }

    fn reject_operator_overloads(&self) -> Option<CompileError> {
        if let Some(window) = self.tokens.windows(5).find(|window| {
            matches!(window[0].kind, TokenKind::KwOperator)
                && matches!(window[1].kind, TokenKind::PlusPlus | TokenKind::MinusMinus)
                && matches!(window[2].kind, TokenKind::LParen)
                && matches!(window[3].kind, TokenKind::KwInt)
                && matches!(window[4].kind, TokenKind::RParen)
        }) {
            return Some(
                CompileError::new(
                    "postfix ++/-- is not supported",
                    window[0].line,
                    window[0].col,
                )
                .with_kind(ErrorKind::Unsupported),
            );
        }
        None
    }

    fn reject_namespace_aliases(&self) -> Option<CompileError> {
        for window in self.tokens.windows(3) {
            if matches!(window[0].kind, TokenKind::KwNamespace)
                && matches!(window[1].kind, TokenKind::Ident(_))
                && matches!(window[2].kind, TokenKind::Assign)
            {
                return Some(
                    CompileError::new(
                        "namespace aliases are not supported",
                        window[2].line,
                        window[2].col,
                    )
                    .with_kind(ErrorKind::Unsupported),
                );
            }
        }
        None
    }

    fn reject_const_overloading(&self) -> Option<CompileError> {
        for window in self.tokens.windows(3) {
            if matches!(window[0].kind, TokenKind::RParen)
                && matches!(window[1].kind, TokenKind::KwConst)
                && matches!(window[2].kind, TokenKind::Amp)
            {
                return Some(
                    CompileError::new(
                        "const overloading is not supported",
                        window[1].line,
                        window[1].col,
                    )
                    .with_kind(ErrorKind::Unsupported),
                );
            }
        }
        None
    }
}
