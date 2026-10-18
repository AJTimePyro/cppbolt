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
                self.register_returned_struct_temp(&sig.return_type, ctx, code);
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
                self.register_returned_struct_temp(&sig.return_type, ctx, code);
                return Ok(());
            }
        }
        let is_fp = lty.is_floating() || rty.is_floating();
        let is_double = lty == Type::Double || rty == Type::Double;
        let is_i64 = lty == Type::LongLong
            || lty == Type::UnsignedLongLong
            || rty == Type::LongLong
            || rty == Type::UnsignedLongLong;
        let is_unsigned = !lty.is_signed() || !rty.is_signed();

        self.compile_expr(left, ctx, code)?;
        if is_double && lty == Type::Float {
            code.push(OP_F64_PROMOTE_F32);
        } else if is_fp && lty.is_integer() {
            if is_double {
                code.push(OP_F64_CONVERT_I32_S);
            } else {
                code.push(OP_F32_CONVERT_I32_S);
            }
        } else if is_i64 && !matches!(lty.strip_const(), Type::LongLong | Type::UnsignedLongLong) {
            code.push(OP_I64_EXTEND_I32_S);
        }

        self.compile_expr(right, ctx, code)?;
        if is_double && rty == Type::Float {
            code.push(OP_F64_PROMOTE_F32);
        } else if is_fp && rty.is_integer() {
            if is_double {
                code.push(OP_F64_CONVERT_I32_S);
            } else {
                code.push(OP_F32_CONVERT_I32_S);
            }
        } else if is_i64 && !matches!(rty.strip_const(), Type::LongLong | Type::UnsignedLongLong) {
            code.push(OP_I64_EXTEND_I32_S);
        } else if (lty.is_pointer() || lty.is_array())
            && rty.is_integer()
            && (op == &BinaryOp::Add || op == &BinaryOp::Sub)
        {
            let esize = lty
                .deref_type()
                .map(|d| d.size_of(&self.sema.struct_sizes))
                .unwrap_or(4) as i32;
            code.push(OP_I32_CONST);
            write_i32(code, esize);
            code.push(OP_I32_MUL);
        }

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

        Ok(())
    }
}
