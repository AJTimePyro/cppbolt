use super::*;

impl Parser {
    pub(super) fn parse_template_template_param(
        &mut self,
        p_line: usize,
        p_col: usize,
        seen_default: &mut bool,
    ) -> Result<TemplateParam, CompileError> {
        self.advance();
        self.expect(TokenKind::Lt)?;
        let _inner_params = self.parse_template_params()?;
        self.active_type_params.pop();
        if self.peek() == &TokenKind::KwClass || self.peek() == &TokenKind::KwTypename {
            self.advance();
        } else {
            return Err(CompileError::new(
                "expected 'class' or 'typename' in template template parameter",
                p_line,
                p_col,
            ));
        }
        let is_pack = self.peek() == &TokenKind::Ellipsis && {
            self.advance();
            true
        };
        let name = if let TokenKind::Ident(id) = self.peek().clone() {
            self.advance();
            id
        } else if is_pack
            || matches!(self.peek(), TokenKind::Comma | TokenKind::Gt | TokenKind::Shr)
        {
            String::new()
        } else {
            return Err(CompileError::new("expected identifier", p_line, p_col));
        };
        if !name.is_empty()
            && let Some(set) = self.active_type_params.last_mut()
        {
            set.insert(name.clone());
        }
        if is_pack && self.peek() == &TokenKind::Assign {
            return Err(CompileError::new(
                "template parameter pack cannot have a default argument",
                self.current().line,
                self.current().col,
            ));
        }
        let default_type = if self.peek() == &TokenKind::Assign {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        if default_type.is_some() {
            *seen_default = true;
        } else if *seen_default && !is_pack {
            return Err(CompileError::new(
                "template parameter without default follows parameter with default",
                p_line,
                p_col,
            ));
        }
        Ok(TemplateParam {
            name,
            default_type,
            param_type: None,
            is_pack,
            line: p_line,
            col: p_col,
        })
    }
}
