use super::*;

impl Parser {
    pub fn validate_supported_features(&self) -> Result<(), CompileError> {
        for window in self.tokens.windows(4) {
            if matches!(window[0].kind, TokenKind::KwOperator)
                && matches!(window[1].kind, TokenKind::LParen)
                && matches!(window[2].kind, TokenKind::RParen)
            {
                return Err(CompileError::new(
                    "operator() is not supported",
                    window[0].line,
                    window[0].col,
                )
                .with_kind(ErrorKind::Unsupported));
            }
            if matches!(window[0].kind, TokenKind::KwOperator)
                && matches!(window[1].kind, TokenKind::Arrow)
                && matches!(window[2].kind, TokenKind::LParen)
            {
                return Err(CompileError::new(
                    "operator-> is not supported",
                    window[0].line,
                    window[0].col,
                )
                .with_kind(ErrorKind::Unsupported));
            }
            if matches!(window[0].kind, TokenKind::KwOperator)
                && matches!(window[1].kind, TokenKind::Star)
                && matches!(window[2].kind, TokenKind::LParen)
                && matches!(window[3].kind, TokenKind::RParen)
            {
                return Err(CompileError::new(
                    "operator* is not supported",
                    window[0].line,
                    window[0].col,
                )
                .with_kind(ErrorKind::Unsupported));
            }
        }
        let has_postfix_overload = self.tokens.windows(5).any(|window| {
            matches!(window[0].kind, TokenKind::KwOperator)
                && matches!(window[1].kind, TokenKind::PlusPlus | TokenKind::MinusMinus)
                && matches!(window[2].kind, TokenKind::LParen)
                && matches!(window[3].kind, TokenKind::KwInt)
                && matches!(window[4].kind, TokenKind::RParen)
        });
        if has_postfix_overload
            && let Some(window) = self.tokens.windows(5).find(|window| {
                matches!(window[0].kind, TokenKind::KwOperator)
                    && matches!(window[1].kind, TokenKind::PlusPlus | TokenKind::MinusMinus)
                    && matches!(window[2].kind, TokenKind::LParen)
                    && matches!(window[3].kind, TokenKind::KwInt)
                    && matches!(window[4].kind, TokenKind::RParen)
            })
        {
            return Err(CompileError::new(
                "postfix ++/-- is not supported",
                window[0].line,
                window[0].col,
            )
            .with_kind(ErrorKind::Unsupported));
        }
        for window in self.tokens.windows(3) {
            if matches!(window[0].kind, TokenKind::KwNamespace)
                && matches!(window[1].kind, TokenKind::Ident(_))
                && matches!(window[2].kind, TokenKind::Assign)
            {
                return Err(CompileError::new(
                    "namespace aliases are not supported",
                    window[2].line,
                    window[2].col,
                )
                .with_kind(ErrorKind::Unsupported));
            }
        }
        for window in self.tokens.windows(3) {
            if matches!(window[0].kind, TokenKind::RParen)
                && matches!(window[1].kind, TokenKind::KwConst)
                && matches!(window[2].kind, TokenKind::Amp)
            {
                return Err(CompileError::new(
                    "const overloading is not supported",
                    window[1].line,
                    window[1].col,
                )
                .with_kind(ErrorKind::Unsupported));
            }
        }
        for (index, window) in self.tokens.windows(2).enumerate() {
            if matches!(window[0].kind, TokenKind::KwFor)
                && matches!(window[1].kind, TokenKind::LParen)
            {
                let mut depth = 0usize;
                let mut has_semicolon = false;
                for token in self.tokens.iter().skip(index + 1) {
                    match token.kind {
                        TokenKind::LParen => depth += 1,
                        TokenKind::RParen => {
                            depth = depth.saturating_sub(1);
                            if depth == 0 {
                                break;
                            }
                        }
                        TokenKind::Semi if depth == 1 => has_semicolon = true,
                        TokenKind::Colon if depth == 1 && !has_semicolon => {
                            return Err(CompileError::new(
                                "range-based for is not supported",
                                token.line,
                                token.col,
                            )
                            .with_kind(ErrorKind::Unsupported));
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}
