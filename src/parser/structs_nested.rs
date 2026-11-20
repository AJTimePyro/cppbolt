use super::structs::StructBody;
use super::*;
use crate::ast::template::TemplateParam;

impl Parser {
    pub(super) fn parse_nested_struct_member(
        &mut self,
        body: &mut StructBody,
    ) -> Result<bool, CompileError> {
        if self.peek() == &TokenKind::KwTemplate {
            return self.parse_nested_template_struct(body);
        }
        if self.peek() != &TokenKind::KwStruct
            && self.peek() != &TokenKind::KwClass
            && !self.is_struct_after_alignas()
        {
            return Ok(false);
        }
        let decl = self.parse_nested_struct(body)?;
        body.nested_structs.push(decl);
        Ok(true)
    }

    fn parse_nested_template_struct(
        &mut self,
        body: &mut StructBody,
    ) -> Result<bool, CompileError> {
        let saved_pos = self.pos;
        let saved_splits = self.shr_splits.len();
        let t_tok = self.advance().clone();
        if self.peek() != &TokenKind::Lt {
            self.rollback_pos(saved_pos, saved_splits);
            return Ok(false);
        }
        self.advance();
        let params = match self.parse_template_params() {
            Ok(p) => p,
            Err(_) => {
                self.rollback_pos(saved_pos, saved_splits);
                return Ok(false);
            }
        };
        if self.peek() != &TokenKind::KwStruct
            && self.peek() != &TokenKind::KwClass
            && !self.is_struct_after_alignas()
        {
            self.active_type_params.pop();
            self.rollback_pos(saved_pos, saved_splits);
            return Ok(false);
        }
        let decl_res = self.parse_nested_struct(body);
        self.active_type_params.pop();
        let decl = decl_res?;
        let Decl::StructDecl {
            name,
            is_class,
            is_final,
            base_classes,
            base_access,
            fields,
            methods,
            static_fields,
            private_members,
            nested_structs,
            alignment,
            ..
        } = decl
        else {
            return Ok(false);
        };
        if !name.contains('<') {
            template::validate_trailing_pack(&params, "class template")?;
        }
        self.register_template_name(&name);
        let pattern_args = self.struct_template_args.get(&name).cloned();
        let member_aliases = self
            .struct_member_aliases
            .get(&name)
            .cloned()
            .unwrap_or_default();
        let template_class = TemplateClass {
            name,
            pattern_args,
            is_class,
            is_final,
            base_classes,
            base_access,
            fields,
            methods,
            friends: Vec::new(),
            static_fields,
            private_members,
            nested_structs,
            member_aliases,
            alignment,
        };
        body.nested_structs.push(Decl::Template(TemplateDecl {
            params,
            kind: TemplateKind::Class(template_class),
            line: t_tok.line,
            col: t_tok.col,
        }));
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
        let is_final = if matches!(self.peek(), TokenKind::Ident(id) if id == "final") {
            self.advance();
            true
        } else {
            false
        };
        let full_name = format!("{}::{}", body.name, short_name);
        self.known_types.insert(full_name.clone());
        if body.is_private {
            body.private_members.push(short_name);
            body.private_members.push(full_name.clone());
        }

        if self.peek() == &TokenKind::Semi {
            self.advance();
            return Ok(
                StructBody::new(full_name, is_class, is_final, start.line, start.col)
                    .into_decl(Vec::new()),
            );
        }

        let base_classes = self.parse_base_classes(is_class)?;
        self.expect(TokenKind::LBrace)?;

        self.active_struct_scopes.push(full_name.clone());
        let mut inner_body =
            StructBody::new(full_name.clone(), is_class, is_final, start.line, start.col);
        while self.peek() != &TokenKind::RBrace && self.peek() != &TokenKind::Eof {
            self.parse_struct_member(&mut inner_body)?;
        }
        self.active_struct_scopes.pop();

        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::Semi)?;
        for (fname, fty, _) in &inner_body.fields {
            self.member_types
                .insert(format!("{}::{}", full_name, fname), fty.clone());
            self.member_types
                .entry(fname.clone())
                .or_insert_with(|| fty.clone());
        }

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
            // `using Base::member;` / `using Base<T>::member;` brings base
            // names into scope; the current lookup already sees base methods.
            let rest: Vec<TokenKind> = self.tokens[self.pos..]
                .iter()
                .take_while(|t| t.kind != TokenKind::Semi && t.kind != TokenKind::Assign)
                .map(|t| t.kind.clone())
                .collect();
            if rest.contains(&TokenKind::ColonColon) {
                while self.peek() != &TokenKind::Semi && self.peek() != &TokenKind::Eof {
                    self.advance();
                }
                self.expect(TokenKind::Semi)?;
                return Ok(true);
            }
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
                self.register_member_alias(&body.name, alias, ty);
            }
            self.expect(TokenKind::Semi)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn parse_struct_access(
        &mut self,
        body: &mut StructBody,
    ) -> Result<bool, CompileError> {
        if self.peek() == &TokenKind::KwPublic {
            self.advance();
            self.expect(TokenKind::Colon)?;
            body.is_private = false;
            return Ok(true);
        }
        if matches!(self.peek(), TokenKind::KwPrivate | TokenKind::KwProtected) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            body.is_private = true;
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn parse_struct_friend(
        &mut self,
        body: &mut StructBody,
        template_params: Option<&[TemplateParam]>,
    ) -> Result<bool, CompileError> {
        if self.peek() != &TokenKind::KwFriend {
            return Ok(false);
        }
        let start = self.current().clone();
        self.advance();
        if !self.is_type_start() {
            // `friend class X;` and other non-function friends have no
            // function body to emit.
            while self.peek() != &TokenKind::Semi && self.peek() != &TokenKind::Eof {
                self.advance();
            }
            if self.peek() == &TokenKind::Semi {
                self.advance();
            }
            return Ok(true);
        }
        let specifiers = self.parse_specifier_type()?;
        let return_type = self.parse_declarator_prefix(&specifiers)?;
        let name = match self.advance().kind.clone() {
            TokenKind::Ident(id) => id,
            TokenKind::KwOperator => self.parse_operator_name()?,
            _ => {
                return Err(CompileError::new(
                    "Expected friend function name",
                    self.current().line,
                    self.current().col,
                ));
            }
        };
        let mut func = self.parse_function_tail(name, return_type, start.line, start.col)?;
        if let Some(params) = template_params {
            func.template_params = params.to_vec();
        }
        body.friends.push(func);
        Ok(true)
    }
}
