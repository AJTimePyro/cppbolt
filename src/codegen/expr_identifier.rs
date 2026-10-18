use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_identifier(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::Identifier(name) = expr else {
            return Err(crate::internal_error!(0, 0, "unexpected expression kind"));
        };

        if let Some(&(offset, ref ty)) = ctx.static_var_offsets.get(name) {
            if matches!(ty.strip_const(), Type::Struct(_) | Type::Array(_, _)) {
                code.push(OP_I32_CONST);
                write_i32(code, offset as i32);
            } else {
                code.push(OP_I32_CONST);
                write_i32(code, offset as i32);
                self.emit_load(ty, 0, code);
            }
        } else if let Some(&offset) = ctx.var_offsets.get(name) {
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
        } else if let Some(cls) = &ctx.current_class {
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
            } else if let Some(&offset) = self
                .global_offsets
                .get(&format!("{}::{}", cls, name))
                .or_else(|| self.global_offsets.get(name))
            {
                let ty = self
                    .sema
                    .lookup_var(&format!("{}::{}", cls, name))
                    .or_else(|| {
                        self.sema
                            .lookup_var_in_context(name, Some(&ctx.current_func_name))
                    })
                    .map(|s| s.ty.clone())
                    .unwrap_or(Type::Int);
                if matches!(ty.strip_const(), Type::Struct(_) | Type::Array(_, _)) {
                    code.push(OP_I32_CONST);
                    write_i32(code, offset as i32);
                } else {
                    code.push(OP_I32_CONST);
                    write_i32(code, offset as i32);
                    self.emit_load(&ty, 0, code);
                }
            } else {
                return Err(CompileError::new(
                    format!("Undefined variable '{}'", name),
                    1,
                    1,
                ));
            }
        } else if let Some(&val) = self.sema.enum_values.get(name) {
            code.push(OP_I32_CONST);
            write_i32(code, val as i32);
        } else if let Some(&table_idx) = self
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
        } else if let Some(sym) = self
            .sema
            .lookup_var_in_context(name, Some(&ctx.current_func_name))
        {
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
        } else if let Some(&offset) = self.global_offsets.get(name) {
            let ty = self
                .sema
                .global_vars
                .get(name)
                .map(|s| &s.ty)
                .cloned()
                .unwrap_or(Type::Int);
            if matches!(ty.strip_const(), Type::Struct(_) | Type::Array(_, _)) {
                code.push(OP_I32_CONST);
                write_i32(code, offset as i32);
            } else {
                code.push(OP_I32_CONST);
                write_i32(code, offset as i32);
                self.emit_load(&ty, 0, code);
            }
        } else {
            return Err(CompileError::new(
                format!("Undefined variable '{}'", name),
                1,
                1,
            ));
        }

        Ok(())
    }
}
