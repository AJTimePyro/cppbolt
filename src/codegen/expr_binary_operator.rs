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
    /// Whether a chain step can use the previous step's stack value as its
    /// left operand.
    ///
    /// An overloaded step whose stack value cannot bind to its left parameter
    /// recompiles the left subtree, because the user-defined conversion lives
    /// in the expression.
    fn binary_left_uses_stack(&self, op: &BinaryOp, lty: &Type, rty: &Type) -> bool {
        let is_class = |ty: &Type| matches!(ty.strip_const(), Type::Struct(_) | Type::Enum(_));
        if !(is_class(lty) || is_class(rty)) {
            return true;
        }
        let Some(op_name) = op.op_name() else {
            return true;
        };
        let Some(param_ty) = self.operator_left_param(op_name, lty, rty) else {
            return false;
        };
        // A struct result sits on the stack as the address of its temporary,
        // which a reference to the same class binds as-is.
        let (Some(target), Type::Struct(result)) = (param_ty.deref_type(), lty.strip_const())
        else {
            return false;
        };
        match target.strip_const() {
            Type::Struct(param) => QualifiedName::new(param).matches(QualifiedName::new(result)),
            _ => false,
        }
    }

    /// Returns the topmost step that must recompile the prefix below it, or
    /// `None` when every step can take its left operand from the stack.
    pub(super) fn recompile_step(
        &self,
        spine: &[(&BinaryOp, &Expr, &Expr)],
        base: &Expr,
        ctx: &FunctionContext,
    ) -> Option<usize> {
        let mut lty = self.get_expr_type(base, ctx);
        let mut step = None;
        for index in (0..spine.len()).rev() {
            let (op, right, _) = spine[index];
            let rty = self.get_expr_type(right, ctx);
            if !self.binary_left_uses_stack(op, &lty, &rty) {
                step = Some(index);
            }
            lty = self
                .binary_result_type(op, &lty, &rty)
                .strip_reference()
                .clone();
        }
        step
    }

    /// Looks up the operator a binary expression calls: the left class's
    /// member operator first, then a free operator.
    pub(super) fn binary_operator_sig(
        &self,
        op_name: &str,
        lty: &Type,
        rty: &Type,
    ) -> Option<(u32, FunctionSig)> {
        if let Type::Struct(sname) = lty.strip_const()
            && let Some(found) = self.find_method(sname, op_name, std::slice::from_ref(rty))
        {
            return Some(found);
        }
        let lookup = |name: &str| self.find_function(name, &[lty.clone(), rty.clone()]);
        lookup(op_name).or_else(|| lookup(&format!("std::{}", op_name)))
    }

    /// The declared type of the left parameter of the operator a step calls.
    fn operator_left_param(&self, op_name: &str, lty: &Type, rty: &Type) -> Option<Type> {
        // The member call's first parameter is `this`; the free call's is the
        // left operand.
        self.binary_operator_sig(op_name, lty, rty)
            .and_then(|(_, sig)| sig.param_types.first().cloned())
    }

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
        else {
            return Ok(false);
        };
        let param_ty0 = sig.param_types.first();
        let is_r0 = param_ty0.map(|p| p.is_reference()).unwrap_or(false);
        if let Some(left) = left {
            if matches!(left, Expr::Binary { .. }) {
                // A recompiled chain prefix goes through the normal call
                // argument path so user-defined conversions are applied.
                self.compile_call_arg(left, param_ty0, ctx, code)?;
            } else if is_r0 {
                self.compile_lvalue_addr(left, ctx, code)?;
            } else {
                self.compile_expr(left, ctx, code)?;
                if let Some(pty) = param_ty0 {
                    self.emit_conversion(lty, pty, code);
                }
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
