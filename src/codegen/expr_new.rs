use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_new(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::New {
            ty,
            args,
            is_array,
            array_size,
            placement,
        } = expr
        else {
            return Err(crate::internal_error!(0, 0, "unexpected expression kind"));
        };

        if let Some(place_expr) = placement {
            self.compile_expr(place_expr, ctx, code)?;
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1); // placement ptr

            if let Type::Struct(sname) = ty.strip_const() {
                if let Some(&v_off) = self.vtable_offsets.get(sname) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_I32_CONST);
                    write_i32(code, v_off as i32);
                    code.push(OP_I32_STORE);
                    write_u32(code, 2);
                    write_u32(code, 0);
                }
                let arg_types: Vec<Type> =
                    args.iter().map(|a| self.get_expr_type(a, ctx)).collect();
                if let Some(ctor_idx) = self.find_constructor(sname, &arg_types) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1); // this
                    for a in args {
                        self.compile_expr(a, ctx, code)?;
                    }
                    code.push(OP_CALL);
                    write_u32(code, ctor_idx);
                }
            } else if !args.is_empty() {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                self.compile_expr(&args[0], ctx, code)?;
                self.emit_store(ty, 0, code);
            }
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            return Ok(());
        }
        if *is_array {
            let elem_size = ty.size_of(&self.sema.struct_sizes) as i32;
            let has_struct = if let Type::Struct(sname) = ty.strip_const() {
                Some(sname.clone())
            } else {
                None
            };

            if let Some(sz_expr) = array_size {
                self.compile_expr(sz_expr, ctx, code)?;
            } else {
                code.push(OP_I32_CONST);
                write_i32(code, 1);
            }
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch2); // count

            if let Some(sname) = has_struct {
                // Allocation size: count * elem_size + 4 (for cookie)
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch2);
                code.push(OP_I32_CONST);
                write_i32(code, elem_size);
                code.push(OP_I32_MUL);
                code.push(OP_I32_CONST);
                write_i32(code, 4);
                code.push(OP_I32_ADD);
                code.push(OP_CALL);
                write_u32(code, self.func_indices["malloc"]);
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1); // base ptr

                // Store count cookie at offset 0
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch2);
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, 0);

                // user ptr = base ptr + 4
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_I32_CONST);
                write_i32(code, 4);
                code.push(OP_I32_ADD);
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1); // user ptr

                let ctor_idx = self.find_constructor(&sname, &[]);
                let v_off = self.vtable_offsets.get(&sname).copied();

                if ctor_idx.is_some() || v_off.is_some() {
                    code.push(OP_I32_CONST);
                    write_i32(code, 0);
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.ret_temp_i32); // loop index i

                    code.push(OP_BLOCK);
                    code.push(0x40);
                    code.push(OP_LOOP);
                    code.push(0x40);

                    // if i >= count break;
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.ret_temp_i32);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch2);
                    code.push(OP_I32_GE_S);
                    code.push(OP_BR_IF);
                    write_u32(code, 1);

                    if let Some(vo) = v_off {
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.scratch1);
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.ret_temp_i32);
                        code.push(OP_I32_CONST);
                        write_i32(code, elem_size);
                        code.push(OP_I32_MUL);
                        code.push(OP_I32_ADD);
                        code.push(OP_I32_CONST);
                        write_i32(code, vo as i32);
                        code.push(OP_I32_STORE);
                        write_u32(code, 2);
                        write_u32(code, 0);
                    }

                    if let Some(ci) = ctor_idx {
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.scratch1);
                        code.push(OP_LOCAL_GET);
                        write_u32(code, ctx.ret_temp_i32);
                        code.push(OP_I32_CONST);
                        write_i32(code, elem_size);
                        code.push(OP_I32_MUL);
                        code.push(OP_I32_ADD); // this
                        code.push(OP_CALL);
                        write_u32(code, ci);
                    }

                    // i++
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.ret_temp_i32);
                    code.push(OP_I32_CONST);
                    write_i32(code, 1);
                    code.push(OP_I32_ADD);
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.ret_temp_i32);

                    code.push(OP_BR);
                    write_u32(code, 0);

                    code.push(OP_END);
                    code.push(OP_END);
                }
            } else {
                // POD array
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch2);
                code.push(OP_I32_CONST);
                write_i32(code, elem_size);
                code.push(OP_I32_MUL);
                code.push(OP_CALL);
                write_u32(code, self.func_indices["malloc"]);
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1);
            }
        } else {
            let size = ty.size_of(&self.sema.struct_sizes) as i32;
            code.push(OP_I32_CONST);
            write_i32(code, size);
            code.push(OP_CALL);
            write_u32(code, self.func_indices["malloc"]);
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1); // heap obj ptr

            if let Type::Struct(sname) = ty.strip_const() {
                // Set up vtable if exists
                if let Some(&v_off) = self.vtable_offsets.get(sname) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_I32_CONST);
                    write_i32(code, v_off as i32);
                    code.push(OP_I32_STORE);
                    write_u32(code, 2);
                    write_u32(code, 0);
                }
                // Call constructor
                let arg_types: Vec<Type> =
                    args.iter().map(|a| self.get_expr_type(a, ctx)).collect();
                if let Some(ctor_idx) = self.find_constructor(sname, &arg_types) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1); // this
                    for a in args {
                        self.compile_expr(a, ctx, code)?;
                    }
                    code.push(OP_CALL);
                    write_u32(code, ctor_idx);
                }
            } else if !args.is_empty() {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1); // addr
                self.compile_expr(&args[0], ctx, code)?; // value
                self.emit_store(ty, 0, code);
            }
        }

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);

        Ok(())
    }

    pub(super) fn compile_delete(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::Delete { expr, is_array } = expr else {
            return Err(crate::internal_error!(0, 0, "unexpected expression kind"));
        };

        let ty = self.get_expr_type(expr, ctx);
        self.compile_expr(expr, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1); // ptr

        if *is_array {
            if let Some(inner) = ty.deref_type()
                && let Type::Struct(sname) = inner.strip_const()
            {
                let elem_size = inner.size_of(&self.sema.struct_sizes) as i32;
                let dtor_name = format!("{}_~{}", sname, sname);
                let dtor_idx = self.func_indices.get(&dtor_name).copied();

                if let Some(di) = dtor_idx {
                    // count = load i32 from (scratch1 - 4)
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_I32_CONST);
                    write_i32(code, 4);
                    code.push(OP_I32_SUB);
                    code.push(OP_I32_LOAD);
                    write_u32(code, 2);
                    write_u32(code, 0);
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.scratch2); // count

                    // Loop backwards: i = count;
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch2);
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.ret_temp_i32); // i = count

                    code.push(OP_BLOCK);
                    code.push(0x40);
                    code.push(OP_LOOP);
                    code.push(0x40);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.ret_temp_i32);
                    code.push(OP_I32_EQZ);
                    code.push(OP_BR_IF);
                    write_u32(code, 1);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.ret_temp_i32);
                    code.push(OP_I32_CONST);
                    write_i32(code, 1);
                    code.push(OP_I32_SUB);
                    code.push(OP_LOCAL_SET);
                    write_u32(code, ctx.ret_temp_i32);

                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.ret_temp_i32);
                    code.push(OP_I32_CONST);
                    write_i32(code, elem_size);
                    code.push(OP_I32_MUL);
                    code.push(OP_I32_ADD); // this
                    code.push(OP_CALL);
                    write_u32(code, di);

                    code.push(OP_BR);
                    write_u32(code, 0);

                    code.push(OP_END);
                    code.push(OP_END);

                    // free(scratch1 - 4)
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                    code.push(OP_I32_CONST);
                    write_i32(code, 4);
                    code.push(OP_I32_SUB);
                    code.push(OP_CALL);
                    write_u32(code, self.func_indices["free"]);
                    return Ok(());
                }
            }
            // Fallback / POD array: free(ptr)
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_CALL);
            write_u32(code, self.func_indices["free"]);
            return Ok(());
        }

        if let Some(inner) = ty.deref_type()
            && let Type::Struct(sname) = inner.strip_const()
        {
            let layout = self.sema.struct_layouts.get(sname);
            let is_virtual_dtor =
                layout.and_then(|l| l.vtable_slots.iter().position(|s| s == "__deleting_dtor"));

            if let Some(slot_idx) = is_virtual_dtor {
                // Virtual deleting destructor dispatch via vtable slot
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, 0); // vptr
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, (slot_idx * 4) as u32); // table_idx
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch2);

                // Push this
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch2);

                code.push(OP_CALL_INDIRECT);
                write_u32(code, self.t_i32_v);
                code.push(0);

                return Ok(());
            } else {
                // Non-virtual destructor call
                let dtor_name = format!("{}_~{}", sname, sname);
                if let Some(&dtor_idx) = self.func_indices.get(&dtor_name) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1); // this
                    code.push(OP_CALL);
                    write_u32(code, dtor_idx);
                }
            }
        }

        // Call free
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_CALL);
        write_u32(code, self.func_indices["free"]);

        Ok(())
    }
}
