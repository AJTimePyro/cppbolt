use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn emit_clean_temporaries(&mut self, ctx: &mut FunctionContext, code: &mut Vec<u8>) {
        let temps = std::mem::take(&mut ctx.temp_objects);
        for (sname, temp_slot, should_destroy) in temps.iter().rev() {
            if *should_destroy && let Some(dtor_idx) = self.lookup_struct_dtor(sname) {
                Self::emit_slot_load(*temp_slot, ctx.fp_local, code);
                code.push(OP_CALL);
                write_u32(code, dtor_idx);
            }
            Self::emit_slot_load(*temp_slot, ctx.fp_local, code);
            code.push(OP_CALL);
            write_u32(code, self.func_indices["free"]);
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
        objs: &[(String, Type, u32)],
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        for (_name, ty, offset) in objs.iter().rev() {
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
            ctx.temp_objects.push((sname.clone(), temp_slot, true));
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
        }
        Ok(())
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
    for i in 0..size {
        dst.emit(code);
        src.emit(code);
        code.push(OP_I32_LOAD8_U);
        write_u32(code, 0);
        write_u32(code, i as u32);
        code.push(OP_I32_STORE8);
        write_u32(code, 0);
        write_u32(code, dst_offset + i as u32);
    }
}
