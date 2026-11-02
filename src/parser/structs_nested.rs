use super::structs::StructBody;
use super::*;

impl Parser {
    pub(super) fn parse_nested_struct_member(
        &mut self,
        body: &mut StructBody,
    ) -> Result<bool, CompileError> {
        if self.peek() != &TokenKind::KwStruct && self.peek() != &TokenKind::KwClass {
            return Ok(false);
        }
        let decl = self.parse_nested_struct(body)?;
        body.nested_structs.push(decl);
        Ok(true)
    }

    fn parse_nested_struct(&mut self, body: &mut StructBody) -> Result<Decl, CompileError> {
        let start = self.current().clone();
        let is_class = self.current().kind == TokenKind::KwClass;
        self.advance();
        let short_name = match self.advance().kind.clone() {
            TokenKind::Ident(id) => id,
            _ => {
                return Err(CompileError::new(
                    "Expected struct/class name",
                    self.current().line,
                    self.current().col,
                ));
            }
        };
        let full_name = format!("{}::{}", body.name, short_name);
        self.known_types.insert(full_name.clone());
        if body.is_private {
            body.private_members.push(short_name);
            body.private_members.push(full_name.clone());
        }

        if self.peek() == &TokenKind::Semi {
            self.advance();
            return Ok(StructBody::new(full_name, is_class, start.line, start.col).into_decl(Vec::new()));
        }

        let base_classes = self.parse_base_classes()?;
        self.expect(TokenKind::LBrace)?;

        self.active_struct_scopes.push(full_name.clone());
        let mut inner_body = StructBody::new(full_name, is_class, start.line, start.col);
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            self.parse_struct_member(&mut inner_body)?;
        }
        self.active_struct_scopes.pop();

        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::Semi)?;

        Ok(inner_body.into_decl(base_classes))
    }

    pub(super) fn parse_struct_typedef(
        &mut self,
        body: &mut StructBody,
    ) -> Result<bool, CompileError> {
        if self.peek() == &TokenKind::KwTypedef {
            self.advance();
            let ty = self.parse_type()?;
            let tname = match self.advance().kind.clone() {
                TokenKind::Ident(id) => id,
                _ => {
                    return Err(CompileError::new(
                        "Expected typedef alias identifier",
                        self.current().line,
                        self.current().col,
                    ));
                }
            };
            self.expect(TokenKind::Semi)?;
            self.register_member_alias(&body.name, tname, ty);
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn parse_struct_using(
        &mut self,
        body: &mut StructBody,
    ) -> Result<bool, CompileError> {
        if self.peek() == &TokenKind::KwUsing {
            self.advance();
            let alias = match self.advance().kind.clone() {
                TokenKind::Ident(id) => id,
                _ => {
                    return Err(CompileError::new(
                        "Expected alias identifier in using",
                        self.current().line,
                        self.current().col,
                    ));
                }
            };
            if self.peek() == &TokenKind::Assign {
                self.advance();
                let ty = self.parse_type()?;
                self.expect(TokenKind::Semi)?;
                self.register_member_alias(&body.name, alias, ty);
                return Ok(true);
            } else {
                self.expect(TokenKind::Semi)?;
                return Ok(true);
            }
        }
        Ok(false)
    }
}
