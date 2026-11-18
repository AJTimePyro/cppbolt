//! Receiver adjustment for member-pointer application.

use super::*;

impl<'a> Codegen<'a> {
    /// Returns the offset from the receiver's class to the class named by a
    /// member pointer. Applying a `Base::*` pointer to a `Derived` object must
    /// first move the receiver to the base subobject; conversions of the
    /// pointer itself (`Base::*` to `Derived::*`) already fold that offset in,
    /// so the delta is zero in that case and cannot be applied twice.
    pub(super) fn member_pointer_receiver_delta(
        &self,
        left: &Expr,
        right: &Expr,
        is_arrow: bool,
        ctx: &FunctionContext,
    ) -> usize {
        let left_ty = self.get_expr_type(left, ctx);
        let clean = left_ty.strip_reference().strip_const();
        let object_ty = if is_arrow {
            match clean {
                Type::Pointer(inner) => inner.strip_reference().strip_const(),
                _ => clean,
            }
        } else {
            clean
        };
        let Type::Struct(obj_class) = object_ty else {
            return 0;
        };
        let right_ty = self.get_expr_type(right, ctx);
        let Type::MemberPointer { class_name, .. } = right_ty.strip_reference().strip_const()
        else {
            return 0;
        };
        if obj_class == class_name {
            return 0;
        }
        self.sema
            .base_subobject_offset(obj_class, class_name)
            .unwrap_or(0)
    }
    /// Adjusts a receiver address that is already on the stack by the delta
    /// between the object's class and the member pointer's class.
    pub(super) fn emit_receiver_base_adjustment(
        &self,
        delta: usize,
        is_arrow: bool,
        code: &mut Vec<u8>,
    ) {
        if delta == 0 {
            return;
        }
        if is_arrow {
            self.emit_guarded_pointer_adjustment(delta, true, code);
        } else {
            code.push(OP_I32_CONST);
            write_i32(code, delta as i32);
            code.push(OP_I32_ADD);
        }
    }
}
