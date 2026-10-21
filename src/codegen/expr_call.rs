use super::call_types::{callee_return_type, wasm_result_types, wasm_value_type};
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
        let Expr::Call { callee, args, .. } = expr else {
            return Err(self.internal_error("unexpected expression kind"));
        };

        let mut arg_types = Vec::new();
        for a in args {
            arg_types.push(self.get_expr_type(a, ctx));
        }

        match &**callee {
            Expr::MemberAccess {
                object,
                member,
                is_arrow,
                ..
            } => self.compile_member_call(
                MemberAccess {
                    object,
                    member,
                    is_arrow: *is_arrow,
                },
                args,
                ctx,
                code,
            ),
            Expr::Identifier(name) => self.compile_named_call(
                NamedCall {
                    callee,
                    name,
                    class_name: None,
                },
                args,
                &arg_types,
                ctx,
                code,
            ),
            _ => self.compile_indirect_call(callee, args, ctx, code),
        }
    }

    fn compile_named_call(
        &mut self,
        call: NamedCall<'_>,
        args: &[Expr],
        arg_types: &[Type],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let NamedCall {
            callee,
            name,
            class_name,
        } = call;
        let math_opcode = match name {
            "sqrt_d" | "std::sqrt_d" | "sqrt" | "std::sqrt" => Some(OP_F64_SQRT),
            "floor_d" | "std::floor_d" | "floor" | "std::floor" => Some(OP_F64_FLOOR),
            "ceil_d" | "std::ceil_d" | "ceil" | "std::ceil" => Some(OP_F64_CEIL),
            "fabs_d" | "std::fabs_d" | "fabs" | "std::fabs" => Some(OP_F64_ABS),
            _ => None,
        };
        if let Some(opcode) = math_opcode {
            return self.compile_math_builtin(opcode, name, args, ctx, code);
        }
        if name == "printf" || name == "std::printf" || name.ends_with("::printf") {
            return self.compile_printf_call(args, ctx, code);
        }
        if name == "__builtin_wasm_memory_size" {
            code.push(OP_MEMORY_SIZE);
            code.push(0);
            return Ok(());
        }
        if self.sema.struct_layouts.contains_key(name) {
            return self.compile_struct_value_call(name, args, arg_types, ctx, code);
        }
        if ctx.var_offsets.contains_key(name) || self.global_offsets.contains_key(name) {
            return self.compile_function_pointer_call(callee, args, ctx, code);
        }
        if let Some(cls) = ctx.current_class.clone()
            && class_name.is_none()
        {
            return self.compile_in_class_call(
                NamedCall {
                    callee,
                    name,
                    class_name: Some(&cls),
                },
                args,
                arg_types,
                ctx,
                code,
            );
        }
        if let Some((func_idx, sig)) = self.find_function(name, arg_types) {
            for (i, a) in args.iter().enumerate() {
                let param_ty = sig.param_types.get(i);
                self.compile_call_arg(a, param_ty, ctx, code)?;
            }
            code.push(OP_CALL);
            write_u32(code, func_idx);
            self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
            return Ok(());
        }
        Err(CompileError::new(
            format!("Undefined function '{}'", name),
            self.current_line,
            self.current_col,
        ))
    }

    fn compile_math_builtin(
        &mut self,
        opcode: u8,
        name: &str,
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if args.len() != 1 {
            return Err(CompileError::new(
                format!("{} expects one argument", name),
                self.current_line,
                self.current_col,
            ));
        }
        self.compile_expr(&args[0], ctx, code)?;
        code.push(opcode);
        Ok(())
    }

    /// Call through a function-pointer variable. Build the call-site
    /// signature from the actual argument types.
    fn compile_function_pointer_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let callee_ty = self.get_expr_type(callee, ctx);
        let param_types = match callee_ty.strip_const() {
            Type::Pointer(inner) => match inner.strip_const() {
                Type::Function { param_types, .. } => Some(param_types.clone()),
                _ => None,
            },
            Type::Function { param_types, .. } => Some(param_types.clone()),
            _ => None,
        };

        let mut params_wasm = Vec::new();
        for (i, a) in args.iter().enumerate() {
            let aty = self.get_expr_type(a, ctx);
            let param_ty = param_types.as_ref().and_then(|params| params.get(i));
            if param_ty.map(|p| p.is_reference()).unwrap_or(false) {
                self.compile_lvalue_addr(a, ctx, code)?;
            } else {
                self.compile_expr(a, ctx, code)?;
                if let Some(pty) = param_ty {
                    self.emit_conversion(&aty, pty, code);
                }
            }
            params_wasm.push(wasm_value_type(param_ty.unwrap_or(&aty)));
        }
        self.compile_expr(callee, ctx, code)?;

        let ret_ty = callee_return_type(&callee_ty);
        let results_wasm = wasm_result_types(&ret_ty);

        let type_idx = self.module.add_type(params_wasm, results_wasm);
        code.push(OP_CALL_INDIRECT);
        write_u32(code, type_idx);
        code.push(0);
        Ok(())
    }

    fn compile_in_class_call(
        &mut self,
        call: NamedCall<'_>,
        args: &[Expr],
        arg_types: &[Type],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let NamedCall {
            name, class_name, ..
        } = call;
        let cls = class_name.unwrap_or("");
        if let Some((m_idx, sig)) = self.find_method(cls, name, arg_types) {
            code.push(OP_LOCAL_GET);
            write_u32(code, 0); // this
            for (i, a) in args.iter().enumerate() {
                let param_ty = sig.param_types.get(i + 1);
                self.compile_call_arg(a, param_ty, ctx, code)?;
            }
            code.push(OP_CALL);
            write_u32(code, m_idx);
            self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
            return Ok(());
        }
        if let Some((func_idx, sig)) = self.find_function(name, arg_types) {
            for (i, a) in args.iter().enumerate() {
                let param_ty = sig.param_types.get(i);
                self.compile_call_arg(a, param_ty, ctx, code)?;
            }
            code.push(OP_CALL);
            write_u32(code, func_idx);
            self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
            return Ok(());
        }
        Err(CompileError::new(
            format!("Undefined function '{}'", name),
            self.current_line,
            self.current_col,
        ))
    }

    fn compile_struct_value_call(
        &mut self,
        name: &str,
        args: &[Expr],
        arg_types: &[Type],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        // A struct-valued call such as `Item(args)` produces a prvalue
        // in a heap temporary; the caller materializes and cleans it up.
        let size = i32::try_from(self.sema.struct_layouts[name].size)
            .map_err(|_| self.internal_error("object is too large"))?;
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

        if let Some(ctor_idx) = self.find_constructor(name, arg_types) {
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

        ctx.temp_objects.push((name.to_string(), temp_slot, true));

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, temp_ptr_slot);
        Ok(())
    }

    fn compile_indirect_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        // General indirect call through an arbitrary callee expression.
        let callee_ty = self.get_expr_type(callee, ctx);
        let param_types = match callee_ty.strip_const() {
            Type::Pointer(inner) => match inner.strip_const() {
                Type::Function { param_types, .. } => Some(param_types.clone()),
                _ => None,
            },
            Type::Function { param_types, .. } => Some(param_types.clone()),
            _ => None,
        };

        let mut params_wasm = Vec::new();
        for (i, a) in args.iter().enumerate() {
            let aty = self.get_expr_type(a, ctx);
            let param_ty = param_types.as_ref().and_then(|params| params.get(i));
            if param_ty.map(|p| p.is_reference()).unwrap_or(false) {
                self.compile_lvalue_addr(a, ctx, code)?;
            } else {
                self.compile_expr(a, ctx, code)?;
                if let Some(pty) = param_ty {
                    self.emit_conversion(&aty, pty, code);
                }
            }
            params_wasm.push(wasm_value_type(param_ty.unwrap_or(&aty)));
        }
        if let Expr::Unary {
            op: UnaryOp::Deref,
            expr,
            ..
        } = callee
        {
            self.compile_expr(expr, ctx, code)?;
        } else {
            self.compile_expr(callee, ctx, code)?;
        }

        let ret_ty = callee_return_type(&callee_ty);
        let results_wasm = wasm_result_types(&ret_ty);

        let type_idx = self.module.add_type(params_wasm, results_wasm);
        code.push(OP_CALL_INDIRECT);
        write_u32(code, type_idx);
        code.push(0);
        Ok(())
    }
}
