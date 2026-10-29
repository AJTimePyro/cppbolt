use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_unary(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::Unary { op, expr, .. } = expr else {
            return Err(self.internal_error("unexpected expression kind"));
        };

        let ty = self.get_expr_type(expr, ctx);
        match op {
            UnaryOp::Not => self.compile_unary_not(expr, &ty, ctx, code)?,
            UnaryOp::BitNot => self.compile_unary_bitnot(expr, &ty, ctx, code)?,
            UnaryOp::Neg => self.compile_unary_neg(expr, &ty, ctx, code)?,
            UnaryOp::Plus => self.compile_expr(expr, ctx, code)?,
            UnaryOp::AddrOf => self.compile_unary_addrof(expr, &ty, ctx, code)?,
            UnaryOp::Deref => self.compile_unary_deref(expr, &ty, ctx, code)?,
            UnaryOp::PreInc | UnaryOp::PreDec => {
                let is_inc = matches!(op, UnaryOp::PreInc);
                self.compile_unary_pre_incdec(expr, &ty, is_inc, ctx, code)?;
            }
            UnaryOp::PostInc | UnaryOp::PostDec => {
                let is_inc = matches!(op, UnaryOp::PostInc);
                self.compile_unary_post_incdec(expr, &ty, is_inc, ctx, code)?;
            }
        }

        Ok(())
    }

    fn compile_unary_not(
        &mut self,
        expr: &Expr,
        ty: &Type,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
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
            } else if let Some((f_idx, sig)) = self.find_free_operator("operator!", ty) {
                self.compile_call_arg(expr, sig.param_types.first(), ctx, code)?;
                code.push(OP_CALL);
                write_u32(code, f_idx);
                return Ok(());
            }
        }
        self.compile_cond(expr, ctx, code)?;
        code.push(OP_I32_EQZ);
        Ok(())
    }

    fn compile_unary_bitnot(
        &mut self,
        expr: &Expr,
        ty: &Type,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Type::Struct(sname) = ty.strip_const() {
            if let Some((m_idx, sig)) = self.find_method(sname, "operator~", &[]) {
                self.compile_lvalue_addr(expr, ctx, code)?;
                code.push(OP_CALL);
                write_u32(code, m_idx);
                self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
                return Ok(());
            }
            if let Some((f_idx, sig)) = self.find_free_operator("operator~", ty) {
                self.compile_call_arg(expr, sig.param_types.first(), ctx, code)?;
                code.push(OP_CALL);
                write_u32(code, f_idx);
                self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
                return Ok(());
            }
        }
        self.compile_expr(expr, ctx, code)?;
        code.push(OP_I32_CONST);
        write_i32(code, -1);
        code.push(OP_I32_XOR);
        Ok(())
    }

    fn compile_unary_neg(
        &mut self,
        expr: &Expr,
        ty: &Type,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Type::Struct(sname) = ty.strip_const() {
            if let Some((m_idx, sig)) = self.find_method(sname, "operator-", &[]) {
                self.compile_lvalue_addr(expr, ctx, code)?;
                code.push(OP_CALL);
                write_u32(code, m_idx);
                self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
                return Ok(());
            }
            if let Some((f_idx, sig)) = self.find_free_operator("operator-", ty) {
                self.compile_call_arg(expr, sig.param_types.first(), ctx, code)?;
                code.push(OP_CALL);
                write_u32(code, f_idx);
                self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
                return Ok(());
            }
        }
        if *ty == Type::Double {
            self.compile_expr(expr, ctx, code)?;
            code.push(OP_F64_NEG);
        } else if *ty == Type::Float {
            self.compile_expr(expr, ctx, code)?;
            code.push(OP_F32_NEG);
        } else if *ty == Type::LongLong || *ty == Type::UnsignedLongLong {
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
        Ok(())
    }

    fn compile_unary_addrof(
        &mut self,
        expr: &Expr,
        ty: &Type,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Type::Struct(sname) = ty.strip_const() {
            if let Some((m_idx, sig)) = self.find_method(sname, "operator&", &[]) {
                self.compile_lvalue_addr(expr, ctx, code)?;
                code.push(OP_CALL);
                write_u32(code, m_idx);
                self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
                return Ok(());
            }
            if let Some((f_idx, sig)) = self.find_free_operator("operator&", ty) {
                self.compile_call_arg(expr, sig.param_types.first(), ctx, code)?;
                code.push(OP_CALL);
                write_u32(code, f_idx);
                self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
                return Ok(());
            }
        }
        if let Expr::Identifier(name) = expr
            && let Some(&table_idx) = self.func_table_indices.get(name.as_str())
        {
            code.push(OP_I32_CONST);
            write_i32(code, table_idx as i32);
            return Ok(());
        }
        self.compile_lvalue_addr(expr, ctx, code)?;
        Ok(())
    }

    fn compile_unary_deref(
        &mut self,
        expr: &Expr,
        ty: &Type,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Type::Struct(sname) = ty.strip_const()
            && let Some((m_idx, sig)) = self.find_method(sname, "operator*", &[])
        {
            self.compile_lvalue_addr(expr, ctx, code)?;
            code.push(OP_CALL);
            write_u32(code, m_idx);
            self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
            if sig.return_type.is_reference() {
                let val_ty = sig.return_type.strip_const().strip_reference();
                if !matches!(val_ty, Type::Struct(_) | Type::Array(_, _)) {
                    self.emit_load(val_ty, 0, code);
                }
            }
            return Ok(());
        }
        if let Some((f_idx, sig)) = self.find_free_operator("operator*", ty) {
            self.compile_call_arg(expr, sig.param_types.first(), ctx, code)?;
            code.push(OP_CALL);
            write_u32(code, f_idx);
            self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
            if sig.return_type.is_reference() {
                let val_ty = sig.return_type.strip_const().strip_reference();
                if !matches!(val_ty, Type::Struct(_) | Type::Array(_, _)) {
                    self.emit_load(val_ty, 0, code);
                }
            }
            return Ok(());
        }
        self.compile_expr(expr, ctx, code)?;
        let elem_ty = ty.deref_type().cloned().unwrap_or(Type::Int);
        if !matches!(elem_ty.strip_const(), Type::Struct(_) | Type::Array(_, _)) {
            self.emit_load(&elem_ty, 0, code);
        }
        Ok(())
    }

    fn compile_unary_pre_incdec(
        &mut self,
        expr: &Expr,
        ty: &Type,
        is_inc: bool,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Type::Struct(sname) = ty.strip_const() {
            let m_name = if is_inc { "operator++" } else { "operator--" };
            if let Some((m_idx, sig)) = self.find_method(sname, m_name, &[]) {
                self.compile_lvalue_addr(expr, ctx, code)?;
                code.push(OP_CALL);
                write_u32(code, m_idx);
                self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
                return Ok(());
            }
        }
        self.compile_lvalue_addr(expr, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        self.emit_load(ty, 0, code);

        self.emit_pre_incdec_numeric(ty, is_inc, ctx, code);
        Ok(())
    }

    fn emit_pre_incdec_numeric(
        &mut self,
        ty: &Type,
        is_inc: bool,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
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
            self.emit_store(ty, 0, code);

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
            self.emit_store(ty, 0, code);

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
            self.emit_store(ty, 0, code);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_f32);
        } else {
            let step = if ty.is_pointer() {
                ty.deref_type()
                    .map(|d| d.size_of(&self.sema.struct_sizes).max(1))
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
            self.emit_store(ty, 0, code);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch2);
        }
    }

    fn compile_unary_post_incdec(
        &mut self,
        expr: &Expr,
        ty: &Type,
        is_inc: bool,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Type::Struct(sname) = ty.strip_const() {
            let m_name = if is_inc { "operator++" } else { "operator--" };
            if let Some((m_idx, sig)) = self.find_method(sname, m_name, &[Type::Int]) {
                self.compile_lvalue_addr(expr, ctx, code)?;
                code.push(OP_I32_CONST);
                write_i32(code, 0);
                code.push(OP_CALL);
                write_u32(code, m_idx);
                self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
                return Ok(());
            }
        }
        self.compile_lvalue_addr(expr, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        self.emit_load(ty, 0, code);

        self.emit_post_incdec_numeric(ty, is_inc, ctx, code);
        Ok(())
    }

    fn emit_post_incdec_numeric(
        &mut self,
        ty: &Type,
        is_inc: bool,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
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
            self.emit_store(ty, 0, code);

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
            self.emit_store(ty, 0, code);

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
            self.emit_store(ty, 0, code);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_f32);
        } else {
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch2);

            let step = if ty.is_pointer() {
                ty.deref_type()
                    .map(|d| d.size_of(&self.sema.struct_sizes).max(1))
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
            self.emit_store(ty, 0, code);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch2);
        }
    }
}
