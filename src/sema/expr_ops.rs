use super::*;

fn builtin_binary_result_type(op: &BinaryOp, lty: &Type, rty: &Type) -> Type {
    match op {
        BinaryOp::LogicalAnd
        | BinaryOp::LogicalOr
        | BinaryOp::Eq
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

/// Splits the left-associative binary spine of `expr` into its base operand
/// and trailing (operator, right operand, position) triples.
fn unwind_binary_spine(expr: &mut Expr) -> (Expr, Vec<(BinaryOp, Expr, ExprPos)>) {
    let placeholder = Expr::IntLiteral(0, Type::Int);
    let mut owned = std::mem::replace(expr, placeholder);
    let mut spine = Vec::new();
    while let Expr::Binary {
        op,
        left,
        right,
        pos,
    } = &mut owned
    {
        let op = *op;
        let pos = *pos;
        let left_owned = std::mem::replace(&mut **left, Expr::BoolLiteral(false));
        let right_owned = std::mem::replace(&mut **right, Expr::BoolLiteral(false));
        spine.push((op, right_owned, pos));
        owned = left_owned;
    }
    (owned, spine)
}

impl Sema {
    /// Result type of a binary operator once both operand types are known.
    ///
    /// Kept separate from `analyze_binary` so left-associative operator chains
    /// can be type-checked by walking the chain iteratively instead of
    /// recursing once per term.
    pub(super) fn binary_result_type(
        &mut self,
        op: &BinaryOp,
        lty: &Type,
        rty: &Type,
        dependent: bool,
        pos: ExprPos,
    ) -> Result<Type, CompileError> {
        let clean_lty = lty.strip_reference().strip_const();
        let clean_rty = rty.strip_reference().strip_const();
        if (matches!(clean_lty, Type::Struct(_) | Type::Enum(_))
            || matches!(clean_rty, Type::Struct(_) | Type::Enum(_)))
            && let Some(op_str) = op.op_name()
            && let Some(res) =
                self.resolve_overloaded_binary_type(op_str, lty, rty, clean_lty, dependent)
        {
            return Ok(res);
        }

        if *op == BinaryOp::Comma {
            return Ok(rty.clone());
        }
        if *op == BinaryOp::DotStar || *op == BinaryOp::ArrowStar {
            return Ok(self
                .check_member_pointer_binary(*op, lty, rty, ExprPos::default())
                .unwrap_or(Type::Int));
        }

        let (l_clean, r_clean) = (
            lty.strip_reference().strip_const(),
            rty.strip_reference().strip_const(),
        );
        if let Type::Struct(s) = l_clean
            && !matches!(r_clean, Type::Struct(_))
            && let Some(sig) = self.find_conversion_to(s, r_clean, lty.strip_reference().is_const())
        {
            let conv = sig.return_type.strip_reference().clone();
            return self.binary_result_type(op, &conv, rty, dependent, pos);
        }
        if let Type::Struct(s) = r_clean
            && !matches!(l_clean, Type::Struct(_))
            && let Some(sig) = self.find_conversion_to(s, l_clean, rty.strip_reference().is_const())
        {
            let conv = sig.return_type.strip_reference().clone();
            return self.binary_result_type(op, lty, &conv, dependent, pos);
        }

        // A class-type operand needs an overload. In a template, a
        // non-dependent operator expression must find it at the definition
        // context; leaving it unresolved would let codegen bind a later
        // declaration that ordinary lookup cannot see.
        if !dependent
            && self.current_template_identity.is_some()
            && op.op_name().is_some()
            && (matches!(l_clean, Type::Struct(_)) || matches!(r_clean, Type::Struct(_)))
        {
            return Err(CompileError::unresolved_name(
                "invalid operands to binary expression",
                pos.line,
                pos.col,
            ));
        }

        Ok(builtin_binary_result_type(op, lty, rty))
    }

    pub(super) fn analyze_binary(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        if !matches!(expr, Expr::Binary { .. }) {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        }

        // Take the left-associative spine apart so a chain of `n` operators
        // needs one stack frame, not `n`.
        let (mut base, mut spine) = unwind_binary_spine(expr);
        let mut lty = self.analyze_expr(&mut base)?;
        for (op, right, pos) in spine.iter_mut().rev() {
            let mut rty = self.analyze_expr(right)?;
            if *op == BinaryOp::DotStar || *op == BinaryOp::ArrowStar {
                lty = self.check_member_pointer_binary(*op, &lty, &rty, *pos)?;
                continue;
            }
            let dependent = self.expression_is_dependent(*pos);
            self.apply_binary_conversions(
                *op,
                (&mut base, &mut lty),
                (right, &mut rty),
                *pos,
                dependent,
            );
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
            lty = self.binary_result_type(op, &lty, &rty, dependent, *pos)?;
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

    fn apply_binary_conversions(
        &mut self,
        op: BinaryOp,
        left: (&mut Expr, &mut Type),
        right: (&mut Expr, &mut Type),
        pos: ExprPos,
        dependent: bool,
    ) {
        let (base, lty) = left;
        let (right, rty) = right;
        let clean_lty = lty.strip_reference().strip_const().clone();
        let clean_rty = rty.strip_reference().strip_const().clone();
        let has_overload = op.op_name().is_some_and(|op_str| {
            if let Type::Struct(ref sname) = clean_lty
                && self.find_method_in_class(sname, op_str).is_some()
            {
                return true;
            }
            self.find_call_candidates(op_str, &[lty.clone(), rty.clone()], dependent)
                .is_some()
        });
        if has_overload || op == BinaryOp::Comma {
            return;
        }
        if let Type::Struct(ref sname) = clean_lty
            && !matches!(clean_rty, Type::Struct(_))
        {
            let is_const = lty.strip_reference().is_const();
            if let Some(sig) = self.find_conversion_to(sname, &clean_rty, is_const) {
                Self::wrap_conversion_call(base, &sig.name, pos);
                *lty = sig.return_type.strip_reference().clone();
            }
        }
        if let Type::Struct(ref sname) = clean_rty
            && !matches!(clean_lty, Type::Struct(_))
        {
            let is_const = rty.strip_reference().is_const();
            if let Some(sig) = self.find_conversion_to(sname, &clean_lty, is_const) {
                Self::wrap_conversion_call(right, &sig.name, pos);
                *rty = sig.return_type.strip_reference().clone();
            }
        }
    }

    pub(super) fn analyze_unary(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::Unary { op, expr, pos } = expr else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        if let Some(ty) = self.resolve_member_pointer_address(*op, expr, *pos)? {
            return Ok(ty);
        }

        let ty = self.analyze_expr(expr)?;
        let dependent = self.expression_is_dependent(*pos);
        match op {
            UnaryOp::Not => {
                if let Type::Struct(sname) = ty.strip_const() {
                    if let Some(sig) = self.find_unary_operator(UnaryOp::Not, &ty, dependent) {
                        self.instantiate_pending_method(&sig.mangled_name)?;
                        return Ok(sig.return_type);
                    }
                    // `!x` may also use a contextual conversion to bool.
                    let has_bool_conv = self.find_method_in_class(sname, "operator bool").is_some();
                    if !dependent && self.current_template_identity.is_some() && !has_bool_conv {
                        return Err(CompileError::unresolved_name(
                            "invalid argument type to unary expression",
                            pos.line,
                            pos.col,
                        ));
                    }
                }
                Ok(Type::Bool)
            }
            UnaryOp::Deref => self.analyze_deref(&ty, *pos, dependent),
            UnaryOp::AddrOf => {
                if let Type::Pointer(inner) = &ty
                    && let Type::Function { .. } = &**inner
                {
                    return Ok(ty);
                }
                // A class operand may name an overloaded `operator&`.
                if let Some(sig) = self.find_unary_operator(UnaryOp::AddrOf, &ty, dependent) {
                    self.instantiate_pending_method(&sig.mangled_name)?;
                    return Ok(sig.return_type);
                }
                Ok(Type::Pointer(Box::new(ty)))
            }
            UnaryOp::BitNot | UnaryOp::Neg | UnaryOp::Plus => {
                // A class operand may name an overloaded operator.
                if let Some(sig) = self.find_unary_operator(*op, &ty, dependent) {
                    self.instantiate_pending_method(&sig.mangled_name)?;
                    return Ok(sig.return_type);
                }
                if *op == UnaryOp::Plus
                    && let Type::Struct(sname) = ty.strip_const()
                    && let Some(conv_sig) =
                        self.find_method_in_class_for_receiver(sname, "__to_fn_ptr", ty.is_const())
                {
                    Self::wrap_conversion_call(expr, &conv_sig.name, *pos);
                    return Ok(conv_sig.return_type);
                }
                if matches!(ty.strip_const(), Type::Struct(_)) {
                    return Err(CompileError::new(
                        "invalid argument type to unary expression",
                        pos.line,
                        pos.col,
                    ));
                }
                Ok(ty.integer_promotion())
            }
            UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::PostInc | UnaryOp::PostDec => {
                self.analyze_unary_incdec(op, &ty, *pos)
            }
        }
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
        if !self.is_assignable_target(target, &target_ty) {
            return Err(CompileError::new(
                "lvalue required as left operand of assignment",
                pos.line,
                pos.col,
            ));
        }
        let target_ty = target_ty.strip_reference().clone();
        if target_ty.is_const() {
            let name = match &**target {
                Expr::Identifier(n) => n.as_str(),
                Expr::MemberAccess { member, .. } => member.as_str(),
                _ => "variable",
            };
            return Err(CompileError::new(
                format!("Cannot assign to variable '{name}' with const-qualified type"),
                pos.line,
                pos.col,
            ));
        }
        if let Expr::InitializerList { elements, pos } = &mut **value {
            let ctx = InitContext::CopyListAssign;
            return self.resolve_list_init(&target_ty, elements, ctx, *pos);
        }
        let expected = is_member_pointer_address(value).then_some(&target_ty);
        let mut value_ty = self.with_expected_type(expected, |s| s.analyze_expr(value))?;
        if let Type::Struct(sname) = value_ty.strip_reference().strip_const() {
            let is_const = value_ty.strip_reference().is_const();
            if let Some(sig) = self.find_conversion_to(sname, &target_ty, is_const) {
                Self::wrap_conversion_call(value, &sig.name, *pos);
                value_ty = target_ty.clone();
            }
        }
        if !self.can_assign_expr(&target_ty, &value_ty, Some(value)) {
            let msg = format!(
                "Type mismatch in assignment: cannot assign {} to {}",
                value_ty.to_c_type(),
                target_ty.to_c_type()
            );
            return Err(CompileError::new(msg, pos.line, pos.col));
        }
        self.instantiate_assign_operator(&target_ty, &value_ty, value, *pos)?;
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
        if !self.is_assignable_target(target, &target_ty) {
            return Err(CompileError::new(
                "lvalue required as left operand of assignment",
                pos.line,
                pos.col,
            ));
        }
        let _value_ty = self.analyze_expr(value)?;
        Ok(target_ty.strip_reference().clone())
    }

    fn resolve_overloaded_binary_type(
        &mut self,
        op_str: &str,
        lty: &Type,
        rty: &Type,
        clean_lty: &Type,
        dependent: bool,
    ) -> Option<Type> {
        if let Type::Struct(sname) = clean_lty
            && let Some(sig) = self.find_method_in_class(sname, op_str)
        {
            let _ = self.instantiate_pending_method(&sig.mangled_name);
            return Some(sig.return_type.clone());
        }
        let args = [lty.clone(), rty.clone()];
        if let Some(sigs) = self.find_call_candidates(op_str, &args, dependent)
            && let Some(sig) = sigs
                .iter()
                .find(|s| self.score_call_candidate(s, &args, None).is_ok())
        {
            return Some(sig.return_type.clone());
        }
        let pos = ExprPos::new(self.current_line, self.current_col);
        if let Ok(Some(sig)) = self.try_resolve_function_template_call(op_str, &args, &[], pos) {
            return Some(sig.return_type);
        }
        let std_op = format!("std::{}", op_str);
        if let Ok(Some(sig)) = self.try_resolve_function_template_call(&std_op, &args, &[], pos) {
            return Some(sig.return_type);
        }
        None
    }
}
