//! Helpers that map C++ types onto the Wasm value types used at call sites.

use super::*;

/// The Wasm value type used to pass a value of `ty`.
pub(crate) fn wasm_value_type(ty: &Type) -> u8 {
    ty.to_wasm_type()
        .map(|w| match w {
            WasmType::I32 => TYPE_I32,
            WasmType::I64 => TYPE_I64,
            WasmType::F32 => TYPE_F32,
            WasmType::F64 => TYPE_F64,
        })
        .unwrap_or(TYPE_I32)
}

/// The Wasm result types of a function returning `ty`.
pub(crate) fn wasm_result_types(ty: &Type) -> Vec<u8> {
    ty.to_wasm_type()
        .map(|w| match w {
            WasmType::I32 => vec![TYPE_I32],
            WasmType::I64 => vec![TYPE_I64],
            WasmType::F32 => vec![TYPE_F32],
            WasmType::F64 => vec![TYPE_F64],
        })
        .unwrap_or_default()
}

/// The Wasm parameter types of a function with the given parameter types.
pub(crate) fn wasm_params_from_types(params: &[Type]) -> Vec<u8> {
    params.iter().map(wasm_value_type).collect()
}

/// Applies an overrider's covariant-return adjustment to a returned
/// pointer/reference value on the stack.
pub(crate) fn emit_return_adjustment(adjust_slot: u32, ctx: &FunctionContext, code: &mut Vec<u8>) {
    code.push(OP_LOCAL_GET);
    write_u32(code, ctx.fp_local);
    code.push(OP_I32_LOAD);
    write_u32(code, 2);
    write_u32(code, adjust_slot);
    code.push(OP_I32_ADD);
}

/// The return type of a callable value: a function, a function pointer, a
/// pointer to member function, or a bound member function.
pub(crate) fn callee_return_type(callee_ty: &Type) -> Type {
    match callee_ty.strip_const() {
        Type::Pointer(inner) => match &**inner {
            Type::Function { return_type, .. } => (**return_type).clone(),
            _ => Type::Int,
        },
        Type::Function { return_type, .. } => (**return_type).clone(),
        Type::MemberPointer { member_type, .. } | Type::BoundMemberFunction { member_type, .. } => {
            callee_return_type(member_type)
        }
        _ => Type::Int,
    }
}
