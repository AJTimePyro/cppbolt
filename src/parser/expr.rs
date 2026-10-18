use super::*;

impl Parser {
    pub fn parse_expr(&mut self) -> Result<Expr, CompileError> {
        self.parse_expr_prec(0)
    }

    pub(super) fn parse_expr_prec(&mut self, min_prec: u8) -> Result<Expr, CompileError> {
        let mut lhs = self.parse_prefix()?;

        while let Some((op, prec, right_assoc)) = self.current_binary_op() {
            if prec < min_prec {
                break;
            }

            self.advance();
            let next_min = if right_assoc { prec } else { prec + 1 };
            let rhs = self.parse_expr_prec(next_min)?;

            lhs = match op {
                BinaryOpChoice::Binary(bop) => Expr::Binary {
                    op: bop,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                },
                BinaryOpChoice::Assign => Expr::Assign {
                    target: Box::new(lhs),
                    value: Box::new(rhs),
                },
                BinaryOpChoice::Compound(bop) => Expr::CompoundAssign {
                    op: bop,
                    target: Box::new(lhs),
                    value: Box::new(rhs),
                },
            };
        }

        // Ternary operator '?' : precedence lowest right above assignment
        if self.peek() == &TokenKind::Question && min_prec <= 3 {
            self.advance();
            let then_expr = self.parse_expr()?;
            self.expect(TokenKind::Colon)?;
            let else_expr = self.parse_expr_prec(3)?;
            lhs = Expr::Ternary {
                cond: Box::new(lhs),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            };
        }

        Ok(lhs)
    }

    pub(super) fn current_binary_op(&self) -> Option<(BinaryOpChoice, u8, bool)> {
        match self.peek() {
            TokenKind::Assign => Some((BinaryOpChoice::Assign, 2, true)),
            TokenKind::PlusAssign => Some((BinaryOpChoice::Compound(BinaryOp::Add), 2, true)),
            TokenKind::MinusAssign => Some((BinaryOpChoice::Compound(BinaryOp::Sub), 2, true)),
            TokenKind::StarAssign => Some((BinaryOpChoice::Compound(BinaryOp::Mul), 2, true)),
            TokenKind::SlashAssign => Some((BinaryOpChoice::Compound(BinaryOp::Div), 2, true)),
            TokenKind::PercentAssign => Some((BinaryOpChoice::Compound(BinaryOp::Rem), 2, true)),
            TokenKind::AmpAssign => Some((BinaryOpChoice::Compound(BinaryOp::BitAnd), 2, true)),
            TokenKind::PipeAssign => Some((BinaryOpChoice::Compound(BinaryOp::BitOr), 2, true)),
            TokenKind::CaretAssign => Some((BinaryOpChoice::Compound(BinaryOp::BitXor), 2, true)),
            TokenKind::ShlAssign => Some((BinaryOpChoice::Compound(BinaryOp::Shl), 2, true)),
            TokenKind::ShrAssign => Some((BinaryOpChoice::Compound(BinaryOp::Shr), 2, true)),

            TokenKind::OrOr => Some((BinaryOpChoice::Binary(BinaryOp::LogicalOr), 4, false)),
            TokenKind::AndAnd => Some((BinaryOpChoice::Binary(BinaryOp::LogicalAnd), 5, false)),
            TokenKind::Pipe => Some((BinaryOpChoice::Binary(BinaryOp::BitOr), 6, false)),
            TokenKind::Caret => Some((BinaryOpChoice::Binary(BinaryOp::BitXor), 7, false)),
            TokenKind::Amp => Some((BinaryOpChoice::Binary(BinaryOp::BitAnd), 8, false)),
            TokenKind::EqEq => Some((BinaryOpChoice::Binary(BinaryOp::Eq), 9, false)),
            TokenKind::BangEq => Some((BinaryOpChoice::Binary(BinaryOp::Ne), 9, false)),
            TokenKind::Lt => Some((BinaryOpChoice::Binary(BinaryOp::Lt), 10, false)),
            TokenKind::LtEq => Some((BinaryOpChoice::Binary(BinaryOp::Le), 10, false)),
            TokenKind::Gt => Some((BinaryOpChoice::Binary(BinaryOp::Gt), 10, false)),
            TokenKind::GtEq => Some((BinaryOpChoice::Binary(BinaryOp::Ge), 10, false)),
            TokenKind::Shl => Some((BinaryOpChoice::Binary(BinaryOp::Shl), 11, false)),
            TokenKind::Shr => Some((BinaryOpChoice::Binary(BinaryOp::Shr), 11, false)),
            TokenKind::Plus => Some((BinaryOpChoice::Binary(BinaryOp::Add), 12, false)),
            TokenKind::Minus => Some((BinaryOpChoice::Binary(BinaryOp::Sub), 12, false)),
            TokenKind::Star => Some((BinaryOpChoice::Binary(BinaryOp::Mul), 13, false)),
            TokenKind::Slash => Some((BinaryOpChoice::Binary(BinaryOp::Div), 13, false)),
            TokenKind::Percent => Some((BinaryOpChoice::Binary(BinaryOp::Rem), 13, false)),

            _ => None,
        }
    }
}

pub(super) enum BinaryOpChoice {
    Binary(BinaryOp),
    Assign,
    Compound(BinaryOp),
}
