//! Lowering of overloaded binary operators.

use super::*;

/// One overloaded operator call.
pub(super) struct OperatorCall<'e> {
    /// The operator name, e.g. `operator+`.
    pub(super) op_name: &'e str,
    /// The type of the left operand.
    pub(super) left_ty: &'e Type,
    /// The type of the right operand.
    pub(super) right_ty: &'e Type,
}

impl<'a> Codegen<'a> {
    pub(super) fn emit_overloaded_binary_node(
        &mut self,
        types: BinaryTypes<'_>,
        left: Option<&Expr>,
        right: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<bool, CompileError> {
        let BinaryTypes {
            op,
            left_ty: lty,
            right_ty: rty,
        } = types;
        if (matches!(lty.strip_const(), Type::Struct(_) | Type::Enum(_))
            || matches!(rty.strip_const(), Type::Struct(_) | Type::Enum(_)))
            && let Some(op_name) = op.op_name()
        {
            let call = OperatorCall {
                op_name,
                left_ty: lty,
                right_ty: rty,
            };
            if self.emit_member_operator(&call, left, right, ctx, code)? {
                return Ok(true);
            }
            if self.emit_free_operator(&call, left, right, ctx, code)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Emits a call of an overloaded member operator.
    fn emit_member_operator(
        &mut self,
        call: &OperatorCall<'_>,
        left: Option<&Expr>,
        right: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<bool, CompileError> {
        let OperatorCall {
            op_name,
            left_ty: lty,
            right_ty: rty,
        } = *call;
        let clean_lty = lty.strip_const();
        if let Type::Struct(sname) = clean_lty
            && let Some((m_idx, sig)) = self.find_method(sname, op_name, std::slice::from_ref(rty))
        {
            if let Some(left) = left {
                self.compile_lvalue_addr(left, ctx, code)?;
            }
            let param_ty = sig.param_types.get(1);
            let is_r = param_ty.map(|p| p.is_reference()).unwrap_or(false);
            if is_r {
                self.compile_lvalue_addr(right, ctx, code)?;
            } else {
                self.compile_expr(right, ctx, code)?;
                if let Some(pty) = param_ty {
                    self.emit_conversion(rty, pty, code);
                }
            }
            code.push(OP_CALL);
            write_u32(code, m_idx);
            self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Emits a call of an overloaded free operator.
    fn emit_free_operator(
        &mut self,
        call: &OperatorCall<'_>,
        left: Option<&Expr>,
        right: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<bool, CompileError> {
        let OperatorCall {
            op_name,
            left_ty: lty,
            right_ty: rty,
        } = *call;
        let Some((f_idx, sig)) = self
            .find_function(op_name, &[lty.clone(), rty.clone()])
            .or_else(|| {
                self.find_function(&format!("std::{}", op_name), &[lty.clone(), rty.clone()])
            })
            .or_else(|| {
                self.find_function(&format!("std::{}", op_name), &[lty.clone(), rty.clone()])
            })
        else {
            return Ok(false);
        };
        let param_ty0 = sig.param_types.first();
        let is_r0 = param_ty0.map(|p| p.is_reference()).unwrap_or(false);
        if is_r0 {
            if let Some(left) = left {
                self.compile_lvalue_addr(left, ctx, code)?;
            }
        } else {
            if let Some(left) = left {
                self.compile_expr(left, ctx, code)?;
            }
            if let Some(pty) = param_ty0 {
                self.emit_conversion(lty, pty, code);
            }
        }
        let param_ty1 = sig.param_types.get(1);
        let is_r1 = param_ty1.map(|p| p.is_reference()).unwrap_or(false);
        if is_r1 {
            self.compile_lvalue_addr(right, ctx, code)?;
        } else {
            self.compile_expr(right, ctx, code)?;
            if let Some(pty) = param_ty1 {
                self.emit_conversion(rty, pty, code);
            }
        }
        code.push(OP_CALL);
        write_u32(code, f_idx);
        self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
        Ok(true)
    }
}
