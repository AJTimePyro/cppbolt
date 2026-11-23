use super::*;

impl<'a> Codegen<'a> {
    fn emit_if_cleanup_bit_set(idx: usize, ctx: &FunctionContext, code: &mut Vec<u8>) {
        code.push(OP_LOCAL_GET);
        write_u32(code, ctx.cleanup_mask_local);
        code.push(OP_I64_CONST);
        crate::wasm::write_i64(code, 1_i64 << idx);
        code.push(OP_I64_AND);
        code.push(OP_I64_EQZ);
        code.push(OP_I32_EQZ);
        code.push(OP_IF);
        code.push(0x40);
    }

    fn emit_destroy_and_free_slot(
        slot: u32,
        dtor_idx: Option<u32>,
        free_idx: Option<u32>,
        fp_local: u32,
        code: &mut Vec<u8>,
    ) {
        Self::emit_slot_load(slot, fp_local, code);
        code.push(OP_IF);
        code.push(0x40);
        if let Some(dtor) = dtor_idx {
            Self::emit_slot_load(slot, fp_local, code);
            code.push(OP_CALL);
            write_u32(code, dtor);
        }
        if let Some(free) = free_idx {
            Self::emit_slot_load(slot, fp_local, code);
            code.push(OP_CALL);
            write_u32(code, free);
        }
        code.push(OP_END);
    }

    pub(super) fn emit_clean_temporaries(&mut self, ctx: &mut FunctionContext, code: &mut Vec<u8>) {
        let temps = std::mem::take(&mut ctx.temp_objects);
        if !temps.is_empty() {
            let free_idx = self.func_indices.get("free").copied();
            for (sname, temp_slot, should_destroy, clean_idx) in temps.iter().rev() {
                let dtor_idx = if *should_destroy {
                    self.lookup_struct_dtor(sname)
                } else {
                    None
                };
                if let Some(idx) = clean_idx {
                    Self::emit_if_cleanup_bit_set(*idx, ctx, code);
                    self.emit_mark_cleanup_inactive(*idx, ctx, code);
                }
                Self::emit_destroy_and_free_slot(*temp_slot, dtor_idx, free_idx, ctx.fp_local, code);
                if clean_idx.is_some() {
                    code.push(OP_END);
                }
            }
        }
        let list_temps = std::mem::take(&mut ctx.initializer_list_temps);
        for (sname, offset) in list_temps.iter().rev() {
            if let Some(dtor_idx) = self.lookup_struct_dtor(sname) {
                self.emit_local_address(*offset, ctx, code);
                code.push(OP_CALL);
                write_u32(code, dtor_idx);
            }
        }
    }

    pub(super) fn emit_dtor_for_scope(
        &self,
        objs: &[(String, Type, u32, Option<usize>)],
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        for (_name, ty, offset, clean_idx) in objs.iter().rev() {
            if ty.is_reference()
                && let Some(inner) = ty.deref_type()
                && let Type::Struct(sname) = inner.strip_const()
                && let Some(dtor_idx) = self.lookup_struct_dtor(sname)
            {
                if let Some(idx) = clean_idx {
                    Self::emit_if_cleanup_bit_set(*idx, ctx, code);
                    self.emit_mark_cleanup_inactive(*idx, ctx, code);
                    self.emit_destroy_and_free_temp(*offset, dtor_idx, ctx, code);
                    code.push(OP_END);
                } else {
                    self.emit_destroy_and_free_temp(*offset, dtor_idx, ctx, code);
                }
            } else {
                if let Some(idx) = clean_idx {
                    self.emit_mark_cleanup_inactive(*idx, ctx, code);
                }
                if let Type::Struct(sname) = ty.strip_const()
                    && let Some(dtor_idx) = self.lookup_struct_dtor(sname)
                {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_I32_CONST);
                    write_i32(code, *offset as i32);
                    code.push(OP_I32_ADD); // this
                    code.push(OP_CALL);
                    write_u32(code, dtor_idx);
                }
            }
        }
    }

    pub(super) fn register_returned_struct_temp(
        &mut self,
        return_type: &Type,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if !return_type.is_reference()
            && let Type::Struct(sname) = return_type.strip_const()
        {
            let temp_slot = ctx.alloc_frame(4, 4)?;
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_I32_STORE);
            write_u32(code, 2);
            write_u32(code, temp_slot);
            let clean_idx = self.register_and_activate_temp_cleanup(sname, temp_slot, ctx, code)?;
            ctx.temp_objects
                .push((sname.clone(), temp_slot, true, clean_idx));
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
        }
        Ok(())
    }

    pub(super) fn emit_destroy_and_free_temp(
        &self,
        slot: u32,
        dtor_idx: u32,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        Self::emit_destroy_and_free_slot(
            slot,
            Some(dtor_idx),
            self.func_indices.get("free").copied(),
            ctx.fp_local,
            code,
        );
    }

    pub(super) fn register_and_activate_temp_cleanup(
        &mut self,
        sname: &str,
        temp_slot: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<Option<usize>, CompileError> {
        let Some(dtor_idx) = self.lookup_struct_dtor(sname) else {
            return Ok(None);
        };
        let idx = ctx.register_temp_cleanup(sname.to_string(), dtor_idx, temp_slot)?;
        self.emit_mark_cleanup_active(idx, ctx, code);
        Ok(Some(idx))
    }

    pub(super) fn emit_copy_struct(
        &self,
        sname: &str,
        target_offset: u32,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let size = self
            .sema
            .struct_layouts
            .get(sname)
            .map(|l| l.size)
            .ok_or_else(|| self.internal_error("missing struct layout"))?;
        emit_struct_byte_copy(
            size,
            target_offset,
            ByteCopyAddr::Local(ctx.fp_local),
            ByteCopyAddr::Local(ctx.scratch1),
            code,
        );
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ByteCopyAddr {
    Local(u32),
    Slot { fp_local: u32, slot: u32 },
}

impl ByteCopyAddr {
    pub fn emit(&self, code: &mut Vec<u8>) {
        match *self {
            ByteCopyAddr::Local(loc) => {
                code.push(OP_LOCAL_GET);
                write_u32(code, loc);
            }
            ByteCopyAddr::Slot { fp_local, slot } => {
                Codegen::emit_slot_load(slot, fp_local, code);
            }
        }
    }
}

/// Copies `size` bytes from `src` to `dst` with `dst_offset`.
pub(crate) fn emit_struct_byte_copy(
    size: usize,
    dst_offset: u32,
    dst: ByteCopyAddr,
    src: ByteCopyAddr,
    code: &mut Vec<u8>,
) {
    if size == 0 {
        return;
    }
    dst.emit(code);
    if dst_offset != 0 {
        code.push(OP_I32_CONST);
        write_i32(code, dst_offset as i32);
        code.push(OP_I32_ADD);
    }
    src.emit(code);
    code.push(OP_I32_CONST);
    write_i32(code, size as i32);
    // WebAssembly bulk memory: memory.copy (0xFC 0x0A 0x00 0x00)
    code.push(0xFC);
    code.push(0x0A);
    code.push(0x00);
    code.push(0x00);
}
