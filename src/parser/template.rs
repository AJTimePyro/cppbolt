//! Template declaration and argument parsing.

use super::*;

pub(super) fn validate_trailing_pack(
    params: &[TemplateParam],
    kind: &str,
) -> Result<(), CompileError> {
    for (i, p) in params.iter().enumerate() {
        if p.is_pack && i + 1 < params.len() {
            return Err(CompileError::new(
                format!("template parameter pack must be the last template parameter in a {kind}"),
                p.line,
                p.col,
            ));
        }
    }
    Ok(())
}

impl Parser {
    pub(super) fn parse_template_decl(
        &mut self,
        declarations: &mut Vec<Decl>,
    ) -> Result<(), CompileError> {
        let t_tok = self.advance().clone();
        let (t_line, t_col) = (t_tok.line, t_tok.col);

        if self.peek() != &TokenKind::Lt {
            return Err(CompileError::new(
                "explicit template instantiation is not supported yet",
                t_line,
                t_col,
            ));
        }
        self.advance();

        if self.peek() == &TokenKind::Gt {
            self.advance();
            self.active_type_params.push(HashSet::new());
            return self.parse_template_body(Vec::new(), t_line, t_col, declarations);
        }

        let params = self.parse_template_params()?;
        self.parse_template_body(params, t_line, t_col, declarations)
    }

    pub(super) fn consume_template_closing_gt(&mut self) -> Result<(), CompileError> {
        if self.peek() == &TokenKind::Gt {
            self.advance();
            Ok(())
        } else if self.peek() == &TokenKind::Shr {
            // Split >> into > followed by >
            self.shr_splits.push(self.pos);
            self.tokens[self.pos].kind = TokenKind::Gt;
            Ok(())
        } else {
            Err(CompileError::new(
                "expected '>' to close template argument list",
                self.current().line,
                self.current().col,
            ))
        }
    }

    fn parse_template_body(
        &mut self,
        params: Vec<TemplateParam>,
        t_line: usize,
        t_col: usize,
        declarations: &mut Vec<Decl>,
    ) -> Result<(), CompileError> {
        let result = if self.peek() == &TokenKind::KwUsing {
            self.parse_template_alias(params, t_line, t_col)
        } else if self.peek() == &TokenKind::KwStruct
            || self.peek() == &TokenKind::KwClass
            || self.is_struct_after_alignas()
        {
            self.parse_template_class(params, t_line, t_col)
        } else {
            self.parse_template_function(params, t_line, t_col)
        };

        self.active_type_params.pop();
        let decls = result?;
        declarations.extend(decls);
        Ok(())
    }

    fn parse_template_class(
        &mut self,
        params: Vec<TemplateParam>,
        t_line: usize,
        t_col: usize,
    ) -> Result<Vec<Decl>, CompileError> {
        let mut struct_decls = self.parse_struct()?;
        let friends: Vec<FunctionDecl> = struct_decls
            .iter()
            .filter_map(|decl| match decl {
                Decl::FriendFunction { func, .. } => Some(func.clone()),
                _ => None,
            })
            .collect();
        struct_decls.retain(|decl| !matches!(decl, Decl::FriendFunction { .. }));
        let Some(decl) = struct_decls.pop() else {
            return Err(CompileError::new(
                "expected struct declaration",
                t_line,
                t_col,
            ));
        };
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
            return Ok(vec![decl]);
        };

