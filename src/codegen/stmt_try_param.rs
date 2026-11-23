//! Catch parameter binding and layout for exception handlers.

use super::*;
use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::runtime::opcodes::*;
use crate::wasm::{write_i32, write_u32};

impl<'a> Codegen<'a> {
    pub(super) fn bind_catch_param_dispatch(
        &mut self,
        name: &str,
        ty: &Type,
        handled_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if ty.strip_reference().is_pointer() {
            self.bind_catch_param_pointer(name, ty, handled_slot, ctx, code)
        } else {
            self.bind_catch_param_object(name, ty, handled_slot, ctx, code)
        }
    }

    fn bind_catch_param_pointer(
        &mut self,
        name: &str,
        ty: &Type,
        handled_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        Self::emit_slot_load(handled_slot, ctx.fp_local, code);
        code.push(OP_I32_LOAD);
        write_u32(code, 2);
        write_u32(code, 16);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch2);

        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_I32_GT_S);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch2);
        code.push(OP_I32_CONST);
        write_i32(code, 0);
        code.push(OP_I32_NE);
        code.push(OP_I32_AND);
        code.push(OP_IF);
        code.push(0x40);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch2);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_ADD);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch2);
        code.push(OP_END);

        if !name.is_empty() {
            if ty.is_reference() {
                let val_slot = ctx.alloc_frame(4, 4)?;
                Self::emit_slot_store(val_slot, ctx.fp_local, ctx.scratch2, code);

                let ref_slot = ctx.alloc_frame(4, 4)?;
                ctx.bind_param(name.to_string(), ref_slot, ty.clone());

                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_CONST);
                write_i32(code, val_slot as i32);
                code.push(OP_I32_ADD);
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, ref_slot);
            } else {
                let offset = ctx.alloc_frame(4, 4)?;
                ctx.bind_param(name.to_string(), offset, ty.clone());
                Self::emit_slot_store(offset, ctx.fp_local, ctx.scratch2, code);
            }
        }
        Ok(())
    }

    fn bind_catch_param_object(
        &mut self,
        name: &str,
        ty: &Type,
        handled_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let payload_addr_slot = ctx.alloc_frame(4, 4)?;
        Self::emit_slot_load(handled_slot, ctx.fp_local, code);
        code.push(OP_I32_CONST);
        write_i32(code, 16);
        code.push(OP_I32_ADD);
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.scratch1);
        code.push(OP_I32_ADD);
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch2);
        Self::emit_slot_store(payload_addr_slot, ctx.fp_local, ctx.scratch2, code);

        if !name.is_empty() || matches!(ty.strip_const(), Type::Struct(_)) {
            self.bind_catch_param(name, ty, payload_addr_slot, ctx, code)?;
        }
        Ok(())
    }

    fn bind_catch_param(
        &mut self,
        name: &str,
        ty: &Type,
        payload_addr_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if ty.is_reference() {
            if !name.is_empty() {
                let offset = ctx.alloc_frame(4, 4)?;
                ctx.bind_param(name.to_string(), offset, ty.clone());
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                Self::emit_slot_load(payload_addr_slot, ctx.fp_local, code);
                code.push(OP_I32_STORE);
                write_u32(code, 2);
                write_u32(code, offset);
            }
        } else if let Type::Struct(sname) = ty.strip_const() {
            let raw_size = self.sema.struct_sizes.get(sname).copied().unwrap_or(4);
            let size = u32::try_from(raw_size).unwrap_or(4);
            let align = u32::try_from(self.sema.struct_aligns.get(sname).copied().unwrap_or(4))
                .unwrap_or(4)
                .max(4);
            let offset = ctx.alloc_frame(size, align)?;
            if !name.is_empty() {
                ctx.bind_param(name.to_string(), offset, ty.clone());
            }

            let is_const = ty.is_const();
            if let Some(ctor_idx) = self.find_copy_constructor(sname, is_const) {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
                code.push(OP_I32_CONST);
                write_i32(code, offset as i32);
                code.push(OP_I32_ADD);
                Self::emit_slot_load(payload_addr_slot, ctx.fp_local, code);
                code.push(OP_CALL);
                write_u32(code, ctor_idx);
            } else {
                Self::emit_slot_load(payload_addr_slot, ctx.fp_local, code);
                code.push(OP_LOCAL_SET);
                write_u32(code, ctx.scratch1);
                raii::emit_struct_byte_copy(
                    raw_size,
                    offset,
                    raii::ByteCopyAddr::Local(ctx.fp_local),
                    raii::ByteCopyAddr::Local(ctx.scratch1),
                    code,
                );
            }
            let clean_idx = if let Some(dtor_idx) = self.lookup_struct_dtor(sname) {
                let idx = ctx.register_cleanup(sname.to_string(), dtor_idx, offset, false)?;
                self.emit_mark_cleanup_active(idx, ctx, code);
                Some(idx)
            } else {
                None
            };
            if let Some(scope) = ctx.scope_objects.last_mut() {
                scope.push((name.to_string(), ty.clone(), offset, clean_idx));
            }
        } else if !name.is_empty() {
            let raw_size = ty.size_of(&self.sema.struct_sizes);
            let size = u32::try_from(raw_size).unwrap_or(4);
            let align = u32::try_from(ty.align_of(&self.sema.struct_aligns))
                .unwrap_or(4)
                .max(4);
            let offset = ctx.alloc_frame(size, align)?;
            ctx.bind_local(name.to_string(), offset, ty.clone());

            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            Self::emit_slot_load(payload_addr_slot, ctx.fp_local, code);
            self.emit_load(ty, 0, code);
            self.emit_store(ty, offset, code);
        }
        Ok(())
    }
}
