use super::call_types::{
    callee_return_type, wasm_params_from_types, wasm_result_types, wasm_value_type,
};
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

        match &**callee {
            Expr::MemberAccess {
                object,
                member,
                is_arrow,
            } => self.compile_member_call(object, member, *is_arrow, args, ctx, code),
            Expr::Identifier(name) => {
                self.compile_named_call(callee, name, args, &arg_types, ctx, code)
            }
            _ => self.compile_indirect_call(callee, args, ctx, code),
        }
    }

    fn compile_member_call(
        &mut self,
        object: &Expr,
        member: &str,
        is_arrow: bool,
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let arg_types: Vec<Type> = args.iter().map(|a| self.get_expr_type(a, ctx)).collect();
        let obj_ty = self.get_expr_type_raw(object, ctx);
        let is_ref = obj_ty.is_reference();
        let sname = if is_arrow || is_ref {
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
        if is_arrow || is_ref {
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
            self.compile_member_call_virtual(
                &sname, member, args, slot_idx, this_slot, fn_slot, ctx, code,
            )?;
        } else if let Some((m_idx, sig)) = self.find_method(&sname, member, &arg_types) {
            self.compile_member_call_direct(
                &sname, member, args, &sig, m_idx, this_slot, ctx, code,
            )?;
        } else {
            return Err(CompileError::new(
                format!("Method '{}' not found in class '{}'", member, sname),
                1,
                1,
            ));
        }
        Ok(())
    }

    /// Emits a virtual method call through the object's vtable.
    #[allow(clippy::too_many_arguments)]
    fn compile_member_call_virtual(
        &mut self,
        sname: &str,
        member: &str,
        args: &[Expr],
        slot_idx: usize,
        this_slot: u32,
        fn_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
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

        let Some(sig) = self.sema.find_method_in_class(sname, member) else {
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

        // The indirect call site needs the callee's exact Wasm signature
        // type, including the implicit `this` parameter.
        let params_wasm = wasm_params_from_types(&sig.param_types);
        let results_wasm = wasm_result_types(&sig.return_type);
        let type_idx = self.module.add_type(params_wasm, results_wasm);

        code.push(OP_CALL_INDIRECT);
        write_u32(code, type_idx);
        code.push(0);
        self.register_returned_struct_temp(&sig.return_type, ctx, code)?;
        Ok(())
    }

    /// Emits a direct (non-virtual) method call.
    #[allow(clippy::too_many_arguments)]
    fn compile_member_call_direct(
        &mut self,
        _sname: &str,
        _member: &str,
        args: &[Expr],
        sig: &FunctionSig,
        m_idx: u32,
        this_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
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
        Ok(())
    }

    fn compile_named_call(
        &mut self,
        callee: &Expr,
        name: &str,
        args: &[Expr],
        arg_types: &[Type],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
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
        if let Some(cls) = ctx.current_class.clone() {
            return self.compile_in_class_call(&cls, name, args, arg_types, ctx, code);
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
            1,
            1,
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
                1,
                1,
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
        let mut params_wasm = Vec::new();
        for a in args {
            let aty = self.get_expr_type(a, ctx);
            self.compile_expr(a, ctx, code)?;
            params_wasm.push(wasm_value_type(&aty));
        }
        self.compile_expr(callee, ctx, code)?;

        let callee_ty = self.get_expr_type(callee, ctx);
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
        cls: &str,
        name: &str,
        args: &[Expr],
        arg_types: &[Type],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
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
            1,
            1,
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
        let mut params_wasm = Vec::new();
        for a in args {
            let aty = self.get_expr_type(a, ctx);
            self.compile_expr(a, ctx, code)?;
            params_wasm.push(wasm_value_type(&aty));
        }
        if let Expr::Unary {
            op: UnaryOp::Deref,
            expr,
        } = callee
        {
            self.compile_expr(expr, ctx, code)?;
        } else {
            self.compile_expr(callee, ctx, code)?;
        }

        let callee_ty = self.get_expr_type(callee, ctx);
        let ret_ty = callee_return_type(&callee_ty);
        let results_wasm = wasm_result_types(&ret_ty);

        let type_idx = self.module.add_type(params_wasm, results_wasm);
        code.push(OP_CALL_INDIRECT);
        write_u32(code, type_idx);
        code.push(0);
        Ok(())
    }
}
