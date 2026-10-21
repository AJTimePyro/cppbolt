use super::*;

impl Sema {
    pub fn can_assign(&self, target: &Type, value: &Type) -> bool {
        let target = strip_assign_type(target);
        let value = strip_assign_type(value);
        if target == value {
            return true;
        }
        if self.struct_assignable(target, value) {
            return true;
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
}

fn strip_assign_type(ty: &Type) -> &Type {
    let ty = ty.strip_const();
    if let Type::Reference(inner) = ty {
        inner.strip_const()
    } else {
        ty
    }
}
