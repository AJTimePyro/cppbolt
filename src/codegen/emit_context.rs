//! Small context structs that group the values the lowering helpers need.
//!
//! The helpers used to take long parameter lists that repeated the same
//! frame slots, flags and declaration pieces at every call site.  Grouping
//! them keeps the signatures short, keeps the related values together, and
//! makes the call sites readable.

use super::*;

/// The receiver and the member being called.
pub(super) struct MemberAccess<'e> {
    /// The object the member is accessed on.
    pub(super) object: &'e Expr,
    /// The member name.
    pub(super) member: &'e str,
    /// Whether the access uses `->` rather than `.`.
    pub(super) is_arrow: bool,
}

/// A call to a named function.
pub(super) struct NamedCall<'e> {
    /// The callee expression, used for indirect calls.
    pub(super) callee: &'e Expr,
    /// The resolved function name.
    pub(super) name: &'e str,
    /// The enclosing class, for a call to a member of the current class.
    pub(super) class_name: Option<&'e str>,
}

/// A method call and the frame slots it uses.
///
/// The same struct serves the virtual and the direct lowering: the virtual
/// path uses `slot_idx` and `fn_slot`, the direct path uses the resolved
/// method index instead.
pub(super) struct CallSite<'e> {
    /// The class that declares the method.
    pub(super) class_name: &'e str,
    /// The method name.
    pub(super) member: &'e str,
    /// The call arguments.
    pub(super) args: &'e [Expr],
    /// The vtable slot of a virtual call.
    pub(super) slot_idx: usize,
    /// The frame slot that holds the receiver address.
    pub(super) this_slot: u32,
    /// The frame slot that holds the resolved function pointer.
    pub(super) fn_slot: u32,
    /// Whether the receiver is const-qualified.
    pub(super) receiver_const: bool,
}

/// A local variable declaration being lowered.
pub(super) struct LocalDecl<'e> {
    /// The declared struct name, empty for non-struct locals.
    pub(super) struct_name: &'e str,
    /// The declared type.
    pub(super) ty: &'e Type,
    /// The initializer, if any.
    pub(super) init: &'e Option<Expr>,
    /// The frame offset of the local.
    pub(super) offset: u32,
    /// The variable name.
    pub(super) name: &'e str,
    /// The statement the declaration belongs to.
    pub(super) stmt: &'e Stmt,
}

/// A local array declaration being lowered.
pub(super) struct ArrayLocal<'e> {
    /// The element type.
    pub(super) elem_ty: &'e Type,
    /// The number of elements.
    pub(super) count: usize,
    /// The frame offset of the array.
    pub(super) offset: u32,
    /// The variable name.
    pub(super) name: &'e str,
    /// The initializer, if any.
    pub(super) init: &'e Option<Expr>,
    /// The statement the declaration belongs to.
    pub(super) stmt: &'e Stmt,
}

/// The record type and frame offset of a struct local.
pub(super) struct StructInit<'e> {
    /// The struct name.
    pub(super) struct_name: &'e str,
    /// The struct type.
    pub(super) ty: &'e Type,
    /// The frame offset of the local.
    pub(super) offset: u32,
}

/// The operand types of a binary operator.
pub(super) struct BinaryTypes<'e> {
    /// The operator.
    pub(super) op: &'e BinaryOp,
    /// The type of the left operand.
    pub(super) left_ty: &'e Type,
    /// The type of the right operand.
    pub(super) right_ty: &'e Type,
}

/// The types of a scalar compound assignment.
pub(super) struct ScalarAssign<'e> {
    /// The operator.
    pub(super) op: &'e BinaryOp,
    /// The type of the target.
    pub(super) ty: &'e Type,
    /// The type of the assigned value.
    pub(super) value_ty: &'e Type,
}

/// The frame slots and width of one switch statement.
pub(super) struct SwitchSlots {
    /// The frame slot that holds the switch value.
    pub(super) target_slot: u32,
    /// The frame slot that records whether a case matched.
    pub(super) matched_slot: u32,
    /// Whether the switch value is 64 bits wide.
    pub(super) switch_is_wide: bool,
}

/// The constructor of a temporary value.
pub(super) struct CtorCall<'e> {
    /// The name of the constructed type.
    pub(super) type_name: &'e str,
    /// The index of the constructor in the function table.
    pub(super) ctor_idx: u32,
}

/// The element size and count slot of a heap array.
pub(super) struct ArraySlots {
    /// The size of one element in bytes.
    pub(super) elem_size: i32,
    /// The frame slot that holds the element count.
    pub(super) count_slot: u32,
}
