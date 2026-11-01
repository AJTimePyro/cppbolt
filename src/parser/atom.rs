use super::*;

impl Parser {
    pub(super) fn parse_atom(&mut self) -> Result<Expr, CompileError> {
        let tok = self.advance().clone();
        match tok.kind {
            TokenKind::IntLiteral(val, raw) => {
                let ty = int_literal_type(val, &raw);
                Ok(Expr::IntLiteral(val, ty))
            }
            TokenKind::FloatLiteral(val, raw) => {
                let ty = if raw.ends_with('f') || raw.ends_with('F') {
                    Type::Float
                } else {
                    Type::Double
                };
                Ok(Expr::FloatLiteral(val, ty))
            }
            TokenKind::KwThis => Ok(Expr::This),
            TokenKind::KwNullptr => Ok(Expr::IntLiteral(0, Type::Nullptr)),
            TokenKind::BoolLiteral(val) => Ok(Expr::BoolLiteral(val)),
            TokenKind::CharLiteral(val) => Ok(Expr::CharLiteral(val)),
            TokenKind::StringLiteral(val) => Ok(Expr::StringLiteral(val)),
            TokenKind::LBracket => self.parse_lambda(tok.line, tok.col),
            TokenKind::ColonColon => {
                let first = match self.advance().kind.clone() {
                    TokenKind::Ident(sub) => sub,
                    _ => {
                        return Err(CompileError::new(
                            "Expected identifier after '::'",
                            tok.line,
                            tok.col,
                        ));
                    }
                };
                let id = self.parse_qualified_tail(first, Some((tok.line, tok.col)))?;
                if let Some(stripped) = id.strip_prefix("std::") {
                    Ok(Expr::Identifier(stripped.to_string()))
                } else {
                    Ok(Expr::Identifier(format!("::{}", id)))
                }
            }
            TokenKind::Ident(id) => {
                let mut id = self.parse_qualified_tail(id, Some((tok.line, tok.col)))?;
                if let Some(stripped) = id.strip_prefix("std::") {
                    id = stripped.to_string();
                }
                Ok(Expr::Identifier(id))
            }
            TokenKind::KwThrow => {
                if self.peek() != &TokenKind::Semi
                    && self.peek() != &TokenKind::RParen
                    && self.peek() != &TokenKind::Colon
                    && self.peek() != &TokenKind::Eof
                {
                    let _ = self.parse_expr()?;
                }
                Err(CompileError::new(
                    "C++ exception handling ('throw') is not supported on this WebAssembly target",
                    tok.line,
                    tok.col,
                ))
            }
            _ => Err(CompileError::new(
                format!(
                    "Unexpected token in expression: {}",
                    tok.kind.diagnostic_name()
                ),
                tok.line,
                tok.col,
            )),
        }
    }

    pub(super) fn parse_postfix(
        &mut self,
        mut expr: Expr,
        start: ExprPos,
    ) -> Result<Expr, CompileError> {
        loop {
            match self.peek() {
                TokenKind::LParen => {
                    self.advance();
                    let args = self.parse_call_args()?;
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                        pos: start,
                    };
                }
                TokenKind::LBrace => {
                    if let Expr::Identifier(ref name) = expr {
                        let (elements, pos) = self.parse_brace_expr_list()?;
                        expr = Expr::TypedInitList {
                            ty: Type::Struct(name.clone()),
                            elements,
                            pos,
                        };
                    } else {
                        break;
                    }
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(TokenKind::RBracket)?;
                    expr = Expr::ArraySubscript {
                        array: Box::new(expr),
                        index: Box::new(index),
                        pos: start,
                    };
                }
                TokenKind::Dot => {
                    self.advance();
                    let member = self.parse_member_name(".")?;
                    expr = Expr::MemberAccess {
                        object: Box::new(expr),
                        member,
                        is_arrow: false,
                        pos: start,
                    };
                }
                TokenKind::Arrow => {
                    self.advance();
                    let member = self.parse_member_name("->")?;
                    expr = Expr::MemberAccess {
                        object: Box::new(expr),
                        member,
                        is_arrow: true,
                        pos: start,
                    };
                }
                TokenKind::PlusPlus => {
                    self.advance();
                    expr = Expr::Unary {
                        op: UnaryOp::PostInc,
                        expr: Box::new(expr),
                        pos: start,
                    };
                }
                TokenKind::MinusMinus => {
                    self.advance();
                    expr = Expr::Unary {
                        op: UnaryOp::PostDec,
                        expr: Box::new(expr),
                        pos: start,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    pub(super) fn parse_functional_cast(
        &mut self,
        ty: Type,
        start: ExprPos,
    ) -> Result<Expr, CompileError> {
        self.advance();
        let mut args = self.parse_call_args()?;
        let expr = if args.is_empty() {
            zero_literal_for_type(&ty)
        } else if args.len() == 1 {
            Expr::Cast {
                kind: CastKind::Functional,
                target_type: ty,
                expr: Box::new(args.swap_remove(0)),
            }
        } else {
            return Err(CompileError::new(
                "scalar functional cast requires 0 or 1 arguments",
                start.line,
                start.col,
            ));
        };
        self.parse_postfix(expr, start)
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, CompileError> {
        // One call costs one level of the call budget.
        let _guard = self.enter_call()?;
        self.parse_paren_expr_list()
    }

    fn parse_member_name(&mut self, operator: &str) -> Result<String, CompileError> {
        if self.peek() == &TokenKind::Tilde {
            self.advance();
            match self.advance().kind.clone() {
                TokenKind::Ident(id) => Ok(format!("~{}", id)),
                _ => Err(CompileError::new(
                    "Expected destructor name after '~'",
                    self.current().line,
                    self.current().col,
                )),
            }
        } else {
            match self.advance().kind.clone() {
                TokenKind::Ident(id) => Ok(id),
                _ => Err(CompileError::new(
                    format!("Expected member name after '{}'", operator),
                    self.current().line,
                    self.current().col,
                )),
            }
        }
    }
}

fn int_literal_type(val: i64, raw: &str) -> Type {
    let lower = raw.to_lowercase();
    let has_u = lower.contains('u');
    let has_ll = lower.contains("ll");
    let has_l = !has_ll && lower.contains('l');

    let is_non_decimal = raw.starts_with("0x")
        || raw.starts_with("0X")
        || raw.starts_with("0b")
        || raw.starts_with("0B")
        || (raw.starts_with('0') && raw.len() > 1 && !raw.contains('.'));

    if has_u && has_ll {
        Type::UnsignedLongLong
    } else if has_ll {
        if is_non_decimal && val < 0 {
            Type::UnsignedLongLong
        } else {
            Type::LongLong
        }
    } else if has_u && has_l {
        if val > u32::MAX as i64 || val < 0 {
            Type::UnsignedLongLong
        } else {
            Type::UnsignedLong
        }
    } else if has_l {
        if val >= 0 && val <= i32::MAX as i64 {
            Type::Long
        } else if is_non_decimal && (val >= 0 && val <= u32::MAX as i64) {
            Type::UnsignedLong
        } else {
            Type::LongLong
        }
    } else if has_u {
        if val >= 0 && val <= u32::MAX as i64 {
            Type::UnsignedInt
        } else {
            Type::UnsignedLongLong
        }
    } else if val >= 0 && val <= i32::MAX as i64 {
        Type::Int
    } else if is_non_decimal && (val >= 0 && val <= u32::MAX as i64) {
        Type::UnsignedInt
    } else {
        Type::LongLong
    }
}
