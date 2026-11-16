use super::*;

pub(crate) struct MemberPointerCall<'a> {
    pub left: &'a Expr,
    pub right: &'a Expr,
    pub is_arrow: bool,
    pub args: &'a [Expr],
}

pub(crate) struct MemberPointerCmp<'a> {
    pub op: &'a BinaryOp,
    pub lty: &'a Type,
    pub rty: &'a Type,
    pub left: Option<&'a Expr>,
    pub right: &'a Expr,
}

struct PmfInvokeSite<'a> {
    this_slot: u32,
    fn_slot: u32,
    args: &'a [Expr],
    param_types: &'a [Type],
    ret_ty: &'a Type,
}

impl<'a> Codegen<'a> {
    pub(super) fn compile_member_pointer_lvalue(
        &mut self,
        left: &Expr,
        right: &Expr,
        is_arrow: bool,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if is_arrow {
            self.compile_expr(left, ctx, code)?;
        } else {
            self.compile_lvalue_addr(left, ctx, code)?;
        }
        self.compile_expr(right, ctx, code)?;
        code.push(OP_I32_ADD);
        Ok(())
    }

    /// Emits a member-function pointer for one exact overload.
    fn emit_member_function_pointer(
        &mut self,
        class_name: &str,
        member_name: &str,
        sig: &FunctionSig,
        code: &mut Vec<u8>,
    ) -> Result<bool, CompileError> {
        let layout = self.sema.struct_layouts.get(class_name);
        let vslot = layout.and_then(|l| l.vtable_slots.iter().position(|s| s == member_name));
        let ptr = if let Some(slot_idx) = vslot {
            ((slot_idx as i64) << 1) | 1
        } else {
            let table_idx = self
                .func_table_indices
                .get(&sig.mangled_name)
                .copied()
                .unwrap_or(0);
            (table_idx as i64) << 1
        };
        let mut adj = 0i64;
        if let Some(first_param) = sig.param_types.first()
            && let Some(this_struct) = first_param.strip_reference().deref_type()
            && let Type::Struct(decl_class) = this_struct.strip_const()
            && decl_class != class_name
            && let Some(off) = self.sema.base_subobject_offset(class_name, decl_class)
        {
            adj = off as i64;
        }
        let val = (adj << 32) | (ptr & 0xFFFFFFFF);
        code.push(OP_I64_CONST);
        write_i64(code, val);
        Ok(true)
    }

    pub(super) fn try_compile_member_pointer_addrof(
        &mut self,
        name: &str,
        code: &mut Vec<u8>,
    ) -> Result<bool, CompileError> {
        if let Some((class_raw, mangled)) = decode_member_pointer_name(name)
            && let Some(sig) = self.sema.functions.get(mangled).and_then(|v| v.first())
        {
            let class_part = class_raw.strip_prefix("::").unwrap_or(class_raw);
            let class_name = self.sema.resolve_alias_in_qualified_name(class_part);
            return self.emit_member_function_pointer(&class_name, &sig.name, sig, code);
        }

        let Some((class_raw, member_name)) = name.rsplit_once("::") else {
            return Ok(false);
        };
        let class_part = class_raw.strip_prefix("::").unwrap_or(class_raw);
        let class_name = self.sema.resolve_alias_in_qualified_name(class_part);

        if let Some(fld) = self.sema.find_field_in_class(&class_name, member_name) {
            code.push(OP_I32_CONST);
            write_i32(code, fld.offset as i32);
            return Ok(true);
        }

        if let Some(sig) = self.sema.find_method_in_class(&class_name, member_name)
            && !sig.is_static
        {
            return self.emit_member_function_pointer(&class_name, member_name, &sig, code);
        }

        Ok(false)
    }

    pub(super) fn compile_member_pointer_eq_ne(
        &mut self,
        cmp: MemberPointerCmp<'_>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let MemberPointerCmp {
            op,
            lty,
            rty,
            left,
            right,
        } = cmp;
        let mptr_ty = if matches!(lty.strip_const(), Type::MemberPointer { .. }) {
            lty.strip_const()
        } else {
            rty.strip_const()
        };
        let is_func = match mptr_ty {
            Type::MemberPointer { member_type, .. } => {
                matches!(member_type.strip_const(), Type::Function { .. })
            }
            _ => false,
        };

        if let Some(left) = left {
            self.compile_expr(left, ctx, code)?;
            Self::normalize_member_pointer_operand(lty, is_func, code);
        }
        self.compile_expr(right, ctx, code)?;
        Self::normalize_member_pointer_operand(rty, is_func, code);

