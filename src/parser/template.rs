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
            self.advance();
            self.active_type_params.push(HashSet::new());
            return self.parse_template_body(Vec::new(), t_line, t_col, declarations);
        }

        let params = self.parse_template_params()?;
        self.parse_template_body(params, t_line, t_col, declarations)
    }

    pub(super) fn parse_template_params(&mut self) -> Result<Vec<TemplateParam>, CompileError> {
        let mut params = Vec::new();
        self.active_type_params.push(HashSet::new());
        let res = self.parse_template_params_inner(&mut params);
        if res.is_err() {
            self.active_type_params.pop();
        }
        res?;
        Ok(params)
    }

    fn parse_template_params_inner(
        &mut self,
        params: &mut Vec<TemplateParam>,
    ) -> Result<(), CompileError> {
        let mut seen_default = false;
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
            let param_type = if self.peek() == &TokenKind::KwTypename
                || self.peek() == &TokenKind::KwClass
            {
                self.advance();
                None
            } else if self.is_type_start() {
                Some(self.parse_type()?)
            } else {
                return Err(CompileError::new(
                    "expected template parameter declaration",
                    p_line,
                    p_col,
                ));
            };

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
            if let Some(set) = self.active_type_params.last_mut() {
                set.insert(name.clone());
            }
            let default_type = self.parse_template_param_default(param_type.as_ref())?;
            if default_type.is_some() {
                seen_default = true;
            } else if seen_default {
                return Err(CompileError::new(
                    "template parameter without default follows parameter with default",
                    p_line,
                    p_col,
                ));
            }
            params.push(TemplateParam {
                name,
                default_type,
                param_type,
                line: p_line,
                col: p_col,
            });

            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.consume_template_closing_gt()
    }

    fn parse_template_param_default(
        &mut self,
        param_type: Option<&Type>,
    ) -> Result<Option<Type>, CompileError> {
        if self.peek() != &TokenKind::Assign {
            return Ok(None);
        }
        self.advance();
        if param_type.is_some() {
            let saved_in_template = self.in_template_arg;
            let saved_depth = self.template_arg_paren_depth;
            self.in_template_arg = true;
            self.template_arg_paren_depth = 0;
            let expr_res = self.parse_expr_prec(2);
            self.in_template_arg = saved_in_template;
            self.template_arg_paren_depth = saved_depth;
            let expr = expr_res?;
            let val = crate::sema::constexpr_eval::eval_integral_constexpr(&expr, None)?;
            Ok(Some(Type::NonType(val)))
        } else {
            Ok(Some(self.parse_type()?))
        }
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
        } else if self.peek() == &TokenKind::KwStruct || self.peek() == &TokenKind::KwClass {
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
            nested_structs,
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
            nested_structs,
        };
        Ok(vec![Decl::Template(TemplateDecl {
            params,
            kind: TemplateKind::Class(template_class),
            line: t_line,
            col: t_col,
        })])
    }

    fn parse_template_alias(
        &mut self,
        params: Vec<TemplateParam>,
        t_line: usize,
        t_col: usize,
    ) -> Result<Vec<Decl>, CompileError> {
        self.advance();
        let (id_line, id_col) = (self.current().line, self.current().col);
        let name = match self.advance().kind.clone() {
            TokenKind::Ident(id) => id,
            _ => return Err(CompileError::new("expected identifier", id_line, id_col)),
        };
        self.register_template_name(&name);
        self.expect(TokenKind::Assign)?;
        let ty = self.parse_type()?;
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
        if self.is_type_start() {
            let saved_pos = self.pos;
            let saved_splits = self.shr_splits.len();
            if let Ok(ty) = self.parse_type() {
                return Ok(ty);
            }
            self.rollback_pos(saved_pos, saved_splits);
        }
        let saved_in_template = self.in_template_arg;
        let saved_depth = self.template_arg_paren_depth;
        self.in_template_arg = true;
        self.template_arg_paren_depth = 0;
        let expr_res = self.parse_expr_prec(2);
        self.in_template_arg = saved_in_template;
        self.template_arg_paren_depth = saved_depth;
        let expr = expr_res?;
        let val = crate::sema::constexpr_eval::eval_integral_constexpr(&expr, None)?;
        Ok(Type::NonType(val))
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
                    if depth == 2 {
                        return Some(idx + 1);
                    } else if depth > 2 {
                        depth -= 2;
                    } else {
                        return None;
                    }
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
