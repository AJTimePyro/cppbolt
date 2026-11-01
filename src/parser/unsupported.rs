use super::*;

impl Parser {
    pub fn validate_supported_features(&self) -> Result<(), CompileError> {
        if let Some(error) = self.reject_const_overloading() {
            return Err(error);
        }
        Ok(())
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
