use super::Parser;
use crate::ast::{BinaryOp, Expr, ExprPos, FoldOp};
use crate::diagnostics::CompileError;
use crate::lexer::TokenKind;

impl Parser {
    pub(super) fn is_fold_operator_token(
        &self,
        kind: &TokenKind,
        next_kind: Option<&TokenKind>,
    ) -> bool {
        match kind {
            TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::Caret
            | TokenKind::Amp
            | TokenKind::Pipe
            | TokenKind::Shl
            | TokenKind::Shr
            | TokenKind::EqEq
            | TokenKind::BangEq
            | TokenKind::Lt
            | TokenKind::Gt
            | TokenKind::LtEq
            | TokenKind::GtEq
            | TokenKind::AndAnd
            | TokenKind::OrOr
            | TokenKind::Comma
            | TokenKind::Assign
            | TokenKind::PlusAssign
            | TokenKind::MinusAssign
            | TokenKind::StarAssign
            | TokenKind::SlashAssign
            | TokenKind::PercentAssign
            | TokenKind::CaretAssign
            | TokenKind::AmpAssign
            | TokenKind::PipeAssign
            | TokenKind::ShlAssign
            | TokenKind::ShrAssign => true,
            TokenKind::Dot | TokenKind::Arrow => next_kind == Some(&TokenKind::Star),
            _ => false,
        }
    }

    pub(super) fn has_depth1_ellipsis(&self) -> bool {
        if self.peek() != &TokenKind::LParen {
            return false;
        }
        let mut depth = 0;
        let mut i = 0;
        while let Some(tok) = self.tokens.get(self.pos + i) {
            match tok.kind {
                TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                    depth += 1;
                }
                TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                TokenKind::Ellipsis if depth == 1 => {
                    let prev_tok = if i > 0 {
                        self.tokens.get(self.pos + i - 1).map(|t| &t.kind)
                    } else {
                        None
                    };
                    let next_tok = self.tokens.get(self.pos + i + 1).map(|t| &t.kind);
                    let after_next = self.tokens.get(self.pos + i + 2).map(|t| &t.kind);

                    if prev_tok == Some(&TokenKind::LParen) {
                        if let Some(next) = next_tok
                            && self.is_fold_operator_token(next, after_next)
                        {
                            return true;
                        }
                    } else if let Some(prev) = prev_tok {
                        let before_prev = if i >= 2 {
                            self.tokens.get(self.pos + i - 2).map(|t| &t.kind)
                        } else {
                            None
                        };
                        if self.is_fold_operator_token(prev, next_tok)
                            || (prev == &TokenKind::Star
                                && matches!(before_prev, Some(&TokenKind::Dot | &TokenKind::Arrow)))
                        {
                            return true;
                        }
                    }
                }
                TokenKind::Semi | TokenKind::Eof => {
                    break;
                }
                _ => {}
            }
            i += 1;
        }
        false
    }

    pub(super) fn parse_fold_operator(&mut self) -> Result<FoldOp, CompileError> {
        if self.peek() == &TokenKind::Dot && self.peek_ahead(1) == &TokenKind::Star {
            self.advance();
            self.advance();
            return Ok(FoldOp::DotStar);
        }
        if self.peek() == &TokenKind::Arrow && self.peek_ahead(1) == &TokenKind::Star {
            self.advance();
            self.advance();
            return Ok(FoldOp::ArrowStar);
        }
        let tok = self.peek();
        let op = match tok {
            TokenKind::Plus => FoldOp::Binary(BinaryOp::Add),
            TokenKind::Minus => FoldOp::Binary(BinaryOp::Sub),
            TokenKind::Star => FoldOp::Binary(BinaryOp::Mul),
            TokenKind::Slash => FoldOp::Binary(BinaryOp::Div),
            TokenKind::Percent => FoldOp::Binary(BinaryOp::Rem),
            TokenKind::Caret => FoldOp::Binary(BinaryOp::BitXor),
            TokenKind::Amp => FoldOp::Binary(BinaryOp::BitAnd),
            TokenKind::Pipe => FoldOp::Binary(BinaryOp::BitOr),
            TokenKind::Shl => FoldOp::Binary(BinaryOp::Shl),
            TokenKind::Shr => FoldOp::Binary(BinaryOp::Shr),
            TokenKind::EqEq => FoldOp::Binary(BinaryOp::Eq),
            TokenKind::BangEq => FoldOp::Binary(BinaryOp::Ne),
            TokenKind::Lt => FoldOp::Binary(BinaryOp::Lt),
            TokenKind::Gt => FoldOp::Binary(BinaryOp::Gt),
            TokenKind::LtEq => FoldOp::Binary(BinaryOp::Le),
            TokenKind::GtEq => FoldOp::Binary(BinaryOp::Ge),
            TokenKind::AndAnd => FoldOp::Binary(BinaryOp::LogicalAnd),
            TokenKind::OrOr => FoldOp::Binary(BinaryOp::LogicalOr),
            TokenKind::Comma => FoldOp::Binary(BinaryOp::Comma),
            TokenKind::Assign => FoldOp::Assign,
            TokenKind::PlusAssign => FoldOp::Compound(BinaryOp::Add),
            TokenKind::MinusAssign => FoldOp::Compound(BinaryOp::Sub),
            TokenKind::StarAssign => FoldOp::Compound(BinaryOp::Mul),
            TokenKind::SlashAssign => FoldOp::Compound(BinaryOp::Div),
            TokenKind::PercentAssign => FoldOp::Compound(BinaryOp::Rem),
            TokenKind::CaretAssign => FoldOp::Compound(BinaryOp::BitXor),
            TokenKind::AmpAssign => FoldOp::Compound(BinaryOp::BitAnd),
            TokenKind::PipeAssign => FoldOp::Compound(BinaryOp::BitOr),
            TokenKind::ShlAssign => FoldOp::Compound(BinaryOp::Shl),
            TokenKind::ShrAssign => FoldOp::Compound(BinaryOp::Shr),
            _ => {
                return Err(CompileError::new(
                    format!("expected fold operator, found {}", tok.diagnostic_name()),
                    self.current().line,
                    self.current().col,
                ));
            }
        };
        self.advance();
        Ok(op)
    }

    pub(super) fn parse_fold_expr(&mut self, start: ExprPos) -> Result<Expr, CompileError> {
        if self.peek() == &TokenKind::Ellipsis {
            self.advance();
            let op = self.parse_fold_operator()?;
            let pack = self.parse_prefix()?;
            self.expect(TokenKind::RParen)?;
            return Ok(Expr::FoldExpr {
                op,
                left: None,
                right: Some(Box::new(pack)),
                pos: start,
            });
        }

        let left = self.parse_prefix()?;
        let op1 = self.parse_fold_operator()?;
        self.expect(TokenKind::Ellipsis)?;
        if self.peek() == &TokenKind::RParen {
            self.advance();
            return Ok(Expr::FoldExpr {
                op: op1,
                left: Some(Box::new(left)),
                right: None,
                pos: start,
            });
        }

        let op2_pos = self.current_pos();
        let op2 = self.parse_fold_operator()?;
        if op1 != op2 {
            return Err(CompileError::new(
                format!(
                    "binary fold expression has mismatched operators ('{}' and '{}')",
                    op1.as_str(),
                    op2.as_str()
                ),
                op2_pos.line,
                op2_pos.col,
            ));
        }
        let right = self.parse_prefix()?;
        self.expect(TokenKind::RParen)?;
        Ok(Expr::FoldExpr {
            op: op1,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            pos: start,
        })
    }
}
