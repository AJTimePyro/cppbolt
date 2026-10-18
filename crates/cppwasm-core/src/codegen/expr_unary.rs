use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_unary(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::Unary { op, expr } = expr else {
            return Err(crate::internal_error!(0, 0, "unexpected expression kind"));
        };

        let ty = self.get_expr_type(expr, ctx);
        match op {
            UnaryOp::Not => {
                let ty = self.get_expr_type(expr, ctx);
                if let Type::Struct(sname) = ty.strip_const() {
                    if let Some((m_idx, _)) = self.find_method(sname, "operator!", &[]) {
                        self.compile_lvalue_addr(expr, ctx, code)?;
                        code.push(OP_CALL);
                        write_u32(code, m_idx);
                        return Ok(());
                    } else if let Some((m_idx, _)) = self.find_method(sname, "operator bool", &[]) {
                        self.compile_lvalue_addr(expr, ctx, code)?;
                        code.push(OP_CALL);
                        write_u32(code, m_idx);
                        code.push(OP_I32_EQZ);
                        return Ok(());
                    }
                }
                self.compile_cond(expr, ctx, code)?;
                code.push(OP_I32_EQZ);
            }
            UnaryOp::BitNot => {
                self.compile_expr(expr, ctx, code)?;
                code.push(OP_I32_CONST);
                write_i32(code, -1);
                code.push(OP_I32_XOR);
            }
            UnaryOp::Neg => {
                if let Type::Struct(sname) = ty.strip_const()
                    && let Some((m_idx, sig)) = self.find_method(sname, "operator-", &[])
                {
                    self.compile_lvalue_addr(expr, ctx, code)?;
                    code.push(OP_CALL);
                    write_u32(code, m_idx);
                    self.register_returned_struct_temp(&sig.return_type, ctx, code);
                    return Ok(());
                }
                if ty == Type::Double {
                    self.compile_expr(expr, ctx, code)?;
                    code.push(OP_F64_NEG);
                } else if ty == Type::Float {
                    self.compile_expr(expr, ctx, code)?;
                    code.push(OP_F32_NEG);
                } else if ty == Type::LongLong || ty == Type::UnsignedLongLong {
                    code.push(OP_I64_CONST);
                    write_i64(code, 0);
                    self.compile_expr(expr, ctx, code)?;
                    code.push(OP_I64_SUB);
                } else {
                    code.push(OP_I32_CONST);
                    write_i32(code, 0);
                    self.compile_expr(expr, ctx, code)?;
                    code.push(OP_I32_SUB);
                }
            }
            UnaryOp::Plus => {
                self.compile_expr(expr, ctx, code)?;
            }
            UnaryOp::AddrOf => {
                if let Expr::Identifier(name) = &**expr
                    && let Some(&table_idx) = self.func_table_indices.get(name)
                {
                    code.push(OP_I32_CONST);
                    write_i32(code, table_idx as i32);
                    return Ok(());
                }
                self.compile_lvalue_addr(expr, ctx, code)?;
            }
            UnaryOp::Deref => {
                self.compile_expr(expr, ctx, code)?;
                let elem_ty = ty.deref_type().cloned().unwrap_or(Type::Int);
                self.emit_load(&elem_ty, 0, code);
            }
            UnaryOp::PreInc | UnaryOp::PreDec => {
                let is_inc = matches!(op, UnaryOp::PreInc);
                if let Type::Struct(sname) = ty.strip_const() {
                    let m_name = if is_inc { "operator++" } else { "operator--" };
                    if let Some((m_idx, sig)) = self.find_method(sname, m_name, &[]) {
                        self.compile_lvalue_addr(expr, ctx, code)?;
                        code.push(OP_CALL);
                        write_u32(code, m_idx);
                        self.register_returned_struct_temp(&sig.return_type, ctx, code);
                        return Ok(());
                    }
                }
                self.compile_lvalue_addr(expr, ctx, code)?;
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1);

                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                self.emit_load(&ty, 0, code);

                let raw_ty = ty.strip_const();
                if matches!(raw_ty, Type::LongLong | Type::UnsignedLongLong) {
                    code.push(OP_I64_CONST);
                    write_i64(code, 1);
                    code.push(if is_inc { OP_I64_ADD } else { OP_I64_SUB });
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch_i64);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch_i64);
                    self.emit_store(&ty, 0, code);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch_i64);
                } else if matches!(raw_ty, Type::Double) {
                    code.push(OP_F64_CONST);
                    code.extend_from_slice(&1.0f64.to_le_bytes());
                    code.push(if is_inc { OP_F64_ADD } else { OP_F64_SUB });
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch_f64);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch_f64);
                    self.emit_store(&ty, 0, code);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch_f64);
                } else if matches!(raw_ty, Type::Float) {
                    code.push(OP_F32_CONST);
                    code.extend_from_slice(&1.0f32.to_le_bytes());
                    code.push(if is_inc { OP_F32_ADD } else { OP_F32_SUB });
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch_f32);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch_f32);
                    self.emit_store(&ty, 0, code);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch_f32);
                } else {
                    let step = if ty.is_pointer() {
                        ty.deref_type()
                            .map(|d| d.size_of(&self.sema.struct_sizes))
                            .unwrap_or(1) as i32
                    } else {
                        1
                    };
                    code.push(OP_I32_CONST);
                    write_i32(code, step);
                    code.push(if is_inc { OP_I32_ADD } else { OP_I32_SUB });
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch2);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch2);
                    self.emit_store(&ty, 0, code);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch2);
                }
            }
            UnaryOp::PostInc | UnaryOp::PostDec => {
                let is_inc = matches!(op, UnaryOp::PostInc);
                self.compile_lvalue_addr(expr, ctx, code)?;
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1);

                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                self.emit_load(&ty, 0, code);

                let raw_ty = ty.strip_const();
                if matches!(raw_ty, Type::LongLong | Type::UnsignedLongLong) {
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch_i64);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch_i64);
                    code.push(OP_I64_CONST);
                    write_i64(code, 1);
                    code.push(if is_inc { OP_I64_ADD } else { OP_I64_SUB });
                    self.emit_store(&ty, 0, code);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch_i64);
                } else if matches!(raw_ty, Type::Double) {
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch_f64);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch_f64);
                    code.push(OP_F64_CONST);
                    code.extend_from_slice(&1.0f64.to_le_bytes());
                    code.push(if is_inc { OP_F64_ADD } else { OP_F64_SUB });
                    self.emit_store(&ty, 0, code);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch_f64);
                } else if matches!(raw_ty, Type::Float) {
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch_f32);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch_f32);
                    code.push(OP_F32_CONST);
                    code.extend_from_slice(&1.0f32.to_le_bytes());
                    code.push(if is_inc { OP_F32_ADD } else { OP_F32_SUB });
                    self.emit_store(&ty, 0, code);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch_f32);
                } else {
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch2);

                    let step = if ty.is_pointer() {
                        ty.deref_type()
                            .map(|d| d.size_of(&self.sema.struct_sizes))
                            .unwrap_or(1) as i32
                    } else {
                        1
                    };
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch2);
                    code.push(OP_I32_CONST);
                    write_i32(code, step);
                    code.push(if is_inc { OP_I32_ADD } else { OP_I32_SUB });
                    self.emit_store(&ty, 0, code);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch2);
                }
            }
        }

        Ok(())
    }
}
