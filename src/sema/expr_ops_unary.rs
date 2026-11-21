//! Unary operator analysis helpers.

use super::*;

impl Sema {
    /// Resolves `&Class::member` to a member-pointer type, rewriting the
    /// operand to the selected overload when one is found.
    pub(super) fn resolve_member_pointer_address(
        &mut self,
        op: UnaryOp,
        expr: &mut Box<Expr>,
        pos: ExprPos,
    ) -> Result<Option<Type>, CompileError> {
        if op != UnaryOp::AddrOf {
            return Ok(None);
        }
        let Expr::Identifier(name) = expr.as_ref() else {
            return Ok(None);
        };
        let Some(resolved) = self.try_resolve_member_pointer_address(name, pos)? else {
            return Ok(None);
        };
        if let Some(encoded_name) = resolved.encoded_name {
            // Taking the address of a template-specialization method must
            // force its deferred instantiation so codegen can emit it.
            if let Some((_, mangled)) = encoded_name.split_once("@@") {
                self.instantiate_pending_method(mangled)?;
            }
            if let Expr::Identifier(name) = &mut **expr {
                *name = encoded_name;
            }
        }
        Ok(Some(resolved.ty))
    }

    pub(super) fn analyze_deref(
        &mut self,
        ty: &Type,
        pos: ExprPos,
        dependent: bool,
    ) -> Result<Type, CompileError> {
        if let Some(inner) = ty.deref_type() {
            Ok(inner.clone())
        } else if let Some(sig) = self.find_unary_operator(UnaryOp::Deref, ty, dependent) {
            self.instantiate_pending_method(&sig.mangled_name)?;
            Ok(sig.return_type)
        } else {
            Err(CompileError::new(
                format!("Cannot dereference non-pointer type {}", ty.to_c_type()),
                pos.line,
                pos.col,
            ))
        }
    }

    pub(super) fn analyze_unary_incdec(
        &mut self,
        op: &UnaryOp,
        ty: &Type,
        pos: ExprPos,
    ) -> Result<Type, CompileError> {
        if let Type::Struct(sname) = ty.strip_const() {
            let m_name = if matches!(op, UnaryOp::PreInc | UnaryOp::PostInc) {
                "operator++"
            } else {
                "operator--"
            };
            let count = if matches!(op, UnaryOp::PreInc | UnaryOp::PreDec) {
                1
            } else {
                2
            };
            let receiver_const = receiver_is_const_type(ty);
            if let Some(sig) =
                self.find_method_with_param_count(sname, m_name, count, receiver_const)
            {
                let _ = self.instantiate_pending_method(&sig.mangled_name);
                return Ok(sig.return_type);
            }
            return Err(CompileError::new(
                format!("no matching '{}' for type '{}'", m_name, ty.to_c_type()),
                pos.line,
                pos.col,
            ));
        }
        Ok(ty.clone())
    }

    /// A member operator of the operand's class wins over a free operator
    /// function. Only a class operand can name an operator, so built-in
    /// operators are never shadowed by an overload.
    pub(super) fn find_unary_operator(
        &mut self,
        op: UnaryOp,
        operand: &Type,
        dependent: bool,
    ) -> Option<FunctionSig> {
        let name = match op {
            UnaryOp::Neg => "operator-",
            UnaryOp::Not => "operator!",
            UnaryOp::BitNot => "operator~",
            UnaryOp::Plus => "operator+",
            UnaryOp::AddrOf => "operator&",
            UnaryOp::Deref => "operator*",
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
        self.find_free_operator(name, operand, dependent)
    }

    /// Finds a free operator function whose first parameter binds `operand`.
    /// Lookup follows the same dependence rules as an ordinary call.
    pub(super) fn find_free_operator(
        &mut self,
        name: &str,
        operand: &Type,
        dependent: bool,
    ) -> Option<FunctionSig> {
        let sigs = self.find_call_candidates(name, std::slice::from_ref(operand), dependent)?;
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
}
