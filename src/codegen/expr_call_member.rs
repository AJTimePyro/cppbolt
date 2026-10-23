//! Lowering of member calls: `object.member(...)` and `ptr->member(...)`.

use super::call_types::{wasm_params_from_types, wasm_result_types};
use super::*;

impl<'a> Codegen<'a> {
    /// Lowers a member call.
    pub(super) fn compile_member_call(
        &mut self,
        access: MemberAccess<'_>,
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let MemberAccess {
            object,
            member,
            is_arrow,
        } = access;
        let arg_types: Vec<Type> = args.iter().map(|a| self.get_expr_type(a, ctx)).collect();
        let obj_ty = self.get_expr_type_raw(object, ctx);
        let is_ref = obj_ty.is_reference();
        let receiver_const = crate::sema::receiver_is_const_type(&obj_ty);
        let sname = self.member_class_name(&obj_ty, is_arrow || is_ref);

        let this_slot = ctx.alloc_frame(4, 4)?;
        let fn_slot = ctx.alloc_frame(4, 4)?;

        // Evaluate the object once and keep its address in a frame slot;
        // argument evaluation may clobber the scratch locals.
        self.emit_member_receiver(object, is_arrow || is_ref, this_slot, ctx, code)?;

        let layout = self.sema.struct_layouts.get(&sname);
        let is_virtual = layout.and_then(|l| l.vtable_slots.iter().position(|s| s == member));
        let site = CallSite {
            class_name: &sname,
            member,
            args,
            slot_idx: 0,
            this_slot,
            fn_slot,
            receiver_const,
        };

        if let Some(slot_idx) = is_virtual {
            self.compile_member_call_virtual(CallSite { slot_idx, ..site }, ctx, code)?;
        } else if let Some((m_idx, sig)) =
            self.find_method_for_receiver(&sname, member, &arg_types, receiver_const)
        {
            self.compile_member_call_direct(site, &sig, m_idx, ctx, code)?;
        } else {
            return Err(CompileError::new(
                format!("Method '{}' not found in class '{}'", member, sname),
                self.current_line,
                self.current_col,
            ));
        }
        Ok(())
    }

    /// Returns the name of the class the receiver belongs to.
    pub(super) fn member_class_name(&self, obj_ty: &Type, through_pointer: bool) -> String {
        if through_pointer {
            if let Type::Pointer(inner) | Type::Reference(inner) = obj_ty.strip_const()
                && let Type::Struct(s) = inner.strip_const()
            {
                return s.clone();
            }
            return String::new();
        }
        if let Type::Struct(s) = obj_ty.strip_const() {
            return s.clone();
        }
        String::new()
    }

    /// Evaluates the receiver and stores its address in a frame slot.
    fn emit_member_receiver(
        &mut self,
        object: &Expr,
        through_pointer: bool,
        this_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if through_pointer {
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
        Ok(())
    }

    /// Emits a virtual method call through the object's vtable.
    pub(super) fn compile_member_call_virtual(
        &mut self,
        site: CallSite<'_>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let CallSite {
            class_name: sname,
            member,
            args,
            slot_idx,
            this_slot,
            fn_slot,
            receiver_const,
        } = site;
        // Virtual dispatch via vtable
        self.load_vtable_slot(slot_idx, this_slot, fn_slot, ctx, code)?;

        let Some(sig) = self
            .sema
            .find_method_in_class_for_receiver(sname, member, receiver_const)
        else {
            return Err(CompileError::new(
                format!("Method '{}' not found in class '{}'", member, sname),
                self.current_line,
                self.current_col,
            ));
        };

        // The implicit `this` argument comes first.
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.fp_local);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, this_slot);
        self.emit_call_arguments(args, &sig.param_types, 1, ctx, code)?;
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

    /// Emits the arguments of a call, starting at `first_param`.
    fn emit_call_arguments(
        &mut self,
        args: &[Expr],
        param_types: &[Type],
        first_param: usize,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        for (i, a) in args.iter().enumerate() {
            let param_ty = param_types.get(i + first_param);
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
        Ok(())
    }

    /// Loads the function pointer of a virtual call from the vtable.
    fn load_vtable_slot(
        &mut self,
        slot_idx: usize,
        this_slot: u32,
        fn_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
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
        Ok(())
    }

    /// Emits a direct (non-virtual) method call.
    pub(super) fn compile_member_call_direct(
        &mut self,
        site: CallSite<'_>,
        sig: &FunctionSig,
        m_idx: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let CallSite {
            args, this_slot, ..
        } = site;
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
}
