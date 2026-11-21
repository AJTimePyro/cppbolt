//! Template parameter list parsing.

use super::template_helpers::has_pack_in_expr;
use super::*;

impl Parser {
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
            params.push(self.parse_single_template_param(&mut seen_default)?);
            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }
        self.consume_template_closing_gt()
    }
    fn parse_template_param_type(
        &mut self,
        line: usize,
        col: usize,
    ) -> Result<(Option<Type>, String), CompileError> {
        if self.peek() == &TokenKind::KwTypename || self.peek() == &TokenKind::KwClass {
            self.advance();
            return Ok((None, String::new()));
        }
        if !self.is_type_start() {
            return Err(CompileError::new(
                "expected template parameter declaration",
                line,
                col,
            ));
        }
        let specifiers = self.parse_specifier_type()?;
        if self.is_function_pointer_declarator_ahead() {
            let (ty, name) =
                self.parse_function_pointer_declarator(specifiers, Some("expected identifier"))?;
            Ok((Some(ty), name))
        } else {
            Ok((
                Some(self.parse_declarator_prefix(&specifiers)?),
                String::new(),
            ))
        }
    }

    fn parse_single_template_param(
        &mut self,
        seen_default: &mut bool,
    ) -> Result<TemplateParam, CompileError> {
        let p_line = self.current().line;
        let p_col = self.current().col;

        if self.peek() == &TokenKind::KwTemplate {
            return self.parse_template_template_param(p_line, p_col, seen_default);
        }
        let (param_type, declarator_name) = self.parse_template_param_type(p_line, p_col)?;

        let is_pack = if self.peek() == &TokenKind::Ellipsis {
            self.advance();
            true
        } else {
            false
        };

        let name = if !declarator_name.is_empty() {
            declarator_name
        } else if let TokenKind::Ident(id) = self.peek().clone() {
            self.advance();
            id
        } else if is_pack
            || self.peek() == &TokenKind::Comma
            || self.peek() == &TokenKind::Gt
            || self.peek() == &TokenKind::Shr
            || self.peek() == &TokenKind::Assign
        {
            // An unnamed template parameter such as `typename = void` is
            // valid; the name is not referenced.
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
        let default_type = self.parse_template_param_default(param_type.as_ref())?;
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
            param_type,
            is_pack,
            line: p_line,
            col: p_col,
        })
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
            Ok(Some(self.parse_nttp_const_arg()?))
        } else {
            Ok(Some(self.parse_type()?))
        }
    }

    pub(super) fn parse_nttp_const_arg(&mut self) -> Result<Type, CompileError> {
        let saved_in_template = self.in_template_arg;
        let saved_depth = self.template_arg_paren_depth;
        self.in_template_arg = true;
        self.template_arg_paren_depth = 0;
        let expr_res = self.parse_expr_prec(2);
        self.in_template_arg = saved_in_template;
        self.template_arg_paren_depth = saved_depth;
        let expr = expr_res?;
        if matches!(&expr, Expr::IntLiteral(0, Type::Nullptr)) {
            return Ok(Type::DependentNonType(
                crate::sema::pack_expansion::NULL_ARG_PREFIX.to_string(),
            ));
        }
        if let Expr::Unary {
            op: UnaryOp::AddrOf,
            expr: inner,
            ..
        } = &expr
            && let Expr::Identifier(name) = &**inner
        {
            return Ok(Type::DependentNonType(format!(
                "{}{}",
                crate::sema::pack_expansion::ADDRESS_ARG_PREFIX,
                name.trim_start_matches("::")
            )));
        }
        // An expression naming an active template parameter stays dependent:
        // evaluating it here would use a placeholder size for the parameter
        // (e.g. `sizeof(T) > 1` would always be true).
        if let Some(text) = super::types_array::expr_to_dependent_str(&expr)
            && super::template_helpers::text_mentions_active(&text, &self.active_type_params)
        {
            return Ok(Type::DependentNonType(text));
        }
        match crate::sema::constexpr_eval::eval_integral_constexpr(&expr, None) {
            Ok(val) => Ok(Type::NonType(val)),
            Err(err) => {
                let is_dep = !self.active_type_params.is_empty() || has_pack_in_expr(&expr);
                if let Some(dep_str) = super::types_array::expr_to_dependent_str(&expr)
                    && (is_dep
                        || dep_str.contains("::")
                        || super::template_helpers::has_identifier_in_expr(&expr))
                {
                    // A qualified name here is a static const member of some
                    // class template; its value is only known after sema has
                    // instantiated that class, so defer evaluation.
                    Ok(Type::DependentNonType(dep_str))
                } else {
                    Err(err)
                }
            }
        }
    }
}
