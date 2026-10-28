use super::*;

impl Sema {
    pub fn can_assign(&self, target: &Type, value: &Type) -> bool {
        let target = strip_assign_type(target);
        let value = strip_assign_type(value);
        if target == value {
            return true;
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
        if self.pointer_assignable(target, value) {
            return true;
        }
        if let (Type::Reference(t_in), Type::Reference(v_in)) = (target, value)
            && let (Type::Struct(base), Type::Struct(derived)) =
                (t_in.strip_const(), v_in.strip_const())
            && self.is_subclass(derived, base)
        {
            return true;
        }
        if let Type::Reference(inner) = target {
            return self.can_assign(inner, value);
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

    fn pointer_assignable(&self, target: &Type, value: &Type) -> bool {
        let Type::Pointer(t_elem) = target else {
            return false;
        };
        if value.is_integer() {
            // Accept integers as potential null pointers; this subset does
            // not track the constant-expression rule here.
            return true;
        }
        if let Type::Pointer(v_elem) = value {
            if t_elem.strip_const() == &Type::Void || v_elem.strip_const() == &Type::Void {
                return true;
            }
            if t_elem.strip_const() == v_elem.strip_const() {
                return true;
            }
            if let (Type::Struct(base), Type::Struct(derived)) =
                (t_elem.strip_const(), v_elem.strip_const())
                && self.is_subclass(derived, base)
            {
                return true;
            }
        }
        if let Type::Array(v_elem, _) = value
            && (t_elem.strip_const() == &Type::Void || t_elem.strip_const() == v_elem.strip_const())
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
    if let Type::Reference(inner) = ty {
        inner.strip_const()
    } else {
        ty
    }
}
