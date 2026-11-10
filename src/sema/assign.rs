use super::*;

impl Sema {
    pub fn is_null_pointer_constant(expr: &Expr) -> bool {
        match expr {
            Expr::IntLiteral(0, ty) => ty.is_integer() || matches!(ty, Type::Nullptr),
            _ => false,
        }
    }

    pub fn can_assign(&self, target: &Type, value: &Type) -> bool {
        self.can_assign_expr(target, value, None)
    }

    pub(crate) fn reference_types_compatible(&self, target: &Type, source: &Type) -> bool {
        if target == source {
            return true;
        }
        if let (Type::Struct(s1), Type::Struct(s2)) = (target, source) {
            let p1 = QualifiedName::new(s1).plain();
            let p2 = QualifiedName::new(s2).plain();
            p1 == p2 || self.is_subclass(p2, p1)
        } else {
            false
        }
    }

    pub(super) fn instantiate_assign_operator(
        &mut self,
        target_ty: &Type,
        value_ty: &Type,
        value: &Expr,
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        if let Type::Struct(sname) = target_ty.strip_reference().strip_const()
            && let Some(sig) = self.select_member_overload(
                (sname, "operator="),
                (false, true),
                std::slice::from_ref(value_ty),
                Some(std::slice::from_ref(value)),
                pos,
            )?
        {
            self.instantiate_pending_method(&sig.mangled_name)?;
        }
        Ok(())
    }

    fn can_assign_reference(
        &self,
        target: &Type,
        value: &Type,
        expr: Option<&Expr>,
    ) -> Option<bool> {
        if let Type::RvalueReference(inner) = target {
            if expr.is_some_and(Expr::is_lvalue) || value.is_lvalue_reference() {
                return Some(false);
            }
            if value.strip_reference().is_const() && !inner.is_const() {
                return Some(false);
            }
            return Some(
                self.reference_types_compatible(
                    inner.strip_const(),
                    value.strip_reference().strip_const(),
                ) || self.can_assign(inner.strip_const(), value.strip_reference().strip_const()),
            );
        }
        if let Type::Reference(inner) = target {
            if inner.is_const() {
                return Some(self.can_assign_expr(inner.strip_const(), value, expr));
            }
            if value.is_rvalue_reference() {
                return Some(false);
            }
            if let Some(e) = expr
                && !value.is_reference()
                && !e.is_lvalue()
            {
                return Some(false);
            }
            if value.strip_reference().is_const() {
                return Some(false);
            }
            return Some(self.reference_types_compatible(
                inner.strip_const(),
                value.strip_reference().strip_const(),
            ));
        }
        None
    }

    pub fn can_assign_expr(&self, target: &Type, value: &Type, expr: Option<&Expr>) -> bool {
        if let Some(ok) = self.can_assign_reference(target, value, expr) {
            return ok;
        }
        let is_lvalue_val = expr.is_some_and(Expr::is_lvalue) || value.is_lvalue_reference();
        let target = strip_assign_type(target);
        let value = strip_assign_type(value);
        if target == value {
            if let Type::Struct(sname) = target.strip_const()
                && self.is_move_only_struct(sname)
                && (expr.is_none() || is_lvalue_val)
            {
                return false;
            }
            return true;
        }
        if matches!(target.strip_const(), Type::Nullptr)
            && let Some(e) = expr
            && Self::is_null_pointer_constant(e)
        {
            return true;
        }
        if matches!(value.strip_const(), Type::Nullptr) {
            return target.strip_const().is_pointer() || matches!(target.strip_const(), Type::Bool);
        }
        if let Type::Enum(ename) = value
            && self.scoped_enums.contains(ename)
        {
            return false;
        }
        if let Type::Enum(ename) = target
            && self.scoped_enums.contains(ename)
        {
            return false;
        }
        if self.struct_assignable(target, value) {
            return true;
        }
        if let Type::Struct(sname) = value.strip_reference().strip_const() {
            let is_const = value.strip_reference().is_const();
            if self.find_conversion_to(sname, target, is_const).is_some() {
                return true;
            }
        }
        if target.is_integer() && value.is_integer() {
            return true;
        }
        if (target.is_floating() && value.is_integer())
            || (target.is_integer() && value.is_floating())
            || (target.is_floating() && value.is_floating())
        {
            return true;
        }
        if self.pointer_assignable(target, value, expr) {
            return true;
        }
        if let (Type::Reference(t_in), Type::Reference(v_in)) = (target, value)
            && let (Type::Struct(base), Type::Struct(derived)) =
                (t_in.strip_const(), v_in.strip_const())
            && self.is_subclass(derived, base)
        {
            return true;
        }
        false
    }

    /// Returns true when the conversion to the parameter type uses a
    /// converting constructor of a class.
    ///
    /// A user-defined conversion is worse than a standard conversion, so the
    /// two need different ranks in the overload resolution.
    pub(crate) fn is_user_conversion(&self, param: &Type, arg: &Type) -> bool {
        let param = param.strip_reference().strip_const();
        let Type::Struct(sname) = param else {
            return false;
        };
        let short_name = sname.split("::").last().unwrap_or(sname);
        let q1 = format!("{sname}::{short_name}");
        let q2 = format!("std::{short_name}::{short_name}");
        let q3 = format!("{short_name}::{short_name}");
        for q in &[&q1, &q2, &q3] {
            let Some(sigs) = self.functions.get(*q) else {
                continue;
            };
            for sig in sigs {
                let params = if sig.param_types.is_empty() {
                    &[][..]
                } else {
                    &sig.param_types[1..]
                };
                if params.len() != 1 {
                    continue;
                }
                let converted = params[0].strip_reference().strip_const();
                if converted != param && self.can_assign(&params[0], arg) {
                    return true;
                }
            }
        }
        false
    }

