use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_expr(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let _guard = self
            .depth
            .enter(self.sema.current_line, self.sema.current_col)?;
        match expr {
            Expr::IntLiteral(..)
            | Expr::FloatLiteral(..)
            | Expr::BoolLiteral(..)
            | Expr::CharLiteral(..)
            | Expr::StringLiteral(..)
            | Expr::This
            | Expr::SizeofType(..)
            | Expr::SizeofExpr(..) => self.compile_literal(expr, ctx, code)?,
            Expr::Identifier { .. } => self.compile_identifier(expr, ctx, code)?,
            Expr::Binary {
                op: BinaryOp::LogicalAnd,
                ..
            } => self.compile_logical_and(expr, ctx, code)?,
            Expr::Binary {
                op: BinaryOp::LogicalOr,
                ..
            } => self.compile_logical_or(expr, ctx, code)?,
            Expr::Binary { .. } => self.compile_binary(expr, ctx, code)?,
            Expr::Unary { .. } => self.compile_unary(expr, ctx, code)?,
            Expr::Assign { .. } => self.compile_assign(expr, ctx, code)?,
            Expr::CompoundAssign { .. } => self.compile_compound_assign(expr, ctx, code)?,
            Expr::New { .. } => self.compile_new(expr, ctx, code)?,
            Expr::Delete { .. } => self.compile_delete(expr, ctx, code)?,
            Expr::Call { .. } => {
                self.compile_call(expr, ctx, code)?;
                let ty = self.get_expr_type_raw(expr, ctx);
                let value_ty = ty.strip_const().strip_reference();
                if ty.is_reference() && !matches!(value_ty, Type::Struct(_) | Type::Array(_, _)) {
                    self.emit_load(value_ty, 0, code);
                }
            }
            Expr::ArraySubscript { .. } | Expr::MemberAccess { .. } => {
                let ty = self.get_expr_type(expr, ctx);
                self.compile_lvalue_addr(expr, ctx, code)?;
                if !matches!(ty.strip_const(), Type::Struct(_) | Type::Array(_, _)) {
                    self.emit_load(&ty, 0, code);
                }
            }
            Expr::Ternary { .. } => self.compile_ternary(expr, ctx, code)?,
            Expr::Cast { target_type, expr } => {
                let src_ty = self.get_expr_type(expr, ctx);
                self.compile_expr(expr, ctx, code)?;
                if target_type == &Type::Void {
                    if src_ty != Type::Void {
                        code.push(OP_DROP);
                    }
                    return Ok(());
                }

                self.emit_conversion(&src_ty, target_type, code);
            }
            _ => {}
        }
        Ok(())
    }

    /// Emits literals, `this`, and the two `sizeof` forms.
    fn compile_literal(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        match expr {
            Expr::IntLiteral(val, ty) => {
                if *ty == Type::LongLong || *ty == Type::UnsignedLongLong {
                    code.push(OP_I64_CONST);
                    write_i64(code, *val);
                } else {
                    code.push(OP_I32_CONST);
                    write_i32(code, *val as i32);
                }
            }
            Expr::FloatLiteral(val, ty) => {
                if *ty == Type::Double {
                    code.push(OP_F64_CONST);
                    code.extend_from_slice(&val.to_le_bytes());
                } else {
                    code.push(OP_F32_CONST);
                    code.extend_from_slice(&(*val as f32).to_le_bytes());
                }
            }
            Expr::BoolLiteral(val) => {
                code.push(OP_I32_CONST);
                write_i32(code, if *val { 1 } else { 0 });
            }
            Expr::CharLiteral(val) => {
                code.push(OP_I32_CONST);
                write_i32(code, *val as i32);
            }
            Expr::StringLiteral(s) => {
                let offset = self.string_table.get(s).cloned().unwrap_or(0);
                code.push(OP_I32_CONST);
                write_i32(code, offset as i32);
            }
            Expr::This => {
                code.push(OP_LOCAL_GET);
                write_u32(code, 0); // param 0 = this
            }
            Expr::SizeofType(inner) => {
                let size = inner.size_of(&self.sema.struct_sizes) as i32;
                code.push(OP_I32_CONST);
                write_i32(code, size);
            }
            Expr::SizeofExpr(inner) => {
                let ty = self.get_expr_type(inner, ctx);
                let size = ty.size_of(&self.sema.struct_sizes) as i32;
                code.push(OP_I32_CONST);
                write_i32(code, size);
            }
            _ => {}
        }
        Ok(())
    }

    /// Emits a conditional expression with a unified result type.
    fn compile_ternary(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } = expr
        else {
            return Ok(());
        };
        let result_ty = self.get_expr_type(expr, ctx);
        let then_ty = self.get_expr_type(then_expr, ctx);
        let else_ty = self.get_expr_type(else_expr, ctx);
        let wty = result_ty
            .to_wasm_type()
            .map(|w| match w {
                WasmType::I32 => TYPE_I32,
                WasmType::I64 => TYPE_I64,
                WasmType::F32 => TYPE_F32,
                WasmType::F64 => TYPE_F64,
            })
            .unwrap_or(TYPE_I32);

        self.compile_cond(cond, ctx, code)?;
        code.push(OP_IF);
        code.push(wty);
        self.compile_expr(then_expr, ctx, code)?;
        self.emit_conversion(&then_ty, &result_ty, code);
        code.push(OP_ELSE);
        self.compile_expr(else_expr, ctx, code)?;
        self.emit_conversion(&else_ty, &result_ty, code);
        code.push(OP_END);
        Ok(())
    }
}
