use super::*;

impl Sema {
    /// Result type of a binary operator once both operand types are known.
    ///
    /// Kept separate from `analyze_binary` so left-associative operator chains
    /// can be type-checked by walking the chain iteratively instead of
    /// recursing once per term.
    pub(super) fn binary_result_type(&self, op: &BinaryOp, lty: &Type, rty: &Type) -> Type {
        if (matches!(lty.strip_const(), Type::Struct(_) | Type::Enum(_))
            || matches!(rty.strip_const(), Type::Struct(_) | Type::Enum(_)))
            && let Some(op_str) = op.op_name()
        {
            let clean_lty = lty.strip_const();
            if let Type::Struct(sname) = clean_lty
                && let Some(sig) = self.find_method_in_class(sname, op_str)
            {
                return sig.return_type.strip_reference().clone();
            }
            let mut op_cands = self
                .functions
                .get(op_str)
                .or_else(|| self.functions.get(&format!("std::{}", op_str)))
                .or_else(|| {
                    self.open_namespaces
                        .iter()
                        .find_map(|ns| self.functions.get(&format!("{}::{}", ns, op_str)))
                });
            if op_cands.is_none() {
                for aty in &[lty, rty] {
                    let clean = aty.strip_const().strip_reference();
                    if let Type::Struct(s) = clean
                        && let Some((ns, _)) = s.rsplit_once("::")
                        && let Some(sigs) = self.functions.get(&format!("{}::{}", ns, op_str))
                    {
                        op_cands = Some(sigs);
                        break;
                    }
                }
            }
            if let Some(sigs) = op_cands {
                return sigs[0].return_type.strip_reference().clone();
            }
        }

        match op {
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => Type::Bool,
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => Type::Bool,
            BinaryOp::Shl | BinaryOp::Shr => lty.strip_reference().integer_promotion(),
            _ => {
                // Pointer arithmetic: ptr + int or ptr - int
                let lty_d = lty.decay_array();
                let rty_d = rty.decay_array();
                if lty_d.is_pointer() && rty.is_integer() {
                    lty_d
                } else if rty_d.is_pointer() && lty.is_integer() && *op == BinaryOp::Add {
                    rty_d
                } else if lty_d.is_pointer() && rty_d.is_pointer() && *op == BinaryOp::Sub {
                    Type::Int
                } else {
                    lty.strip_reference()
                        .usual_arithmetic_conversion(rty.strip_reference())
                }
            }
        }
    }

    pub(super) fn analyze_binary(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::Binary { .. } = expr else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        // Take the left-associative spine apart so a chain of `n` operators
        // needs one stack frame, not `n`.
        let placeholder = Expr::IntLiteral(0, Type::Int);
        let mut owned = std::mem::replace(expr, placeholder);
        let mut spine: Vec<(BinaryOp, Expr)> = Vec::new();
        loop {
            let Some((op, left_owned, right_owned)) =
                (if let Expr::Binary { op, left, right } = &mut owned {
                    let op = *op;
                    Some((
                        op,
                        std::mem::replace(&mut **left, Expr::BoolLiteral(false)),
                        std::mem::replace(&mut **right, Expr::BoolLiteral(false)),
                    ))
                } else {
                    None
                })
            else {
                break;
            };
            spine.push((op, right_owned));
            owned = left_owned;
        }

        let mut base = owned;
        let mut lty = self.analyze_expr(&mut base)?;
        for (op, right) in spine.iter_mut().rev() {
            let rty = self.analyze_expr(right)?;
            lty = self.binary_result_type(op, &lty, &rty);
        }

        let mut rebuilt = base;
        for (op, right) in spine.into_iter().rev() {
            rebuilt = Expr::Binary {
                op,
                left: Box::new(rebuilt),
                right: Box::new(right),
            };
        }
        *expr = rebuilt;
        Ok(lty)
    }

    pub(super) fn analyze_unary(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::Unary { op, expr } = expr else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        let ty = self.analyze_expr(expr)?;
        match op {
            UnaryOp::Not => Ok(Type::Bool),
            UnaryOp::Deref => {
                if let Some(inner) = ty.deref_type() {
                    Ok(inner.clone())
                } else {
                    Err(CompileError::new(
                        format!("Cannot dereference non-pointer type {}", ty.to_c_type()),
                        1,
                        1,
                    ))
                }
            }
            UnaryOp::AddrOf => {
                if let Type::Pointer(inner) = &ty
                    && let Type::Function { .. } = &**inner
                {
                    return Ok(ty);
                }
                Ok(Type::Pointer(Box::new(ty)))
            }
            UnaryOp::BitNot | UnaryOp::Neg | UnaryOp::Plus => Ok(ty.integer_promotion()),
            UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::PostInc | UnaryOp::PostDec => Ok(ty),
        }
    }

    pub(super) fn analyze_assign_expr(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::Assign { target, value } = expr else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        let target_ty = self.analyze_expr(target)?;
        if target_ty.is_const() {
            let name = match &**target {
                Expr::Identifier(n) => n.clone(),
                _ => "variable".to_string(),
            };
            return Err(CompileError::new(
                format!(
                    "Cannot assign to variable '{}' with const-qualified type",
                    name
                ),
                self.current_line,
                self.current_col,
            ));
        }
        let value_ty = self.analyze_expr(value)?;
        if !self.can_assign(&target_ty, &value_ty) {
            return Err(CompileError::new(
                format!(
                    "Type mismatch in assignment: cannot assign {} to {}",
                    value_ty.to_c_type(),
                    target_ty.to_c_type()
                ),
                self.current_line,
                self.current_col,
            ));
        }
        Ok(target_ty)
    }

    pub(super) fn analyze_compound_assign(
        &mut self,
        expr: &mut Expr,
    ) -> Result<Type, CompileError> {
        let Expr::CompoundAssign { target, value, .. } = expr else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        let target_ty = self.analyze_expr(target)?;
        let _value_ty = self.analyze_expr(value)?;
        Ok(target_ty)
    }

    pub(super) fn analyze_subscript(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::ArraySubscript { array, index } = expr else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        let arr_ty = self.analyze_expr(array)?;
        let _idx_ty = self.analyze_expr(index)?;
        let receiver_const = crate::sema::receiver_is_const_type(&arr_ty);
        let clean_ty = arr_ty.strip_const();
        if let Type::Struct(sname) = clean_ty
            && let Some(sig) =
                self.find_method_in_class_for_receiver(sname, "operator[]", receiver_const)
        {
            return Ok(sig.return_type.strip_reference().clone());
        }
        if let Some(inner) = clean_ty.deref_type() {
            Ok(inner.strip_reference().clone())
        } else {
            Err(CompileError::new(
                format!(
                    "Subscripted value is not an array or pointer: {}",
                    arr_ty.to_c_type()
                ),
                1,
                1,
            ))
        }
    }

    pub(super) fn analyze_ternary(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } = expr
        else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        self.analyze_expr(cond)?;
        let t_ty = self.analyze_expr(then_expr)?;
        let e_ty = self.analyze_expr(else_expr)?;
        Ok(t_ty.conditional_result_type(&e_ty))
    }
}