        if !name.contains('<') {
            validate_trailing_pack(&params, "class template")?;
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
            friends,
            static_fields,
            private_members,
            nested_structs,
            member_aliases,
            alignment,
        };
        struct_decls.push(Decl::Template(TemplateDecl {
            params,
            kind: TemplateKind::Class(template_class),
            line: t_line,
            col: t_col,
        }));
        Ok(struct_decls)
    }

    fn parse_template_alias(
        &mut self,
        params: Vec<TemplateParam>,
        t_line: usize,
        t_col: usize,
    ) -> Result<Vec<Decl>, CompileError> {
        validate_trailing_pack(&params, "template alias")?;
        self.advance();
        let (id_line, id_col) = (self.current().line, self.current().col);
        let name = match self.advance().kind.clone() {
            TokenKind::Ident(id) => id,
            _ => return Err(CompileError::new("expected identifier", id_line, id_col)),
        };
        let name = self.qualify_name(name);
        self.register_template_name(&name);
        self.expect(TokenKind::Assign)?;
        let mut ty = self.parse_type_id()?;
        ty.set_decltype_alias(name.clone());
        self.expect(TokenKind::Semi)?;
        Ok(vec![Decl::Template(TemplateDecl {
            params,
            kind: TemplateKind::Alias(name, ty),
            line: t_line,
            col: t_col,
        })])
    }

    fn parse_template_function(
        &mut self,
        params: Vec<TemplateParam>,
        t_line: usize,
        t_col: usize,
    ) -> Result<Vec<Decl>, CompileError> {
        let decls = self.parse_global_or_func()?;
        let mut template_decls = Vec::new();
        for decl in decls {
            if let Decl::Function(func) = decl {
                self.register_template_name(&func.name);
                template_decls.push(Decl::Template(TemplateDecl {
                    params: params.clone(),
                    kind: TemplateKind::Function(Box::new(func)),
                    line: t_line,
                    col: t_col,
                }));
            } else if let Decl::GlobalVar { name, ty, init, .. } = decl {
                self.register_template_name(&name);
                template_decls.push(Decl::Template(TemplateDecl {
                    params: params.clone(),
                    kind: TemplateKind::Variable(name, ty, init),
                    line: t_line,
                    col: t_col,
                }));
            } else {
                template_decls.push(decl);
            }
        }
        Ok(template_decls)
    }

    pub fn register_template_name(&mut self, name: &str) {
        self.known_templates.insert(name.to_string());
        if let Some(short) = name.split("::").last() {
            self.known_templates.insert(short.to_string());
        }
        let qualified = QualifiedName::new(name);
        if qualified.is_global() {
            self.known_templates
                .insert(qualified.std_name().into_owned());
        } else if qualified.in_std() {
            self.known_templates.insert(qualified.plain().to_string());
        }
    }

    pub(super) fn parse_template_args(&mut self) -> Result<Vec<Type>, CompileError> {
        self.expect(TokenKind::Lt)?;
        let mut args = Vec::new();
        while self.peek() != &TokenKind::Gt && self.peek() != &TokenKind::Shr {
            let ty = self.parse_template_arg_type()?;
            args.push(ty);
            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.consume_template_closing_gt()?;
        Ok(args)
    }

    pub fn parse_template_arg_type(&mut self) -> Result<Type, CompileError> {
        let is_pack_ahead = matches!(self.peek(), TokenKind::Ident(_))
            && self.peek_ahead(1) == &TokenKind::Ellipsis;
        if self.is_type_start() || is_pack_ahead {
            let saved_pos = self.pos;
            let saved_splits = self.shr_splits.len();
            let saved_allow = self.allow_unknown_types;
            if is_pack_ahead {
                self.allow_unknown_types = true;
            }
            let res = self.parse_type_id();
            self.allow_unknown_types = saved_allow;
            if let Ok(mut ty) = res {
                // An array type can appear directly as a template argument
                // (`is_array<T[3]>`); type-id parsing stops before the suffix.
                if self.peek() == &TokenKind::LBracket {
                    ty = self.parse_array_suffix(ty)?;
                }
                if self.peek() == &TokenKind::Ellipsis {
                    self.advance();
                    ty = Type::PackExpansion(Box::new(ty));
                }
                return Ok(ty);
            }
            self.rollback_pos(saved_pos, saved_splits);
        }
        self.parse_nttp_const_arg()
    }

    pub(super) fn is_template_qualifier_ahead(&self) -> bool {
        if self.peek() != &TokenKind::Lt {
            return false;
        }
        self.skip_template_angles(0)
            .is_some_and(|end| self.peek_ahead(end) == &TokenKind::ColonColon)
    }

    fn skip_template_angles(&self, mut idx: usize) -> Option<usize> {
        let mut depth = 0;
        let mut paren = 0usize;
        let mut bracket = 0usize;
        let mut brace = 0usize;
        while let Some(tok) = self.tokens.get(self.pos + idx) {
            match tok.kind {
                // A `>` inside a parenthesized or braced subexpression is a
                // comparison, not the end of the argument list.
                TokenKind::Lt if paren == 0 && bracket == 0 && brace == 0 => depth += 1,
                TokenKind::Gt if paren == 0 && bracket == 0 && brace == 0 => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(idx + 1);
                    }
                }
                TokenKind::Shr if paren == 0 && bracket == 0 && brace == 0 => {
                    if depth == 2 {
                        return Some(idx + 1);
                    } else if depth > 2 {
                        depth -= 2;
                    } else {
                        return None;
                    }
                }
                TokenKind::LParen => paren += 1,
                TokenKind::RParen => paren = paren.saturating_sub(1),
                TokenKind::LBracket => bracket += 1,
                TokenKind::RBracket => bracket = bracket.saturating_sub(1),
                TokenKind::LBrace => {
                    if depth == 0 {
                        return None;
                    }
                    brace += 1;
                }
                TokenKind::RBrace => brace = brace.saturating_sub(1),
                TokenKind::Semi | TokenKind::Eof if depth == 0 => return None,
                _ => {}
            }
            idx += 1;
        }
        None
    }

    pub(super) fn is_out_of_class_ctor_or_dtor(&self) -> bool {
        let mut idx = 0;
        if self.peek_ahead(idx) == &TokenKind::ColonColon {
            idx += 1;
        }
        let mut last_class = String::new();
        loop {
            let TokenKind::Ident(name) = self.peek_ahead(idx) else {
                return false;
            };
            last_class.clear();
            last_class.push_str(name);
            idx += 1;
            if self.peek_ahead(idx) == &TokenKind::Lt {
                let Some(next_idx) = self.skip_template_angles(idx) else {
                    return false;
                };
                idx = next_idx;
            }
            if self.peek_ahead(idx) != &TokenKind::ColonColon {
                return false;
            }
            idx += 1;
            let mut check_idx = idx;
            if self.peek_ahead(check_idx) == &TokenKind::Tilde {
                check_idx += 1;
            }
            if let TokenKind::Ident(sub_name) = self.peek_ahead(check_idx)
                && sub_name == &last_class
                && self.peek_ahead(check_idx + 1) == &TokenKind::LParen
            {
                return true;
            }
        }
    }
}
