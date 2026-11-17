//! Const-receiver checks for functions.

use super::*;

/// Returns true when a type refers to a const-qualified object, looking
/// through references and pointers.
pub(crate) fn receiver_is_const_type(ty: &Type) -> bool {
    match ty {
        Type::Const(_) => true,
        Type::Reference(inner) | Type::RvalueReference(inner) | Type::Pointer(inner) => {
            receiver_is_const_type(inner)
        }
        _ => false,
    }
}

/// Returns true when a non-static method's implicit `this` is `const`.
pub(crate) fn sig_receiver_is_const(sig: &FunctionSig) -> bool {
    !sig.is_static
        && sig
            .param_types
            .first()
            .map(receiver_is_const_type)
            .unwrap_or(false)
}
