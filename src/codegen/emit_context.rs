//! Small context structs that group the values the lowering helpers need.
//!
//! The helpers used to take long parameter lists that repeated the same
//! frame slots, flags and declaration pieces at every call site.  Grouping
//! them keeps the signatures short, keeps the related values together, and
//! makes the call sites readable.

use super::*;

/// The receiver and the member being called.
pub(super) struct MemberAccess<'e> {
    pub(super) object: &'e Expr,
    pub(super) member: &'e str,
    pub(super) is_arrow: bool,
}

/// A call to a named function.
pub(super) struct NamedCall<'e> {
    pub(super) callee: &'e Expr,
    pub(super) name: &'e str,
    pub(super) class_name: Option<&'e str>,
}

/// A method call and the frame slots it uses.
///
/// The same struct serves the virtual and the direct lowering: the virtual
/// path uses `slot_idx` and `fn_slot`, the direct path uses the resolved
/// method index instead.
pub(super) struct CallSite<'e> {
    pub(super) class_name: &'e str,
    pub(super) member: &'e str,
    pub(super) args: &'e [Expr],
    pub(super) slot_idx: usize,
    pub(super) this_slot: u32,
    pub(super) fn_slot: u32,
    pub(super) receiver_const: bool,
}

/// A local array declaration being lowered.
pub(super) struct ArrayLocal<'e> {
    pub(super) elem_ty: &'e Type,
    pub(super) count: usize,
    pub(super) offset: u32,
    pub(super) name: &'e str,
    pub(super) init: &'e Option<Expr>,
    pub(super) stmt: &'e Stmt,
}

/// The operand types of a binary operator.
pub(super) struct BinaryTypes<'e> {
    pub(super) op: &'e BinaryOp,
    pub(super) left_ty: &'e Type,
    pub(super) right_ty: &'e Type,
}

/// The types of a scalar compound assignment.
pub(super) struct ScalarAssign<'e> {
    pub(super) op: &'e BinaryOp,
    pub(super) ty: &'e Type,
    pub(super) value_ty: &'e Type,
}

/// The frame slots and width of one switch statement.
pub(super) struct SwitchSlots {
    pub(super) target_slot: u32,
    pub(super) matched_slot: u32,
    pub(super) any_match_slot: u32,
    pub(super) switch_is_wide: bool,
}

/// The constructor of a temporary value.
pub(super) struct CtorCall<'e> {
    pub(super) type_name: &'e str,
    pub(super) ctor_idx: u32,
}

/// The element size and count slot of a heap array.
pub(super) struct ArraySlots {
    pub(super) elem_size: i32,
    pub(super) count_slot: u32,
    pub(super) base_slot: u32,
}

/// Parameters for iterating array element constructors/vtables.
pub(super) struct ArrayLoop {
    pub(super) ctor_idx: Option<u32>,
    pub(super) v_off: Option<u32>,
    pub(super) slots: ArraySlots,
    pub(super) start_idx: i32,
    pub(super) user_slot: u32,
}

/// One namespace-scope object that is initialized before `main` runs.
pub(super) struct GlobalInit<'e> {
    pub(super) ty: &'e Type,
    pub(super) init: &'e Option<Expr>,
    pub(super) offset: u32,
}
