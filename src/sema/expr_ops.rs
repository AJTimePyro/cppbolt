use super::*;

impl Sema {
    /// Result type of a binary operator once both operand types are known.
    ///
    /// Kept separate from `analyze_binary` so left-associative operator chains
    /// can be type-checked by walking the chain iteratively instead of
    /// recursing once per term.
    pub(super) fn binary_result_type(&mut self, op: &BinaryOp, lty: &Type, rty: &Type) -> Type {
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
            let op_cands = self.find_call_candidates(op_str, &[lty.clone(), rty.clone()]);
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
        let mut spine: Vec<(BinaryOp, Expr, ExprPos)> = Vec::new();
        loop {
            let Some((op, left_owned, right_owned, pos)) = (if let Expr::Binary {
                op,
                left,
                right,
                pos,
            } = &mut owned
            {
                let op = *op;
                let pos = *pos;
                Some((
                    op,
                    std::mem::replace(&mut **left, Expr::BoolLiteral(false)),
                    std::mem::replace(&mut **right, Expr::BoolLiteral(false)),
                    pos,
                ))
            } else {
                None
            }) else {
                break;
            };
            spine.push((op, right_owned, pos));
            owned = left_owned;
        }

        let mut base = owned;
        let mut lty = self.analyze_expr(&mut base)?;
        for (op, right, pos) in spine.iter_mut().rev() {
            let rty = self.analyze_expr(right)?;
            // Adding two pointers is not a C++ expression.
            if *op == BinaryOp::Add
                && lty.decay_array().is_pointer()
                && rty.decay_array().is_pointer()
            {
                return Err(CompileError::new(
                    "invalid operands to binary expression: two pointers cannot be added",
                    pos.line,
                    pos.col,
                ));
            }
            lty = self.binary_result_type(op, &lty, &rty);
        }

        let mut rebuilt = base;
        for (op, right, pos) in spine.into_iter().rev() {
            rebuilt = Expr::Binary {
                op,
                left: Box::new(rebuilt),
                right: Box::new(right),
                pos,
            };
        }
        *expr = rebuilt;
        Ok(lty)
    }

    pub(super) fn analyze_unary(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::Unary { op, expr, pos } = expr else {
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
                        pos.line,
                        pos.col,
                    ))
                }
            }
            UnaryOp::AddrOf => {
                if let Type::Pointer(inner) = &ty
                    && let Type::Function { .. } = &**inner
                {
                    return Ok(ty);
                }
                // A class operand may name an overloaded `operator&`.
                if let Some(sig) = self.find_unary_operator(UnaryOp::AddrOf, &ty) {
                    return Ok(sig.return_type);
                }
                Ok(Type::Pointer(Box::new(ty)))
            }
            UnaryOp::BitNot | UnaryOp::Neg | UnaryOp::Plus => {
                // A class operand may name an overloaded operator.
                if let Some(sig) = self.find_unary_operator(*op, &ty) {
                    return Ok(sig.return_type);
                }
                Ok(ty.integer_promotion())
            }
            UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::PostInc | UnaryOp::PostDec => Ok(ty),
        }
    }

    /// Finds the operator that a unary operator on `operand` resolves to.
    ///
    /// A member operator of the operand's class wins over a free operator
    /// function.  Only a class operand can name an operator, so the built-in
    /// operators are never shadowed by an overload.
    pub(super) fn find_unary_operator(&self, op: UnaryOp, operand: &Type) -> Option<FunctionSig> {
        let name = match op {
            UnaryOp::Neg => "operator-",
            UnaryOp::Not => "operator!",
            UnaryOp::BitNot => "operator~",
            UnaryOp::Plus => "operator+",
            UnaryOp::AddrOf => "operator&",
            _ => return None,
        };
        let Type::Struct(class_name) = operand.strip_const() else {
            return None;
        };
        let receiver_const = receiver_is_const_type(operand);
        if let Some(sig) = self.find_method_in_class_for_receiver(class_name, name, receiver_const)
        {
            return Some(sig);
        }
        self.find_free_operator(name, operand)
    }

    /// Finds a free operator function whose first parameter binds `operand`.
    pub(super) fn find_free_operator(&self, name: &str, operand: &Type) -> Option<FunctionSig> {
        let sigs = self.functions.get(name)?;
        let mut best: Option<FunctionSig> = None;
        let mut best_score = usize::MAX;
        for sig in sigs {
            let Some(param) = sig.param_types.first() else {
                continue;
            };
            let score = if param.strip_all_const() == operand.strip_all_const() {
                0
            } else if param.is_reference()
                && param.deref_type().map(|d| d.strip_all_const())
                    == Some(operand.strip_all_const())
            {
                1
            } else {
                continue;
            };
            if score < best_score {
                best_score = score;
                best = Some(sig.clone());
            }
        }
        best
    }

    pub(super) fn analyze_assign_expr(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::Assign { target, value, pos } = expr else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        let target_ty = self.analyze_expr_keeping_reference(target)?;
        if !is_assignable_target(target, &target_ty) {
            return Err(CompileError::new(
                "lvalue required as left operand of assignment",
                pos.line,
                pos.col,
            ));
        }
        let target_ty = target_ty.strip_reference().clone();
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
                pos.line,
                pos.col,
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
                pos.line,
                pos.col,
            ));
        }
        Ok(target_ty)
    }

    pub(super) fn analyze_compound_assign(
        &mut self,
        expr: &mut Expr,
    ) -> Result<Type, CompileError> {
        let Expr::CompoundAssign {
            target, value, pos, ..
        } = expr
        else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        let target_ty = self.analyze_expr_keeping_reference(target)?;
        if !is_assignable_target(target, &target_ty) {
            return Err(CompileError::new(
                "lvalue required as left operand of assignment",
                pos.line,
                pos.col,
            ));
        }
        let _value_ty = self.analyze_expr(value)?;
        Ok(target_ty.strip_reference().clone())
    }

    pub(super) fn analyze_subscript(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::ArraySubscript { array, index, pos } = expr else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        let arr_ty = self.analyze_expr(array)?;
        let _idx_ty = self.analyze_expr(index)?;
        let receiver_const = receiver_is_const_type(&arr_ty);
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
                pos.line,
                pos.col,
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

/// Returns true when an expression names a storage location that an
/// assignment may modify.
///
/// An expression that returns a reference names a storage location too, so a
/// call such as `get_ref() = 1` is assignable.  Every other expression must
/// name an object, a member, an element, or the object a pointer refers to.
pub(super) fn is_assignable_target(expr: &Expr, ty: &Type) -> bool {
    ty.is_reference() || is_modifiable_expression(expr)
}

/// Returns true when the expression itself names a modifiable object.
///
/// A compound assignment names the object it modifies, so `(a += b) += c`
/// modifies the same object as `a += b`.
fn is_modifiable_expression(expr: &Expr) -> bool {
    match expr {
        Expr::Identifier(_) | Expr::MemberAccess { .. } | Expr::ArraySubscript { .. } => true,
        Expr::Unary {
            op: UnaryOp::Deref, ..
        } => true,
        Expr::CompoundAssign { target, .. } => is_modifiable_expression(target),
        _ => false,
    }
}
