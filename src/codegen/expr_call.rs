use super::*;

impl<'a> Codegen<'a> {
    /// Lowers direct, virtual, and indirect calls, plus `printf`, math
    /// builtins, and prvalue constructor calls.
    pub(super) fn compile_call(
        &mut self,
        expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Expr::Call { callee, args } = expr else {
            return Err(crate::internal_error!(0, 0, "unexpected expression kind"));
        };

        let mut arg_types = Vec::new();
        for a in args {
            arg_types.push(self.get_expr_type(a, ctx));
        }

        if let Expr::MemberAccess {
            object,
            member,
            is_arrow,
        } = &**callee
        {
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

            let this_slot = ctx.alloc_frame(4, 4)?;
            let fn_slot = ctx.alloc_frame(4, 4)?;

            // Evaluate the object once and keep its address in a frame slot;
            // argument evaluation may clobber the scratch locals.
            if *is_arrow || is_ref {
                self.compile_expr(object, ctx, code)?;
            } else {
                self.compile_lvalue_addr(object, ctx, code)?;
            }
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1); // this_ptr
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_I32_STORE);
            write_u32(code, 2);
            write_u32(code, this_slot);

            let layout = self.sema.struct_layouts.get(&sname);
            let is_virtual = layout.and_then(|l| l.vtable_slots.iter().position(|s| s == member));

