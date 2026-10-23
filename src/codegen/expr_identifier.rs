use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_identifier(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::Identifier(name) = expr else {
            return Err(self.internal_error("unexpected expression kind"));
        };

        if let Some(&(offset, ref ty)) = ctx.static_var_offsets.get(name) {
            self.emit_identifier_address(offset as i32, ty, code);
            return Ok(());
        }
        if let Some(&offset) = ctx.var_offsets.get(name) {
            self.emit_local_identifier(name, offset, ctx, code);
            return Ok(());
        }
        if let Some(cls) = ctx.current_class.clone() {
            return self.compile_identifier_class_member(&cls, name, ctx, code);
        }
        if let Some(&val) = self.sema.enum_values.get(name) {
            code.push(OP_I32_CONST);
            write_i32(code, val as i32);
            return Ok(());
        }
        if let Some(&table_idx) = self
            .func_table_indices
            .get(name)
            .or_else(|| self.func_table_indices.get(&format!("std::{}", name)))
            .or_else(|| {
                self.func_table_indices
                    .get(QualifiedName::new(name).plain())
            })
        {
            code.push(OP_I32_CONST);
            write_i32(code, table_idx as i32);
            return Ok(());
        }
        if let Some(sym) = self
            .sema
            .lookup_var_in_context(name, &ctx.current_func_name)
        {
            self.emit_context_identifier(sym, name, code);
            return Ok(());
        }
        if let Some(&offset) = self.global_offsets.get(name) {
            let ty = self
                .sema
                .global_vars
                .get(name)
                .map(|s| &s.ty)
                .cloned()
                .unwrap_or(Type::Int);
            self.emit_identifier_address(offset as i32, &ty, code);
            return Ok(());
        }
        Err(CompileError::new(
            format!("Undefined variable '{}'", name),
            self.current_line,
            self.current_col,
        ))
    }

    /// Emits the address of an aggregate, or the loaded value of a scalar.
    fn emit_identifier_address(&mut self, offset: i32, ty: &Type, code: &mut Vec<u8>) {
        code.push(OP_I32_CONST);
        write_i32(code, offset);
        if !matches!(ty.strip_const(), Type::Struct(_) | Type::Array(_, _)) {
            self.emit_load(ty, 0, code);
        }
    }

    fn emit_local_identifier(
        &mut self,
        name: &str,
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) {
        let ty = ctx.var_types.get(name).cloned().unwrap_or(Type::Int);
        if ty.is_reference() {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, offset);
            if let Some(inner) = ty.deref_type()
                && !matches!(inner.strip_const(), Type::Struct(_))
            {
                self.emit_load(inner, 0, code);
            }
        } else if matches!(ty.strip_const(), Type::Struct(_) | Type::Array(_, _)) {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_CONST);
            write_i32(code, offset as i32);
            code.push(OP_I32_ADD);
        } else {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            self.emit_load(&ty, offset, code);
        }
    }

    fn compile_identifier_class_member(
        &mut self,
        cls: &str,
        name: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if let Some(fld) = self.sema.find_field_in_class(cls, name) {
            if matches!(fld.ty.strip_const(), Type::Struct(_) | Type::Array(_, _)) {
                code.push(OP_LOCAL_GET);
                write_u32(code, 0); // this
                code.push(OP_I32_CONST);
                write_i32(code, fld.offset as i32);
                code.push(OP_I32_ADD);
            } else {
                code.push(OP_LOCAL_GET);
                write_u32(code, 0); // this
                self.emit_load(&fld.ty, fld.offset as u32, code);
            }
            return Ok(());
        }
        if let Some(&offset) = self
            .global_offsets
            .get(&format!("{}::{}", cls, name))
            .or_else(|| self.global_offsets.get(name))
        {
            let ty = self
                .sema
                .lookup_var(&format!("{}::{}", cls, name))
                .or_else(|| {
                    self.sema
                        .lookup_var_in_context(name, &ctx.current_func_name)
                })
                .map(|s| s.ty.clone())
                .unwrap_or(Type::Int);
            self.emit_identifier_address(offset as i32, &ty, code);
            return Ok(());
        }
        Err(CompileError::new(
            format!("Undefined variable '{}'", name),
            self.current_line,
            self.current_col,
        ))
    }

    fn emit_context_identifier(
        &mut self,
        sym: &crate::sema::VarSymbol,
        name: &str,
        code: &mut Vec<u8>,
    ) {
        if sym.is_global {
            let offset = self
                .global_offsets
                .get(&sym.name)
                .or_else(|| self.global_offsets.get(name))
                .or_else(|| {
                    name.strip_prefix("::")
                        .and_then(|s| self.global_offsets.get(s))
                })
                .copied();
            if let Some(off) = offset {
                if matches!(sym.ty.strip_const(), Type::Struct(_) | Type::Array(_, _)) {
                    code.push(OP_I32_CONST);
                    write_i32(code, off as i32);
                } else {
                    code.push(OP_I32_CONST);
                    write_i32(code, off as i32);
                    self.emit_load(&sym.ty, 0, code);
                }
            } else {
                code.push(OP_I32_CONST);
                write_i32(code, sym.stack_offset);
                self.emit_load(&sym.ty, 0, code);
            }
        } else {
            code.push(OP_I32_CONST);
            write_i32(code, sym.stack_offset);
            self.emit_load(&sym.ty, 0, code);
        }
    }
}