        if is_func {
            code.push(if *op == BinaryOp::Eq {
                OP_I64_EQ
            } else {
                OP_I64_NE
            });
        } else {
            code.push(if *op == BinaryOp::Eq {
                OP_I32_EQ
            } else {
                OP_I32_NE
            });
        }
        Ok(())
    }

    fn normalize_member_pointer_operand(ty: &Type, is_func: bool, code: &mut Vec<u8>) {
        if matches!(ty.strip_const(), Type::Nullptr) {
            code.push(OP_DROP);
            if is_func {
                code.push(OP_I64_CONST);
                write_i64(code, 0);
            } else {
                code.push(OP_I32_CONST);
                write_i32(code, -1);
            }
        }
    }

    pub(super) fn compile_member_pointer_call(
        &mut self,
        call: MemberPointerCall<'_>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let MemberPointerCall {
            left,
            right,
            is_arrow,
            args,
        } = call;

        let right_ty = self.get_expr_type(right, ctx);
        let (param_types, ret_ty) = match right_ty.strip_const() {
            Type::MemberPointer { member_type, .. } => match member_type.strip_const() {
                Type::Function {
                    param_types,
                    return_type,
                } => (param_types.clone(), (**return_type).clone()),
                _ => (Vec::new(), Type::Int),
            },
            _ => (Vec::new(), Type::Int),
        };

        let this_slot = ctx.alloc_frame(4, 4)?;
        let fn_slot = ctx.alloc_frame(4, 4)?;

        self.emit_pmf_receiver(left, is_arrow, this_slot, ctx, code)?;
        self.emit_pmf_target(right, this_slot, fn_slot, ctx, code)?;
        self.emit_pmf_invocation(
            PmfInvokeSite {
                this_slot,
                fn_slot,
                args,
                param_types: &param_types,
                ret_ty: &ret_ty,
            },
            ctx,
            code,
        )?;

        Ok(())
    }

    fn emit_pmf_receiver(
        &mut self,
        left: &Expr,
        is_arrow: bool,
        this_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        if is_arrow {
            self.compile_expr(left, ctx, code)?;
        } else {
            self.compile_lvalue_addr(left, ctx, code)?;
        }
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, this_slot);
        Ok(())
    }

    fn emit_pmf_target(
        &mut self,
        right: &Expr,
        this_slot: u32,
        fn_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        self.compile_expr(right, ctx, code)?;
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch_i64);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, this_slot);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch_i64);
        code.push(OP_I64_CONST);
        write_i64(code, 32);
        code.push(OP_I64_SHR_S);
        code.push(OP_I32_WRAP_I64);
        code.push(OP_I32_ADD);
        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, this_slot);

        self.emit_pmf_resolve_fn_slot(this_slot, fn_slot, ctx, code);
        Ok(())
    }

    fn emit_pmf_resolve_fn_slot(
        &mut self,
        this_slot: u32,
        fn_slot: u32,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch_i64);
        code.push(OP_I32_WRAP_I64);
        code.push(OP_I32_CONST);
        write_i32(code, 1);
        code.push(OP_I32_AND);
        code.push(OP_IF);
        code.push(TYPE_I32);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, this_slot);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, 0);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch_i64);
        code.push(OP_I32_WRAP_I64);
        code.push(OP_I32_CONST);
        write_i32(code, 1);
        code.push(OP_I32_SHR_U);
        code.push(OP_I32_CONST);
        write_i32(code, 4);
        code.push(OP_I32_MUL);
        code.push(OP_I32_ADD);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, 0);

        code.push(OP_ELSE);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch_i64);
        code.push(OP_I32_WRAP_I64);
        code.push(OP_I32_CONST);
        write_i32(code, 1);
        code.push(OP_I32_SHR_U);
        code.push(OP_END);

        code.push(OP_I32_STORE);
        write_u32(code, 2);
        write_u32(code, fn_slot);
    }

    fn emit_pmf_invocation(
        &mut self,
        site: PmfInvokeSite<'_>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let PmfInvokeSite {
            this_slot,
            fn_slot,
            args,
            param_types,
            ret_ty,
        } = site;

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, this_slot);

        for (i, a) in args.iter().enumerate() {
            let aty = self.get_expr_type(a, ctx);
            let pty = param_types.get(i);
            if pty.map(|p| p.is_reference()).unwrap_or(false) {
                self.compile_lvalue_addr(a, ctx, code)?;
            } else {
                self.compile_expr(a, ctx, code)?;
                if let Some(pty) = pty {
                    self.emit_conversion(&aty, pty, code);
                }
            }
        }

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, fn_slot);

        let mut params_wasm = vec![TYPE_I32];
        for pty in param_types {
            params_wasm.push(call_types::wasm_value_type(pty));
        }
        let results_wasm = call_types::wasm_result_types(ret_ty);
        let type_idx = self.module.add_type(params_wasm, results_wasm);

        code.push(OP_CALL_INDIRECT);
        write_u32(code, type_idx);
        code.push(0);

        self.register_returned_struct_temp(ret_ty, ctx, code)?;
        Ok(())
    }
}
