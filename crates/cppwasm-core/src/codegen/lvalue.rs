use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn is_struct_lvalue(&self, expr: &Expr, ctx: &FunctionContext) -> bool {
        match expr {
            Expr::Identifier(_) | Expr::MemberAccess { .. } => true,
            Expr::Unary {
                op: UnaryOp::Deref, ..
            } => true,
            Expr::ArraySubscript { .. } => true,
            Expr::Call { callee, args } => {
                let arg_types: Vec<Type> =
                    args.iter().map(|a| self.get_expr_type(a, ctx)).collect();
                if let Expr::Identifier(name) = &**callee {
                    if let Some((_, sig)) = self.find_function(name, &arg_types) {
                        return sig.return_type.is_reference();
                    }
                } else if let Expr::MemberAccess { object, member, .. } = &**callee {
                    let obj_ty = self.get_expr_type(object, ctx);
                    if let Type::Struct(sname) = obj_ty.strip_const().strip_reference()
                        && let Some((_, sig)) = self.find_method(sname, member, &arg_types)
                    {
                        return sig.return_type.is_reference();
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub(super) fn compile_lvalue_addr(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        match expr {
            Expr::Identifier(name) => {
                if let Some(&(offset, _)) = ctx.static_var_offsets.get(name) {
                    code.push(OP_I32_CONST);
                    write_i32(code, offset as i32);
                    Ok(())
                } else if let Some(&offset) = ctx.var_offsets.get(name) {
                    let ty = ctx.var_types.get(name).cloned().unwrap_or(Type::Int);
                    if ty.is_reference() {
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.fp_local);
                        code.push(OP_I32_LOAD);
                        write_u32(code, 2);
                        write_u32(code, offset);
                    } else {
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.fp_local);
                        code.push(OP_I32_CONST);
                        write_i32(code, offset as i32);
                        code.push(OP_I32_ADD);
                    }
                    Ok(())
                } else if let Some(cls) = &ctx.current_class {
                    if let Some(fld) = self.sema.find_field_in_class(cls, name) {
                        code.push(OP_LOCAL_GET);
                        write_u32(code, 0); // this
                        code.push(OP_I32_CONST);
                        write_i32(code, fld.offset as i32);
                        code.push(OP_I32_ADD);
                        Ok(())
                    } else if let Some(&offset) = self
                        .global_offsets
                        .get(&format!("{}::{}", cls, name))
                        .or_else(|| self.global_offsets.get(name))
                    {
                        code.push(OP_I32_CONST);
                        write_i32(code, offset as i32);
                        Ok(())
                    } else {
                        Err(CompileError::new(
                            format!("Undefined variable for lvalue: {}", name),
                            1,
                            1,
                        ))
                    }
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
                            code.push(OP_I32_CONST);
                            write_i32(code, off as i32);
                            Ok(())
                        } else {
                            code.push(OP_I32_CONST);
                            write_i32(code, sym.stack_offset);
                            Ok(())
                        }
                    } else {
                        code.push(OP_I32_CONST);
                        write_i32(code, sym.stack_offset);
                        Ok(())
                    }
                } else if let Some(&offset) = self.global_offsets.get(name) {
                    code.push(OP_I32_CONST);
                    write_i32(code, offset as i32);
                    Ok(())
                } else {
                    Err(CompileError::new(
                        format!("Undefined variable for lvalue: {}", name),
                        1,
                        1,
                    ))
                }
            }
            Expr::Unary {
                op: UnaryOp::Deref,
                expr,
            } => self.compile_expr(expr, ctx, code),
            Expr::ArraySubscript { array, index } => {
                let arr_ty = self.get_expr_type(array, ctx);
                let clean_ty = arr_ty.strip_const();
                if let Type::Struct(sname) = clean_ty {
                    let idx_ty = self.get_expr_type(index, ctx);
                    if let Some((m_idx, sig)) = self.find_method(sname, "operator[]", &[idx_ty]) {
                        self.compile_lvalue_addr(array, ctx, code)?;
                        let is_r = sig
                            .param_types
                            .get(1)
                            .map(|p| p.is_reference())
                            .unwrap_or(false);
                        if is_r {
                            self.compile_lvalue_addr(index, ctx, code)?;
                        } else {
                            self.compile_expr(index, ctx, code)?;
                        }
                        code.push(OP_CALL);
                        write_u32(code, m_idx);
                        return Ok(());
                    }
                }
                let elem_size = match clean_ty {
                    Type::Array(elem, _) | Type::Pointer(elem) => {
                        elem.size_of(&self.sema.struct_sizes) as i32
                    }
                    _ => 4,
                };
                self.compile_expr(array, ctx, code)?;
                self.compile_expr(index, ctx, code)?;
                code.push(OP_I32_CONST);
                write_i32(code, elem_size);
                code.push(OP_I32_MUL);
                code.push(OP_I32_ADD);
                Ok(())
            }
            Expr::MemberAccess {
                object,
                member,
                is_arrow,
            } => {
                let obj_ty = self.get_expr_type_raw(object, ctx);
                let is_ref = obj_ty.is_reference();
                let sname = if *is_arrow || is_ref {
                    if let Type::Pointer(inner) | Type::Reference(inner) = obj_ty.strip_const() {
                        if let Type::Struct(s) = inner.strip_const() {
                            s.clone()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    if let Type::Struct(s) = obj_ty.strip_const() {
                        s.clone()
                    } else {
                        String::new()
                    }
                };
                let offset = if let Some(layout) = self.sema.struct_layouts.get(&sname) {
                    layout
                        .fields
                        .iter()
                        .find(|f| &f.name == member)
                        .map(|f| f.offset as i32)
                        .unwrap_or(0)
                } else {
                    0
                };

                if *is_arrow {
                    self.compile_expr(object, ctx, code)?;
                } else {
                    self.compile_lvalue_addr(object, ctx, code)?;
                }
                code.push(OP_I32_CONST);
                write_i32(code, offset);
                code.push(OP_I32_ADD);
                Ok(())
            }
            Expr::Call { .. } | Expr::Binary { .. } | Expr::Unary { .. } => {
                let ty = self.get_expr_type_raw(expr, ctx);
                if ty.is_reference() || matches!(ty.strip_const(), Type::Struct(_)) {
                    self.compile_expr(expr, ctx, code)
                } else {
                    // Create temporary stack slot for non-reference call return value
                    let offset = ctx.next_local_offset;
                    let size = ty.size_of(&self.sema.struct_sizes) as u32;
                    ctx.next_local_offset += (size + 3) & !3;
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    self.compile_expr(expr, ctx, code)?;
                    self.emit_store(&ty, offset, code);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_I32_CONST);
                    write_i32(code, offset as i32);
                    code.push(OP_I32_ADD);
                    Ok(())
                }
            }
            _ => {
                let ty = self.get_expr_type(expr, ctx);
                let offset = ctx.next_local_offset;
                let size = ty.size_of(&self.sema.struct_sizes) as u32;
                ctx.next_local_offset += (size + 3) & !3;
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                self.compile_expr(expr, ctx, code)?;
                self.emit_store(&ty, offset, code);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_CONST);
                write_i32(code, offset as i32);
                code.push(OP_I32_ADD);
                Ok(())
            }
        }
    }
}
