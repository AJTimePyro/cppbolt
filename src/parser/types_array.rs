//! Array declarator suffix parsing for types.

use super::*;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;

enum ArrayDim {
    Fixed(usize),
    Dependent(String),
}

pub(crate) fn expr_to_dependent_str(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Identifier(id) => Some(id.clone()),
        Expr::IntLiteral(val, _) => Some(val.to_string()),
        Expr::SizeofPack { name, .. } => Some(format!("sizeof...({name})")),
        Expr::SizeofType(ty) => Some(format!("sizeof({})", ty.to_c_type())),
        Expr::AlignofType(ty) => Some(format!("alignof({})", ty.to_c_type())),
        Expr::Cast { expr: inner, .. } => expr_to_dependent_str(inner),
        Expr::Unary { op, expr, .. } => {
            let inner = expr_to_dependent_str(expr)?;
            let sym = match op {
                UnaryOp::Plus => "+",
                UnaryOp::Neg => "-",
                UnaryOp::BitNot => "~",
                UnaryOp::Not => "!",
                _ => return None,
            };
            Some(format!("{sym}{inner}"))
        }
        Expr::Binary { op, left, right, .. } => {
            let l = expr_to_dependent_str(left)?;
            let r = expr_to_dependent_str(right)?;
            let sym = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Rem => "%",
                BinaryOp::BitAnd => "&",
                BinaryOp::BitOr => "|",
                BinaryOp::BitXor => "^",
                BinaryOp::Shl => "<<",
                BinaryOp::Shr => ">>",
                _ => return None,
            };
            Some(format!("({l} {sym} {r})"))
        }
        _ => None,
    }
}

impl Parser {
    pub(super) fn parse_array_suffix(&mut self, elem_type: Type) -> Result<Type, CompileError> {
        let mut dimensions = Vec::new();
        while self.peek() == &TokenKind::LBracket {
            let bracket_pos = self.current_pos();
            self.advance();
            let dim = if self.peek() != &TokenKind::RBracket {
                let expr = self.parse_expr()?;
                if let Expr::Identifier(ref id) = expr
                    && self.is_active_type_param(id)
                {
                    ArrayDim::Dependent(id.clone())
                } else if let Some(dep_str) = expr_to_dependent_str(&expr)
                    && (!self.active_type_params.is_empty() || matches!(expr, Expr::SizeofPack { .. }))
                {
                    ArrayDim::Dependent(dep_str)
                } else {
                    match crate::sema::constexpr_eval::eval_integral_constexpr(&expr, None) {
                        Ok(n) if n >= 0 => ArrayDim::Fixed(n as usize),
                        Ok(_) => {
                            return Err(CompileError::new(
                                "Array dimension must be non-negative integer constant",
                                bracket_pos.line,
                                bracket_pos.col,
                            ));
                        }
                        Err(mut err) => {
                            if !self.active_type_params.is_empty()
                                && let Some(dep_str) = expr_to_dependent_str(&expr)
                            {
                                ArrayDim::Dependent(dep_str)
                            } else {
                                err.line = bracket_pos.line;
                                err.col = bracket_pos.col;
                                return Err(err);
                            }
                        }
                    }
                }
            } else {
                ArrayDim::Fixed(0)
            };
            self.expect(TokenKind::RBracket)?;
            dimensions.push(dim);
        }
        let mut cur_type = elem_type;
        for dim in dimensions.into_iter().rev() {
            match dim {
                ArrayDim::Fixed(size) => cur_type = Type::Array(Box::new(cur_type), size),
                ArrayDim::Dependent(param) => {
                    cur_type = Type::DependentArray(Box::new(cur_type), param);
                }
            }
        }
        Ok(cur_type)
    }
}