            if let Some(slot_idx) = is_virtual {
                // Virtual dispatch via vtable
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, this_slot);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, 0); // vptr
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, (slot_idx * 4) as u32); // table_idx
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch2);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch2);
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, fn_slot);

                let Some(sig) = self.sema.find_method_in_class(&sname, member) else {
                    return Err(CompileError::new(
                        format!("Method '{}' not found in class '{}'", member, sname),
                        1,
                        1,
                    ));
                };

                // The implicit `this` argument comes first.
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, this_slot);
                for (i, a) in args.iter().enumerate() {
                    let param_ty = sig.param_types.get(i + 1);
                    let is_r = param_ty.map(|p| p.is_reference()).unwrap_or(false);
                    if is_r {
                        self.compile_lvalue_addr(a, ctx, code)?;
                    } else {
                        let aty = self.get_expr_type(a, ctx);
                        self.compile_expr(a, ctx, code)?;
                        if let Some(pty) = param_ty {
                            self.emit_conversion(&aty, pty, code);
                        }
                    }
                }
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, fn_slot);

                // The indirect call site needs the callee's exact Wasm
                // signature type, including the implicit `this` parameter.
                let params_wasm: Vec<u8> = sig
                    .param_types
                    .iter()
                    .map(|p| {
                        p.to_wasm_type()
                            .map(|w| match w {
                                WasmType::I32 => TYPE_I32,
                                WasmType::I64 => TYPE_I64,
                                WasmType::F32 => TYPE_F32,
                                WasmType::F64 => TYPE_F64,
                            })
                            .unwrap_or(TYPE_I32)
                    })
                    .collect();
                let results_wasm = sig
                    .return_type
                    .to_wasm_type()
                    .map(|w| match w {
                        WasmType::I32 => vec![TYPE_I32],
                        WasmType::I64 => vec![TYPE_I64],
                        WasmType::F32 => vec![TYPE_F32],
                        WasmType::F64 => vec![TYPE_F64],
                    })
                    .unwrap_or_default();
                let type_idx = self.module.add_type(params_wasm, results_wasm);

                code.push(OP_CALL_INDIRECT);
                write_u32(code, type_idx);
                code.push(0);
                self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
            } else if let Some((m_idx, sig)) = self.find_method(&sname, member, &arg_types) {
                if !sig.is_static {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_I32_LOAD);
                    write_u32(code, 2);
                    write_u32(code, this_slot);
                }
                for (i, a) in args.iter().enumerate() {
                    let param_index = if sig.is_static { i } else { i + 1 };
                    let param_ty = sig.param_types.get(param_index);
                    let is_r = param_ty.map(|p| p.is_reference()).unwrap_or(false);
                    if is_r {
                        self.compile_lvalue_addr(a, ctx, code)?;
                    } else {
                        let aty = self.get_expr_type(a, ctx);
                        self.compile_expr(a, ctx, code)?;
                        if let Some(pty) = param_ty {
                            self.emit_conversion(&aty, pty, code);
                        }
                    }
                }
                code.push(OP_CALL);
                write_u32(code, m_idx);
                self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
            } else {
                return Err(CompileError::new(
                    format!("Method '{}' not found in class '{}'", member, sname),
                    1,
                    1,
                ));
            }
        } else if let Expr::Identifier(name) = &**callee {
            let math_opcode = match name.as_str() {
                "sqrt_d" | "std::sqrt_d" | "sqrt" | "std::sqrt" => Some(OP_F64_SQRT),
                "floor_d" | "std::floor_d" | "floor" | "std::floor" => Some(OP_F64_FLOOR),
                "ceil_d" | "std::ceil_d" | "ceil" | "std::ceil" => Some(OP_F64_CEIL),
                "fabs_d" | "std::fabs_d" | "fabs" | "std::fabs" => Some(OP_F64_ABS),
                _ => None,
            };
            if let Some(opcode) = math_opcode {
                if args.len() != 1 {
                    return Err(CompileError::new(
                        format!("{} expects one argument", name),
                        1,
                        1,
                    ));
                }
                self.compile_expr(&args[0], ctx, code)?;
                code.push(opcode);
            } else if name == "printf" || name == "std::printf" || name.ends_with("::printf") {
                // Variadic arguments are laid out by value in a compact frame
                // block. Floats are promoted to double, matching the runtime's
                // 4/8-byte reader.
                let mut vararg_bytes = 0u32;
                for a in args.iter().skip(1) {
                    let aty = self.get_expr_type(a, ctx);
                    let size = match aty.strip_const().strip_reference() {
                        Type::Float | Type::Double | Type::LongLong | Type::UnsignedLongLong => 8,
                        _ => 4,
                    };
                    vararg_bytes = vararg_bytes.checked_add(size).ok_or_else(|| {
                        crate::internal_error!(0, 0, "printf arguments are too large")
                    })?;
                }
                let varargs_base = ctx.alloc_frame(vararg_bytes, 8)?;
                let mut arg_offset = varargs_base;

                for a in args.iter().skip(1) {
                    let aty = self.get_expr_type(a, ctx);
                    let clean_aty = aty.strip_const().strip_reference();
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    self.compile_expr(a, ctx, code)?;

                    if matches!(clean_aty, Type::Float) {
                        code.push(OP_F64_PROMOTE_F32);
                        code.push(OP_F64_STORE);
                        write_u32(code, 3);
                        write_u32(code, arg_offset);
                        arg_offset += 8;
                    } else if matches!(clean_aty, Type::Double) {
                        code.push(OP_F64_STORE);
                        write_u32(code, 3);
                        write_u32(code, arg_offset);
                        arg_offset += 8;
                    } else if matches!(clean_aty, Type::LongLong | Type::UnsignedLongLong) {
                        code.push(OP_I64_STORE);
                        write_u32(code, 3);
                        write_u32(code, arg_offset);
                        arg_offset += 8;
                    } else {
                        code.push(OP_I32_STORE);
                        write_u32(code, 2);
                        write_u32(code, arg_offset);
                        arg_offset += 4;
                    }
                }

                self.compile_expr(&args[0], ctx, code)?;
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_CONST);
                write_i32(code, varargs_base as i32);
                code.push(OP_I32_ADD);

                code.push(OP_CALL);
                write_u32(code, self.func_printf);
            } else if name == "__builtin_wasm_memory_size" {
                code.push(OP_MEMORY_SIZE);
                code.push(0);
                return Ok(());
            } else if self.sema.struct_layouts.contains_key(name) {
                // A struct-valued call such as `Item(args)` produces a prvalue
                // in a heap temporary; the caller materializes and cleans it up.
                let size = i32::try_from(self.sema.struct_layouts[name].size)
                    .map_err(|_| crate::internal_error!(0, 0, "object is too large"))?;
                let temp_ptr_slot = ctx.alloc_frame(4, 4)?;
                code.push(OP_I32_CONST);
                write_i32(code, size);
                code.push(OP_CALL);
                write_u32(code, self.func_indices["malloc"]);
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1); // temp_ptr
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, temp_ptr_slot);

                if let Some(&v_off) = self.vtable_offsets.get(name) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_I32_LOAD);
                    write_u32(code, 2);
                    write_u32(code, temp_ptr_slot);
                    code.push(OP_I32_CONST);
                    write_i32(code, v_off as i32);
                    code.push(OP_I32_STORE);
                    write_u32(code, 2);
                    write_u32(code, 0);
                }

                if let Some(ctor_idx) = self.find_constructor(name, &arg_types) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_I32_LOAD);
                    write_u32(code, 2);
                    write_u32(code, temp_ptr_slot); // this
                    for a in args {
                        self.compile_expr(a, ctx, code)?;
                    }
                    code.push(OP_CALL);
                    write_u32(code, ctor_idx);
                }

                // Save the temporary pointer in a frame slot so statement
                // cleanup can destroy and free it after the full expression.
                let temp_slot = ctx.alloc_frame(4, 4)?;
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, temp_ptr_slot);
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, temp_slot);

                ctx.temp_objects.push((name.clone(), temp_slot, true));

                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_LOAD);
                write_u32(code, 2);
                write_u32(code, temp_ptr_slot);
            } else if ctx.var_offsets.contains_key(name) || self.global_offsets.contains_key(name) {
                // Call through a function-pointer variable. Build the call-site
                // signature from the actual argument types.
                let mut params_wasm = Vec::new();
                for a in args {
                    let aty = self.get_expr_type(a, ctx);
                    self.compile_expr(a, ctx, code)?;
                    params_wasm.push(
                        aty.to_wasm_type()
                            .map(|w| match w {
                                WasmType::I32 => TYPE_I32,
                                WasmType::I64 => TYPE_I64,
                                WasmType::F32 => TYPE_F32,
                                WasmType::F64 => TYPE_F64,
                            })
                            .unwrap_or(TYPE_I32),
                    );
                }
                self.compile_expr(callee, ctx, code)?;

                let callee_ty = self.get_expr_type(callee, ctx);
                let ret_ty = if let Type::Pointer(inner) = callee_ty.strip_const() {
                    if let Type::Function { return_type, .. } = &**inner {
                        (**return_type).clone()
                    } else {
                        Type::Int
                    }
                } else if let Type::Function { return_type, .. } = callee_ty.strip_const() {
                    (**return_type).clone()
                } else {
                    Type::Int
                };

                let results_wasm = ret_ty
                    .to_wasm_type()
                    .map(|w| match w {
                        WasmType::I32 => vec![TYPE_I32],
                        WasmType::I64 => vec![TYPE_I64],
                        WasmType::F32 => vec![TYPE_F32],
                        WasmType::F64 => vec![TYPE_F64],
                    })
                    .unwrap_or_default();

                let type_idx = self.module.add_type(params_wasm, results_wasm);
                code.push(OP_CALL_INDIRECT);
                write_u32(code, type_idx);
                code.push(0);
            } else if let Some(cls) = &ctx.current_class.clone() {
                if let Some((m_idx, sig)) = self.find_method(cls, name, &arg_types) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, 0); // this
                    for (i, a) in args.iter().enumerate() {
                        let param_ty = sig.param_types.get(i + 1);
                        self.compile_call_arg(a, param_ty, ctx, code)?;
                    }
                    code.push(OP_CALL);
                    write_u32(code, m_idx);
                    self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
                } else if let Some((func_idx, sig)) = self.find_function(name, &arg_types) {
                    for (i, a) in args.iter().enumerate() {
                        let param_ty = sig.param_types.get(i);
                        self.compile_call_arg(a, param_ty, ctx, code)?;
                    }
                    code.push(OP_CALL);
                    write_u32(code, func_idx);
                    self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
                } else {
                    return Err(CompileError::new(
                        format!("Undefined function '{}'", name),
                        1,
                        1,
                    ));
                }
            } else if let Some((func_idx, sig)) = self.find_function(name, &arg_types) {
                for (i, a) in args.iter().enumerate() {
                    let param_ty = sig.param_types.get(i);
                    self.compile_call_arg(a, param_ty, ctx, code)?;
                }
                code.push(OP_CALL);
                write_u32(code, func_idx);
                self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
            } else {
                return Err(CompileError::new(
                    format!("Undefined function '{}'", name),
                    1,
                    1,
                ));
            }
        } else {
            // General indirect call through an arbitrary callee expression.
            let mut params_wasm = Vec::new();
            for a in args {
                let aty = self.get_expr_type(a, ctx);
                self.compile_expr(a, ctx, code)?;
                params_wasm.push(
                    aty.to_wasm_type()
                        .map(|w| match w {
                            WasmType::I32 => TYPE_I32,
                            WasmType::I64 => TYPE_I64,
                            WasmType::F32 => TYPE_F32,
                            WasmType::F64 => TYPE_F64,
                        })
                        .unwrap_or(TYPE_I32),
                );
            }
            if let Expr::Unary {
                op: UnaryOp::Deref,
                expr,
            } = &**callee
            {
                self.compile_expr(expr, ctx, code)?;
            } else {
                self.compile_expr(callee, ctx, code)?;
            }

            let callee_ty = self.get_expr_type(callee, ctx);
            let ret_ty = if let Type::Pointer(inner) = callee_ty.strip_const() {
                if let Type::Function { return_type, .. } = &**inner {
                    (**return_type).clone()
                } else {
                    Type::Int
                }
            } else if let Type::Function { return_type, .. } = callee_ty.strip_const() {
                (**return_type).clone()
            } else {
                Type::Int
            };

            let results_wasm = ret_ty
                .to_wasm_type()
                .map(|w| match w {
                    WasmType::I32 => vec![TYPE_I32],
                    WasmType::I64 => vec![TYPE_I64],
                    WasmType::F32 => vec![TYPE_F32],
                    WasmType::F64 => vec![TYPE_F64],
                })
                .unwrap_or_default();

            let type_idx = self.module.add_type(params_wasm, results_wasm);
            code.push(OP_CALL_INDIRECT);
            write_u32(code, type_idx);
            code.push(0);
        }

        Ok(())
    }
}
