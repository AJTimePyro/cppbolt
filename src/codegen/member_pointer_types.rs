//! Support types for pointer-to-member lowering.

use super::*;

pub(crate) struct MemberPointerCall<'a> {
    pub left: &'a Expr,
    pub right: &'a Expr,
    pub is_arrow: bool,
    pub args: &'a [Expr],
}

pub(crate) struct MemberPointerCmp<'a> {
    pub op: &'a BinaryOp,
    pub lty: &'a Type,
    pub rty: &'a Type,
    pub left: Option<&'a Expr>,
    pub right: &'a Expr,
}

pub(super) struct PmfSlots {
    pub(super) this: u32,
    pub(super) fn_slot: u32,
    pub(super) ret_adjust: u32,
}

