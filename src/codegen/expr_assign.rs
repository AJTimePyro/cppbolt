use super::*;

impl<'a> Codegen<'a> {
    /// Lowers assignment. The destination address is computed first and
    /// spilled to a frame slot so evaluating the RHS cannot change the target.
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
            return self.compile_assign_struct(target, value, sname, ctx, code);
        }

        let target_ty = self.get_expr_type(target, ctx);
        let val_ty = self.get_expr_type(value, ctx);
        let dest_slot = ctx.alloc_frame(4, 4)?;
        self.compile_lvalue_addr(target, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1); // dest addr
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, dest_slot);

        self.compile_expr(value, ctx, code)?;

        self.emit_conversion(&val_ty, &target_ty, code);

        self.emit_assign_store(&target_ty, dest_slot, ctx, code);

        Ok(())
    }

    fn compile_assign_struct(
        &mut self,
        target: &Expr,
        value: &Expr,
        sname: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
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
        }
        self.compile_assign_struct_copy(target, value, sname, ctx, code)
    }

    /// Implicit copy assignment is bytewise for this subset's layout; every
    /// byte in the record is part of the object.
    fn compile_assign_struct_copy(
        &mut self,
        target: &Expr,
        value: &Expr,
        sname: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let dest_slot = ctx.alloc_frame(4, 4)?;
        let src_slot = ctx.alloc_frame(4, 4)?;
        self.compile_lvalue_addr(target, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch2); // dest addr
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch2);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, dest_slot);

        self.compile_lvalue_addr(value, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1); // src addr
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, src_slot);

        let size = self
            .sema
            .struct_layouts
            .get(sname)
            .map(|l| l.size)
            .unwrap_or(4);
        for i in 0..size {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, dest_slot);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, src_slot);
            code.push(OP_I32_LOAD8_U);
            write_u32(code, 0);
            write_u32(code, i as u32);
            code.push(OP_I32_STORE8);
            write_u32(code, 0);
            write_u32(code, i as u32);
        }
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, dest_slot);
        Ok(())
    }

    /// Stores the converted value into the spilled destination address and
    /// leaves the assigned value on the stack.
    fn emit_assign_store(
        &mut self,
        target_ty: &Type,
        dest_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        let target_raw = target_ty.strip_const();
        if matches!(target_raw, Type::LongLong | Type::UnsignedLongLong) {
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch_i64);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, dest_slot);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_i64);
            self.emit_store(target_ty, 0, code);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_i64);
        } else if matches!(target_raw, Type::Double) {
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch_f64);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, dest_slot);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_f64);
            self.emit_store(target_ty, 0, code);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_f64);
        } else if matches!(target_raw, Type::Float) {
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch_f32);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, dest_slot);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_f32);
            self.emit_store(target_ty, 0, code);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch_f32);
        } else {
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch2);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, dest_slot);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch2);
            self.emit_store(target_ty, 0, code);

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch2);
        }
    }

    /// Lowers compound assignment without evaluating the target twice.
    ///
    /// The lvalue address is spilled to a frame slot, then the current value is
    /// loaded, combined in the usual arithmetic common type, and stored back.
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
        let op_name = compound_assign_operator_name(op);
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
        self.compile_compound_assign_scalar(op, target, value, &ty, &val_ty, ctx, code)
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_compound_assign_scalar(
        &mut self,
        op: &BinaryOp,
        target: &Expr,
        value: &Expr,
        ty: &Type,
        val_ty: &Type,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let is_pointer_arithmetic =
            ty.is_pointer() && (op == &BinaryOp::Add || op == &BinaryOp::Sub);
        let arithmetic_ty = if is_pointer_arithmetic {
            ty.clone()
        } else if matches!(op, BinaryOp::Shl | BinaryOp::Shr) {
            promote_integer(ty)
        } else {
            arithmetic_common_type(ty, val_ty)
        };
        let dest_slot = ctx.alloc_frame(4, 4)?;
        self.compile_lvalue_addr(target, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, dest_slot);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, dest_slot);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, dest_slot);
        self.emit_load(ty, 0, code);
        self.emit_conversion(ty, &arithmetic_ty, code);
        self.compile_expr(value, ctx, code)?;

        if is_pointer_arithmetic {
            self.emit_conversion(val_ty, &Type::UnsignedInt, code);
            let esize = ty
                .deref_type()
                .map(|d| d.size_of(&self.sema.struct_sizes))
                .unwrap_or(4)
                .max(1) as i32;
            code.push(OP_I32_CONST);
            write_i32(code, esize);
            code.push(OP_I32_MUL);
        } else {
            self.emit_conversion(val_ty, &arithmetic_ty, code);
        }

        emit_compound_operator(op, &arithmetic_ty, code);

        self.emit_conversion(&arithmetic_ty, ty, code);
        self.emit_store(ty, 0, code);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, dest_slot);
        self.emit_load(ty, 0, code);

        Ok(())
    }
}

fn compound_assign_operator_name(op: &BinaryOp) -> String {
    format!(
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
    )
}

fn emit_compound_operator(op: &BinaryOp, arithmetic_ty: &Type, code: &mut Vec<u8>) {
    let unsigned = !arithmetic_ty.is_signed();
    if *arithmetic_ty == Type::LongLong || *arithmetic_ty == Type::UnsignedLongLong {
        match op {
            BinaryOp::Add => code.push(OP_I64_ADD),
            BinaryOp::Sub => code.push(OP_I64_SUB),
            BinaryOp::Mul => code.push(OP_I64_MUL),
            BinaryOp::Div => code.push(if unsigned { OP_I64_DIV_U } else { OP_I64_DIV_S }),
            BinaryOp::Rem => code.push(if unsigned { OP_I64_REM_U } else { OP_I64_REM_S }),
            BinaryOp::BitAnd => code.push(OP_I64_AND),
            BinaryOp::BitOr => code.push(OP_I64_OR),
            BinaryOp::BitXor => code.push(OP_I64_XOR),
            BinaryOp::Shl => code.push(OP_I64_SHL),
            BinaryOp::Shr => code.push(if unsigned { OP_I64_SHR_U } else { OP_I64_SHR_S }),
            _ => {}
        }
    } else if *arithmetic_ty == Type::Double {
        match op {
            BinaryOp::Add => code.push(OP_F64_ADD),
            BinaryOp::Sub => code.push(OP_F64_SUB),
            BinaryOp::Mul => code.push(OP_F64_MUL),
            BinaryOp::Div => code.push(OP_F64_DIV),
            _ => {}
        }
    } else if *arithmetic_ty == Type::Float {
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
            BinaryOp::Div => code.push(if unsigned { OP_I32_DIV_U } else { OP_I32_DIV_S }),
            BinaryOp::Rem => code.push(if unsigned { OP_I32_REM_U } else { OP_I32_REM_S }),
            BinaryOp::BitAnd => code.push(OP_I32_AND),
            BinaryOp::BitOr => code.push(OP_I32_OR),
            BinaryOp::BitXor => code.push(OP_I32_XOR),
            BinaryOp::Shl => code.push(OP_I32_SHL),
            BinaryOp::Shr => code.push(if unsigned { OP_I32_SHR_U } else { OP_I32_SHR_S }),
            _ => {}
        }
    }
}
