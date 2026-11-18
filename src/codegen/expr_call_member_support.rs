//! Support types for member call lowering.

use super::*;

pub(super) fn default_sig_from_class(class_name: &str) -> FunctionSig {
    FunctionSig {
        name: class_name.to_string(),
        mangled_name: class_name.to_string(),
        return_type: Type::Int,
        param_types: vec![Type::Pointer(Box::new(Type::Struct(
            class_name.to_string(),
        )))],
        default_args: Vec::new(),
        is_static: false,
        is_explicit: false,
        ref_qualifier: RefQualifier::None,
        qualifiers: FunctionQualifiers::default(),
    }
}

pub(super) struct MemberDispatch<'e> {
    pub(super) site: CallSite<'e>,
    pub(super) sname: &'e str,
    pub(super) member: &'e str,
    pub(super) virtual_slot: Option<(usize, u32)>,
    pub(super) resolved: Option<(u32, FunctionSig)>,
    pub(super) sig_for_key: Option<FunctionSig>,
}

#[derive(Clone, Copy)]
pub(super) struct VTableDispatch {
    pub(super) vptr_offset: u32,
    pub(super) slot_idx: usize,
    pub(super) this_slot: u32,
    pub(super) fn_slot: u32,
    pub(super) ret_adjust_slot: u32,
}

