use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_assign(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::Assign { target, value } = expr else {
            return Err(crate::internal_error!(0, 0, "unexpected expression kind"));
        };

        let ty = self.get_expr_type(target, ctx);
        if let Type::Struct(sname) = ty.strip_const() {
            let val_ty = self.get_expr_type(value, ctx);
            if let Some((assign_idx, sig)) =
                self.find_method(sname, "operator=", std::slice::from_ref(&val_ty))
            {
                self.compile_lvalue_addr(target, ctx, code)?; // this
                let is_r = sig
                    .param_types
                    .get(1)
                    .map(|p| p.is_reference())
                    .unwrap_or(false);
                if is_r {
                    self.compile_lvalue_addr(value, ctx, code)?;
                } else {
                    self.compile_expr(value, ctx, code)?;
                }
                code.push(OP_CALL);
                write_u32(code, assign_idx);
                return Ok(());
            } else {
                self.compile_lvalue_addr(target, ctx, code)?;
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch2); // dest addr

                self.compile_lvalue_addr(value, ctx, code)?;
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1); // src addr

                let size = self
                    .sema
                    .struct_layouts
                    .get(sname)
                    .map(|l| l.size)
                    .unwrap_or(4);
                for i in (0..size).step_by(4) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch2);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_I32_LOAD);
                    write_u32(code, 2);
                    write_u32(code, i as u32);
                    code.push(OP_I32_STORE);
                    write_u32(code, 2);
                    write_u32(code, i as u32);
                }
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch2);
                return Ok(());
            }
        }

        let target_ty = self.get_expr_type(target, ctx);
        let val_ty = self.get_expr_type(value, ctx);
        self.compile_lvalue_addr(target, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1); // dest addr

        self.compile_expr(value, ctx, code)?;

        if matches!(
            val_ty.strip_const(),
            Type::LongLong | Type::UnsignedLongLong
        ) {
            if target_ty.is_floating() {
                if target_ty == Type::Double {
                    code.push(OP_F64_CONVERT_I64_S);
                } else {
                    code.push(OP_F32_CONVERT_I64_S);
                }
            } else if !matches!(
                target_ty.strip_const(),
                Type::LongLong | Type::UnsignedLongLong
            ) {
                code.push(OP_I32_WRAP_I64);
            }
        } else if matches!(
            target_ty.strip_const(),
            Type::LongLong | Type::UnsignedLongLong
        ) {
            if val_ty == Type::Double {
                code.push(OP_I64_TRUNC_F64_S);
            } else if val_ty == Type::Float {
                code.push(OP_I64_TRUNC_F32_S);
            } else if val_ty.is_integer() {
                if val_ty.is_signed() {
                    code.push(OP_I64_EXTEND_I32_S);
                } else {
                    code.push(OP_I64_EXTEND_I32_U);
                }
            }
        } else if target_ty == Type::Double && val_ty.is_integer() {
            code.push(OP_F64_CONVERT_I32_S);
        } else if target_ty == Type::Float && val_ty.is_integer() {
            code.push(OP_F32_CONVERT_I32_S);
        } else if target_ty == Type::Float && val_ty == Type::Double {
            code.push(OP_F32_DEMOTE_F64);
        } else if target_ty == Type::Double && val_ty == Type::Float {
            code.push(OP_F64_PROMOTE_F32);
        }

        let target_raw = target_ty.strip_const();
        if matches!(target_raw, Type::LongLong | Type::UnsignedLongLong) {
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch_i64);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_i64);
            self.emit_store(&target_ty, 0, code);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_i64);
        } else if matches!(target_raw, Type::Double) {
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch_f64);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_f64);
            self.emit_store(&target_ty, 0, code);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_f64);
        } else if matches!(target_raw, Type::Float) {
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch_f32);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_f32);
            self.emit_store(&target_ty, 0, code);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_f32);
        } else {
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch2);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch2);
            self.emit_store(&target_ty, 0, code);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch2);
        }

        Ok(())
    }

    pub(super) fn compile_compound_assign(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::CompoundAssign { op, target, value } = expr else {
            return Err(crate::internal_error!(0, 0, "unexpected expression kind"));
        };

        let ty = self.get_expr_type(target, ctx);
        let val_ty = self.get_expr_type(value, ctx);
        let op_name = format!(
            "operator{}=",
            match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Rem => "%",
                BinaryOp::BitAnd => "&",
                BinaryOp::BitOr => "|",
                BinaryOp::BitXor => "^",
                BinaryOp::Shl => "<<",
                BinaryOp::Shr => ">>",
                _ => "",
            }
        );
        if let Type::Struct(sname) = ty.strip_const()
            && let Some((m_idx, sig)) =
                self.find_method(sname, &op_name, std::slice::from_ref(&val_ty))
        {
            self.compile_lvalue_addr(target, ctx, code)?;
            let is_r = sig
                .param_types
                .get(1)
                .map(|p| p.is_reference())
                .unwrap_or(false);
            if is_r {
                self.compile_lvalue_addr(value, ctx, code)?;
            } else {
                self.compile_expr(value, ctx, code)?;
            }
            code.push(OP_CALL);
            write_u32(code, m_idx);
            return Ok(());
        }
        self.compile_lvalue_addr(target, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        self.emit_load(&ty, 0, code);
        self.compile_expr(value, ctx, code)?;

        if ty.is_pointer() && (op == &BinaryOp::Add || op == &BinaryOp::Sub) {
            let esize = ty
                .deref_type()
                .map(|d| d.size_of(&self.sema.struct_sizes))
                .unwrap_or(4) as i32;
            code.push(OP_I32_CONST);
            write_i32(code, esize);
            code.push(OP_I32_MUL);
        }

        let val_ty = self.get_expr_type(value, ctx);
        if (ty == Type::LongLong || ty == Type::UnsignedLongLong)
            && val_ty != Type::LongLong
            && val_ty != Type::UnsignedLongLong
        {
            code.push(OP_I64_EXTEND_I32_S);
        } else if ty == Type::Double && val_ty == Type::Float {
            code.push(OP_F64_PROMOTE_F32);
        } else if ty == Type::Float && val_ty == Type::Double {
            code.push(OP_F32_DEMOTE_F64);
        }

        if ty == Type::LongLong || ty == Type::UnsignedLongLong {
            match op {
                BinaryOp::Add => code.push(OP_I64_ADD),
                BinaryOp::Sub => code.push(OP_I64_SUB),
                BinaryOp::Mul => code.push(OP_I64_MUL),
                BinaryOp::Div => code.push(OP_I64_DIV_S),
                BinaryOp::Rem => code.push(OP_I64_REM_S),
                BinaryOp::BitAnd => code.push(OP_I64_AND),
                BinaryOp::BitOr => code.push(OP_I64_OR),
                BinaryOp::BitXor => code.push(OP_I64_XOR),
                BinaryOp::Shl => code.push(OP_I64_SHL),
                BinaryOp::Shr => code.push(OP_I64_SHR_S),
                _ => {}
            }
        } else if ty == Type::Double {
            match op {
                BinaryOp::Add => code.push(OP_F64_ADD),
                BinaryOp::Sub => code.push(OP_F64_SUB),
                BinaryOp::Mul => code.push(OP_F64_MUL),
                BinaryOp::Div => code.push(OP_F64_DIV),
                _ => {}
            }
        } else if ty == Type::Float {
            match op {
                BinaryOp::Add => code.push(OP_F32_ADD),
                BinaryOp::Sub => code.push(OP_F32_SUB),
                BinaryOp::Mul => code.push(OP_F32_MUL),
                BinaryOp::Div => code.push(OP_F32_DIV),
                _ => {}
            }
        } else {
            match op {
                BinaryOp::Add => code.push(OP_I32_ADD),
                BinaryOp::Sub => code.push(OP_I32_SUB),
                BinaryOp::Mul => code.push(OP_I32_MUL),
                BinaryOp::Div => code.push(OP_I32_DIV_S),
                BinaryOp::Rem => code.push(OP_I32_REM_S),
                BinaryOp::BitAnd => code.push(OP_I32_AND),
                BinaryOp::BitOr => code.push(OP_I32_OR),
                BinaryOp::BitXor => code.push(OP_I32_XOR),
                BinaryOp::Shl => code.push(OP_I32_SHL),
                BinaryOp::Shr => code.push(OP_I32_SHR_S),
                _ => {}
            }
        }

        self.emit_store(&ty, 0, code);
        self.compile_expr(target, ctx, code)?;

        Ok(())
    }
}
