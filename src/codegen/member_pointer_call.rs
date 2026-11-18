//! Pointer-to-member call lowering.

use super::emit_context::{PmfInvokeSite, PmfReceiver};
use super::member_pointer_types::PmfSlots;
use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn normalize_member_pointer_operand(
        ty: &Type,
        is_func: bool,
        is_null_constant: bool,
        code: &mut Vec<u8>,
    ) {
        if is_null_constant || matches!(ty.strip_const(), Type::Nullptr) {
            code.push(OP_DROP);
            if is_func {
                code.push(OP_I64_CONST);
                write_i64(code, 0);
            } else {
                code.push(OP_I32_CONST);
                write_i32(code, -1);
            }
        }
    }

    pub(super) fn compile_member_pointer_call(
        &mut self,
        call: MemberPointerCall<'_>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let MemberPointerCall {
            left,
            right,
            is_arrow,
            args,
        } = call;

        let right_ty = self.get_expr_type(right, ctx);
        let (param_types, ret_ty) = match right_ty.strip_const() {
            Type::MemberPointer { member_type, .. } => match member_type.strip_const() {
                Type::Function {
                    param_types,
                    return_type,
                    ..
                } => (param_types.clone(), (**return_type).clone()),
                _ => (Vec::new(), Type::Int),
            },
            _ => (Vec::new(), Type::Int),
        };

        let this_slot = ctx.alloc_frame(4, 4)?;
        let fn_slot = ctx.alloc_frame(4, 4)?;
        let ret_adjust_slot = ctx.alloc_frame(4, 4)?;

        self.emit_pmf_receiver(
            PmfReceiver {
                left,
                right,
                is_arrow,
                this_slot,
            },
            ctx,
            code,
        )?;
        self.emit_pmf_target(
            right,
            PmfSlots {
                this: this_slot,
                fn_slot,
                ret_adjust: ret_adjust_slot,
            },
            ctx,
            code,
        )?;
        self.emit_pmf_invocation(
            PmfInvokeSite {
                this_slot,
                fn_slot,
                ret_adjust_slot,
                args,
                param_types: &param_types,
                ret_ty: &ret_ty,
            },
            ctx,
            code,
        )?;

        Ok(())
    }

    pub(super) fn emit_pmf_receiver(
        &mut self,
        site: PmfReceiver<'_>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let PmfReceiver {
            left,
            right,
            is_arrow,
            this_slot,
        } = site;
        let delta = self.member_pointer_receiver_delta(left, right, is_arrow, ctx);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        if is_arrow {
            self.compile_expr(left, ctx, code)?;
        } else {
            self.compile_lvalue_addr(left, ctx, code)?;
        }
        self.emit_receiver_base_adjustment(delta, is_arrow, code);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, this_slot);
        Ok(())
    }
}
