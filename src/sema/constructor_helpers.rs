//! Constructor overload scoring helpers.

use super::*;

pub(super) fn score_copy_init_param(param: &Type, arg_ty: &Type, arg_expr: Option<&Expr>) -> Option<usize> {
    let clean_param = param.deref_type()?.strip_all_const();
    let clean_arg = arg_ty.strip_reference().strip_all_const();
    let is_copy_param = match (clean_param, clean_arg) {
        (Type::Struct(p), Type::Struct(a)) => {
            QualifiedName::new(p.as_str()).matches(QualifiedName::new(a.as_str()))
        }
        (p, a) => p == a,
    };
    if !is_copy_param {
        return None;
    }
    let is_lval = arg_expr.is_some_and(Expr::is_lvalue) || arg_ty.is_lvalue_reference();
    if param.is_rvalue_reference() {
        if is_lval {
            return None;
        }
        Some(1)
    } else if let Some(inner) = param.deref_type() {
        if !inner.is_const() {
            if !is_lval {
                return None;
            }
            Some(1)
        } else {
            Some(if is_lval { 2 } else { 3 })
        }
    } else {
        None
    }
}

