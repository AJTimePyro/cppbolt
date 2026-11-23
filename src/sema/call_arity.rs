//! Arity checks for indirect calls.

use super::*;

/// The function type behind a function pointer, member-function pointer, or
/// bound member function.
fn indirect_function_type(ty: &Type) -> Option<(&Type, bool)> {
    let clean = ty.strip_reference().strip_const();
    match clean {
        Type::Pointer(inner) => indirect_function_type(inner),
        Type::Function { .. } => Some((clean, false)),
        Type::BoundMemberFunction { member_type, .. } => {
            indirect_function_type(member_type).map(|(f, _)| (f, true))
        }
        _ => None,
    }
}
/// Default arguments are a property of the declaration, not of the pointer,
/// so an indirect call must supply every parameter explicitly.
pub(super) fn check_indirect_call_arity(
    ty: &Type,
    arg_count: usize,
    pos: ExprPos,
) -> Result<(), CompileError> {
    let Some((fn_ty, is_member)) = indirect_function_type(ty) else {
        return Ok(());
    };
    let Type::Function { param_types, .. } = fn_ty else {
        return Ok(());
    };
    let expected = param_types.len();
    if arg_count == expected {
        return Ok(());
    }
    let kind = if is_member {
        "pointer-to-member function call"
    } else {
        "function pointer call"
    };
    let msg = if arg_count < expected {
        format!("too few arguments to {kind}, expected {expected}, have {arg_count}")
    } else {
        format!("too many arguments to {kind}, expected {expected}, have {arg_count}")
    };
    Err(CompileError::new(msg, pos.line, pos.col))
}
