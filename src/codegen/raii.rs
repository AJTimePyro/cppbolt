use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn emit_clean_temporaries(&mut self, ctx: &mut FunctionContext, code: &mut Vec<u8>) {
        let temps = std::mem::take(&mut ctx.temp_objects);
        for (sname, temp_slot, should_destroy) in temps.iter().rev() {
            if *should_destroy {
                let dtor_name = format!("{}_~{}", sname, sname);
                if let Some(&dtor_idx) = self.func_indices.get(&dtor_name) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.fp_local);
                    code.push(OP_I32_LOAD);
                    write_u32(code, 2);
                    write_u32(code, *temp_slot);
                    code.push(OP_CALL);
                    write_u32(code, dtor_idx);
                }
            }
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.fp_local);
            code.push(OP_I32_LOAD);
            write_u32(code, 2);
            write_u32(code, *temp_slot);
            code.push(OP_CALL);
            write_u32(code, self.func_indices["free"]);
        }
    }

    pub(super) fn emit_dtor_for_scope(
        &self,
        objs: &[(String, Type, u32)],
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) {
        for (_name, ty, offset) in objs.iter().rev() {
            if let Type::Struct(sname) = ty.strip_const() {
                let dtor_name = format!("{}_~{}", sname, sname);
                if let Some(&dtor_idx) = self.func_indices.get(&dtor_name) {
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
            |code| {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.fp_local);
            },
            |code| {
                code.push(OP_LOCAL_GET);
                write_u32(code, ctx.scratch1);
            },
            code,
        );
        Ok(())
    }
}

/// Copies `size` bytes from the address produced by `emit_src` to the address
/// produced by `emit_dst`.
pub(super) fn emit_struct_byte_copy(
    size: usize,
    dst_offset: u32,
    mut emit_dst: impl FnMut(&mut Vec<u8>),
    mut emit_src: impl FnMut(&mut Vec<u8>),
    code: &mut Vec<u8>,
) {
    for i in 0..size {
        emit_dst(code);
        emit_src(code);
        code.push(OP_I32_LOAD8_U);
        write_u32(code, 0);
        write_u32(code, i as u32);
        code.push(OP_I32_STORE8);
        write_u32(code, 0);
        write_u32(code, dst_offset + i as u32);
    }
}