    fn struct_assignable(&self, target: &Type, value: &Type) -> bool {
        let s_target = if let Type::Struct(s) = target {
            Some(QualifiedName::new(s).plain())
        } else {
            None
        };
        let s_val = if let Type::Struct(s) = value {
            Some(QualifiedName::new(s).plain())
        } else {
            None
        };
        if let (Some(s1), Some(s2)) = (s_target, s_val)
            && (s1 == s2 || self.is_subclass(s2, s1))
        {
            return true;
        }
        if let Type::Struct(sname) = target {
            let short_name = sname.split("::").last().unwrap_or(sname);
            let q1 = format!("{}::{}", sname, short_name);
            let q2 = format!("std::{}::{}", short_name, short_name);
            let q3 = format!("{}::{}", short_name, short_name);
            for q in &[&q1, &q2, &q3] {
                if let Some(sigs) = self.functions.get(*q) {
                    for sig in sigs {
                        let params = if !sig.param_types.is_empty() {
                            &sig.param_types[1..]
                        } else {
                            &[]
                        };
                        if params.len() == 1 {
                            let p = &params[0];
                            let p_clean = p.strip_reference().strip_const();
                            if p_clean != target
                                && p_clean != target.strip_reference().strip_const()
                                && self.can_assign(p, value)
                            {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        if matches!(target, Type::Struct(_)) && value == &Type::Void {
            return true;
        }
        if let Type::Array(_, _) = target
            && value == &Type::Void
        {
            return true;
        }
        false
    }

    fn pointer_assignable(&self, target: &Type, value: &Type, expr: Option<&Expr>) -> bool {
        if !target.strip_const().is_pointer() {
            return false;
        }
        if let Some(e) = expr
            && Self::is_null_pointer_constant(e)
        {
            return true;
        }
        if matches!(value.strip_const(), Type::Nullptr) {
            return true;
        }
        let target_decay = target.strip_const().decay_array();
        let value_decay = value.strip_const().decay_array();
        self.check_pointer_conversion(&target_decay, &value_decay)
    }

    pub fn check_pointer_conversion(&self, target: &Type, value: &Type) -> bool {
        let (levels_to, base_to) = decompose_pointer(target);
        let (levels_from, base_from) = decompose_pointer(value);

        if levels_to.is_empty() || levels_from.is_empty() {
            return false;
        }

        if levels_to.len() == 1 && base_to == &Type::Void {
            let from_pointee_const = levels_from[0];
            let to_pointee_const = levels_to[0];
            if from_pointee_const && !to_pointee_const {
                return false;
            }
            return true;
        }

        if levels_to.len() != levels_from.len() {
            return false;
        }

        let n = levels_to.len();
        for i in 0..n {
            let c_from = levels_from[i];
            let c_to = levels_to[i];
            if c_from && !c_to {
                return false;
            }
            if c_to && !c_from && levels_to[..i].iter().any(|&c| !c) {
                return false;
            }
        }

        if base_to == base_from {
            return true;
        }
        if n == 1
            && let (Type::Struct(base), Type::Struct(derived)) = (base_to, base_from)
            && self.is_subclass(derived, base)
        {
            return true;
        }
        false
    }

    /// Finds a non-explicit conversion operator from a struct to `target_ty`.
    pub fn find_conversion_to(
        &self,
        sname: &str,
        target_ty: &Type,
        is_const: bool,
    ) -> Option<FunctionSig> {
        let conv_name = format!("operator {}", target_ty.to_c_type());
        if let Some(sig) = self.find_method_in_class_for_receiver(sname, &conv_name, is_const)
            && !sig.is_explicit
        {
            return Some(sig);
        }
        if target_ty.is_integer()
            && let Some(sig) =
                self.find_method_in_class_for_receiver(sname, "operator int", is_const)
            && !sig.is_explicit
        {
            return Some(sig);
        }
        if let Type::Pointer(inner) = target_ty.strip_const()
            && let Type::Function {
                return_type,
                param_types,
            } = &**inner
            && let Some(sig) =
                self.find_method_in_class_for_receiver(sname, "__to_fn_ptr", is_const)
            && let Type::Pointer(sig_inner) = &sig.return_type
            && let Type::Function {
                return_type: sig_ret,
                param_types: sig_params,
            } = &**sig_inner
            && return_type == sig_ret
            && param_types == sig_params
        {
            return Some(sig);
        }
        None
    }

    /// Wraps an expression in a user-defined conversion call node.
    pub fn wrap_conversion_call(expr: &mut Expr, conv_name: &str, pos: ExprPos) {
        let old = std::mem::replace(expr, Expr::BoolLiteral(false));
        *expr = Expr::Call {
            callee: Box::new(Expr::MemberAccess {
                object: Box::new(old),
                member: conv_name.to_string(),
                is_arrow: false,
                pos,
            }),
            args: vec![],
            pos,
        };
    }
}

fn strip_assign_type(ty: &Type) -> &Type {
    let ty = ty.strip_const();
    if let Type::Reference(inner) | Type::RvalueReference(inner) = ty {
        inner.strip_const()
    } else {
        ty
    }
}

fn decompose_pointer(mut ty: &Type) -> (Vec<bool>, &Type) {
    let mut levels = Vec::new();
    while let Type::Pointer(inner) = ty.strip_const() {
        levels.push(inner.is_const());
        ty = inner.strip_const();
    }
    (levels, ty)
}
