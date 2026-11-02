//! Template declaration and argument parsing.

use super::*;

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
        self.advance(); // consume '<'

        if self.peek() == &TokenKind::Gt {
            return Err(CompileError::new(
                "explicit template specialization is not supported yet",
                t_line,
                t_col,
            ));
        }

        let params = self.parse_template_params()?;
        self.parse_template_body(params, t_line, t_col, declarations)
    }

    pub(super) fn parse_template_params(&mut self) -> Result<Vec<TemplateParam>, CompileError> {
        let mut params = Vec::new();
        while self.peek() != &TokenKind::Gt && self.peek() != &TokenKind::Shr {
            let p_line = self.current().line;
            let p_col = self.current().col;

            if self.peek() == &TokenKind::KwTemplate {
                return Err(CompileError::new(
                    "template template parameters are not supported yet",
                    p_line,
                    p_col,
                ));
            }
            if self.peek() != &TokenKind::KwTypename && self.peek() != &TokenKind::KwClass {
                return Err(CompileError::new(
                    "non-type template parameters are not supported yet",
                    p_line,
                    p_col,
                ));
            }
            self.advance();

            if self.peek() == &TokenKind::Dot {
                return Err(CompileError::new(
                    "variadic template parameters are not supported yet",
                    p_line,
                    p_col,
                ));
            }

            let name = match self.advance().kind.clone() {
                TokenKind::Ident(id) => id,
                _ => return Err(CompileError::new("expected identifier", p_line, p_col)),
            };
            params.push(TemplateParam {
                name,
                line: p_line,
                col: p_col,
            });

            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.consume_template_closing_gt()?;
        Ok(params)
    }

    pub(super) fn consume_template_closing_gt(&mut self) -> Result<(), CompileError> {
        if self.peek() == &TokenKind::Gt {
            self.advance();
            Ok(())
        } else if self.peek() == &TokenKind::Shr {
            // Split >> into > followed by >
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
        let b_line = self.current().line;
        let b_col = self.current().col;

        if self.peek() == &TokenKind::KwUsing {
            return Err(CompileError::new(
                "alias templates are not supported yet",
                b_line,
                b_col,
            ));
        }

        let param_names: HashSet<String> = params.iter().map(|p| p.name.clone()).collect();
        self.active_type_params.push(param_names);

        let result = if self.peek() == &TokenKind::KwStruct || self.peek() == &TokenKind::KwClass {
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
        let decl = self.parse_struct()?;
        let Decl::StructDecl {
            name,
            is_class,
            base_classes,
            fields,
            methods,
            static_fields,
            private_members,
            ..
        } = decl
        else {
            return Ok(vec![decl]);
        };

        self.register_template_name(&name);
        let template_class = TemplateClass {
            name,
            is_class,
            base_classes,
            fields,
            methods,
            static_fields,
            private_members,
        };
        Ok(vec![Decl::Template(TemplateDecl {
            params,
            kind: TemplateKind::Class(template_class),
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
            let ty = self.parse_type()?;
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

    pub(super) fn is_template_qualifier_ahead(&self) -> bool {
        if self.peek() != &TokenKind::Lt {
            return false;
        }
        self.skip_template_angles(0)
            .is_some_and(|end| self.peek_ahead(end) == &TokenKind::ColonColon)
    }

    fn skip_template_angles(&self, mut idx: usize) -> Option<usize> {
        let mut depth = 0;
        while let Some(tok) = self.tokens.get(self.pos + idx) {
            match tok.kind {
                TokenKind::Lt => depth += 1,
                TokenKind::Gt => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(idx + 1);
                    }
                }
                TokenKind::Shr => {
                    if depth <= 2 {
                        return Some(idx + 1);
                    }
                    depth -= 2;
                }
                TokenKind::Semi | TokenKind::LBrace | TokenKind::RBrace | TokenKind::Eof => {
                    return None;
                }
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
