use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_logical_and(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::Binary {
            op: BinaryOp::LogicalAnd,
            left,
            right,
        } = expr
        else {
            return Err(crate::internal_error!(0, 0, "unexpected expression kind"));
        };

        self.compile_cond(left, ctx, code)?;
        code.push(OP_IF);
        code.push(TYPE_I32);
        self.compile_cond(right, ctx, code)?;
        code.push(OP_I32_EQZ);
        code.push(OP_I32_EQZ);
        code.push(OP_ELSE);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_END);

        Ok(())
    }

    pub(super) fn compile_logical_or(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::Binary {
            op: BinaryOp::LogicalOr,
            left,
            right,
        } = expr
        else {
            return Err(crate::internal_error!(0, 0, "unexpected expression kind"));
        };

        self.compile_cond(left, ctx, code)?;
        code.push(OP_IF);
        code.push(TYPE_I32);
        code.push(OP_I32_CONST);
        write_i32(code, 1);
        code.push(OP_ELSE);
        self.compile_cond(right, ctx, code)?;
        code.push(OP_I32_EQZ);
        code.push(OP_I32_EQZ);
        code.push(OP_END);

        Ok(())
    }

    pub(super) fn compile_binary(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::Binary { op, left, right } = expr else {
            return Err(crate::internal_error!(0, 0, "unexpected expression kind"));
        };

        let lty = self.get_expr_type(left, ctx);
        let rty = self.get_expr_type(right, ctx);
        let pointer_diff_elem = if *op == BinaryOp::Sub
            && lty.decay_array().is_pointer()
            && rty.decay_array().is_pointer()
        {
            let size = lty
                .decay_array()
                .deref_type()
                .map(|inner| inner.size_of(&self.sema.struct_sizes))
                .unwrap_or(1)
                .max(1);
            Some(
                i32::try_from(size)
                    .map_err(|_| crate::internal_error!(0, 0, "pointer element is too large"))?,
            )
        } else {
            None
        };
        if (matches!(lty.strip_const(), Type::Struct(_) | Type::Enum(_))
            || matches!(rty.strip_const(), Type::Struct(_) | Type::Enum(_)))
            && let Some(op_name) = op.op_name()
        {
            let clean_lty = lty.strip_const();
            if let Type::Struct(sname) = clean_lty
                && let Some((m_idx, sig)) =
                    self.find_method(sname, op_name, std::slice::from_ref(&rty))
            {
                self.compile_lvalue_addr(left, ctx, code)?;
                let param_ty = sig.param_types.get(1);
                let is_r = param_ty.map(|p| p.is_reference()).unwrap_or(false);
                if is_r {
                    self.compile_lvalue_addr(right, ctx, code)?;
                } else {
                    self.compile_expr(right, ctx, code)?;
                    if let Some(pty) = param_ty {
                        self.emit_conversion(&rty, pty, code);
                    }
                }
                code.push(OP_CALL);
                write_u32(code, m_idx);
                self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
                return Ok(());
            }

            if let Some((f_idx, sig)) = self
                .find_function(op_name, &[lty.clone(), rty.clone()])
                .or_else(|| {
                    self.find_function(&format!("std::{}", op_name), &[lty.clone(), rty.clone()])
                })
                .or_else(|| {
                    self.find_function(&format!("std::{}", op_name), &[lty.clone(), rty.clone()])
                })
            {
                let param_ty0 = sig.param_types.first();
                let is_r0 = param_ty0.map(|p| p.is_reference()).unwrap_or(false);
                if is_r0 {
                    self.compile_lvalue_addr(left, ctx, code)?;
                } else {
                    self.compile_expr(left, ctx, code)?;
                    if let Some(pty) = param_ty0 {
                        self.emit_conversion(&lty, pty, code);
                    }
                }
                let param_ty1 = sig.param_types.get(1);
                let is_r1 = param_ty1.map(|p| p.is_reference()).unwrap_or(false);
                if is_r1 {
                    self.compile_lvalue_addr(right, ctx, code)?;
                } else {
                    self.compile_expr(right, ctx, code)?;
                    if let Some(pty) = param_ty1 {
                        self.emit_conversion(&rty, pty, code);
                    }
                }
                code.push(OP_CALL);
                write_u32(code, f_idx);
                self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
                return Ok(());
            }
        }
        let lraw = lty.strip_const();
        let rraw = rty.strip_const();
        let is_shift = matches!(op, BinaryOp::Shl | BinaryOp::Shr);
        let pointer_arithmetic = matches!(op, BinaryOp::Add | BinaryOp::Sub)
            && (((lty.is_pointer() || lty.is_array()) && rty.is_integer())
                || (*op == BinaryOp::Add
                    && (rty.is_pointer() || rty.is_array())
                    && lty.is_integer()));
        let common_ty = if pointer_arithmetic
            || ((lty.is_pointer() || lty.is_array()) && (rty.is_pointer() || rty.is_array()))
        {
            Type::UnsignedInt
        } else if is_shift {
            promote_integer(lraw)
        } else {
            arithmetic_common_type(lraw, rraw)
        };
        self.compile_expr(left, ctx, code)?;
        Self::emit_operand_conversion(lraw, &common_ty, code);
        self.compile_expr(right, ctx, code)?;
        Self::emit_operand_conversion(rraw, &common_ty, code);
        if (lty.is_pointer() || lty.is_array())
            && rty.is_integer()
            && (op == &BinaryOp::Add || op == &BinaryOp::Sub)
        {
            let esize = lty
                .deref_type()
                .map(|d| d.size_of(&self.sema.struct_sizes))
                .unwrap_or(4)
                .max(1) as i32;
            code.push(OP_I32_CONST);
            write_i32(code, esize);
            code.push(OP_I32_MUL);
        } else if (rty.is_pointer() || rty.is_array()) && lty.is_integer() && op == &BinaryOp::Add {
            let esize = rty
                .deref_type()
                .map(|d| d.size_of(&self.sema.struct_sizes))
                .unwrap_or(4)
                .max(1) as i32;
            // The stack holds [integer, pointer]. Keep the pointer in a scratch
            // local while the integer is scaled, then put them back in add order.
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1);
            code.push(OP_I32_CONST);
            write_i32(code, esize);
            code.push(OP_I32_MUL);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
        }

        Self::emit_binary_op(op, &common_ty, code);
        if let Some(elem_size) = pointer_diff_elem
            && elem_size > 1
        {
            code.push(OP_I32_CONST);
            write_i32(code, elem_size);
            code.push(OP_I32_DIV_S);
        }

        Ok(())
    }

    fn emit_operand_conversion(raw: &Type, common_ty: &Type, code: &mut Vec<u8>) {
        let is_double = *common_ty == Type::Double;
        let is_fp = matches!(common_ty, Type::Float | Type::Double);
        let is_i64 = matches!(common_ty, Type::LongLong | Type::UnsignedLongLong);
        if is_double {
            if matches!(raw, Type::Float) {
                code.push(OP_F64_PROMOTE_F32);
            } else if matches!(raw, Type::LongLong | Type::UnsignedLongLong) {
                code.push(if raw.is_signed() {
                    OP_F64_CONVERT_I64_S
                } else {
                    OP_F64_CONVERT_I64_U
                });
            } else if raw.is_integer() {
                code.push(if raw.is_signed() {
                    OP_F64_CONVERT_I32_S
                } else {
                    OP_F64_CONVERT_I32_U
                });
            }
        } else if is_fp {
            if matches!(raw, Type::LongLong | Type::UnsignedLongLong) {
                code.push(if raw.is_signed() {
                    OP_F32_CONVERT_I64_S
                } else {
                    OP_F32_CONVERT_I64_U
                });
            } else if raw.is_integer() {
                code.push(if raw.is_signed() {
                    OP_F32_CONVERT_I32_S
                } else {
                    OP_F32_CONVERT_I32_U
                });
            }
        } else if is_i64 {
            if !matches!(raw, Type::LongLong | Type::UnsignedLongLong) {
                code.push(if raw.is_signed() {
                    OP_I64_EXTEND_I32_S
                } else {
                    OP_I64_EXTEND_I32_U
                });
            }
        } else if matches!(raw, Type::LongLong | Type::UnsignedLongLong) {
            code.push(OP_I32_WRAP_I64);
        }
    }

    fn emit_binary_op(op: &BinaryOp, common_ty: &Type, code: &mut Vec<u8>) {
        let is_double = *common_ty == Type::Double;
        let is_fp = matches!(common_ty, Type::Float | Type::Double);
        let is_i64 = matches!(common_ty, Type::LongLong | Type::UnsignedLongLong);
        let is_unsigned = !common_ty.is_signed();
        if is_double {
            match op {
                BinaryOp::Add => code.push(OP_F64_ADD),
                BinaryOp::Sub => code.push(OP_F64_SUB),
                BinaryOp::Mul => code.push(OP_F64_MUL),
                BinaryOp::Div => code.push(OP_F64_DIV),
                BinaryOp::Eq => code.push(OP_F64_EQ),
                BinaryOp::Ne => code.push(OP_F64_NE),
                BinaryOp::Lt => code.push(OP_F64_LT),
                BinaryOp::Le => code.push(OP_F64_LE),
                BinaryOp::Gt => code.push(OP_F64_GT),
                BinaryOp::Ge => code.push(OP_F64_GE),
                _ => {}
            }
        } else if is_fp {
            match op {
                BinaryOp::Add => code.push(OP_F32_ADD),
                BinaryOp::Sub => code.push(OP_F32_SUB),
                BinaryOp::Mul => code.push(OP_F32_MUL),
                BinaryOp::Div => code.push(OP_F32_DIV),
                BinaryOp::Eq => code.push(OP_F32_EQ),
                BinaryOp::Ne => code.push(OP_F32_NE),
                BinaryOp::Lt => code.push(OP_F32_LT),
                BinaryOp::Le => code.push(OP_F32_LE),
                BinaryOp::Gt => code.push(OP_F32_GT),
                BinaryOp::Ge => code.push(OP_F32_GE),
                _ => {}
            }
        } else if is_i64 {
            match op {
                BinaryOp::Add => code.push(OP_I64_ADD),
                BinaryOp::Sub => code.push(OP_I64_SUB),
                BinaryOp::Mul => code.push(OP_I64_MUL),
                BinaryOp::Div => code.push(if is_unsigned {
                    OP_I64_DIV_U
                } else {
                    OP_I64_DIV_S
                }),
                BinaryOp::Rem => code.push(if is_unsigned {
                    OP_I64_REM_U
                } else {
                    OP_I64_REM_S
                }),
                BinaryOp::BitAnd => code.push(OP_I64_AND),
                BinaryOp::BitOr => code.push(OP_I64_OR),
                BinaryOp::BitXor => code.push(OP_I64_XOR),
                BinaryOp::Shl => code.push(OP_I64_SHL),
                BinaryOp::Shr => code.push(if is_unsigned {
                    OP_I64_SHR_U
                } else {
                    OP_I64_SHR_S
                }),
                BinaryOp::Eq => code.push(OP_I64_EQ),
                BinaryOp::Ne => code.push(OP_I64_NE),
                BinaryOp::Lt => code.push(if is_unsigned {
                    OP_I64_LT_U
                } else {
                    OP_I64_LT_S
                }),
                BinaryOp::Le => code.push(if is_unsigned {
                    OP_I64_LE_U
                } else {
                    OP_I64_LE_S
                }),
                BinaryOp::Gt => code.push(if is_unsigned {
                    OP_I64_GT_U
                } else {
                    OP_I64_GT_S
                }),
                BinaryOp::Ge => code.push(if is_unsigned {
                    OP_I64_GE_U
                } else {
                    OP_I64_GE_S
                }),
                _ => {}
            }
        } else {
            match op {
                BinaryOp::Add => code.push(OP_I32_ADD),
                BinaryOp::Sub => code.push(OP_I32_SUB),
                BinaryOp::Mul => code.push(OP_I32_MUL),
                BinaryOp::Div => code.push(if is_unsigned {
                    OP_I32_DIV_U
                } else {
                    OP_I32_DIV_S
                }),
                BinaryOp::Rem => code.push(if is_unsigned {
                    OP_I32_REM_U
                } else {
                    OP_I32_REM_S
                }),
                BinaryOp::BitAnd => code.push(OP_I32_AND),
                BinaryOp::BitOr => code.push(OP_I32_OR),
                BinaryOp::BitXor => code.push(OP_I32_XOR),
                BinaryOp::Shl => code.push(OP_I32_SHL),
                BinaryOp::Shr => code.push(if is_unsigned {
                    OP_I32_SHR_U
                } else {
                    OP_I32_SHR_S
                }),
                BinaryOp::Eq => code.push(OP_I32_EQ),
                BinaryOp::Ne => code.push(OP_I32_NE),
                BinaryOp::Lt => code.push(if is_unsigned {
                    OP_I32_LT_U
                } else {
                    OP_I32_LT_S
                }),
                BinaryOp::Le => code.push(if is_unsigned {
                    OP_I32_LE_U
                } else {
                    OP_I32_LE_S
                }),
                BinaryOp::Gt => code.push(if is_unsigned {
                    OP_I32_GT_U
                } else {
                    OP_I32_GT_S
                }),
                BinaryOp::Ge => code.push(if is_unsigned {
                    OP_I32_GE_U
                } else {
                    OP_I32_GE_S
                }),
                BinaryOp::LogicalAnd => {
                    code.push(OP_I32_AND);
                }
                BinaryOp::LogicalOr => {
                    code.push(OP_I32_OR);
                }
            }
        }
    }
}
