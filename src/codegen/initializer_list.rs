//! Lowering of `std::initializer_list` values and their backing arrays.

use super::*;

impl<'a> Codegen<'a> {
    pub(super) fn compile_initializer_list_value(
        &mut self,
        ty: &Type,
        elements: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let list_slot = ctx.alloc_frame(8, 4)?;
        self.emit_initializer_list_storage(
            InitializerListStorage {
                ty,
                elements,
                list_offset: list_slot,
                lifetime: InitializerListLifetime::Temporary,
            },
            ctx,
            code,
        )?;
        self.emit_local_address(list_slot, ctx, code);
        Ok(())
    }

    pub(super) fn compile_initializer_list_local(
        &mut self,
        ty: &Type,
        elements: &[Expr],
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        self.emit_initializer_list_storage(
            InitializerListStorage {
                ty,
                elements,
                list_offset: offset,
                lifetime: InitializerListLifetime::Local,
            },
            ctx,
            code,
        )
    }

    fn emit_initializer_list_storage(
        &mut self,
        storage: InitializerListStorage<'_>,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Some(elem_ty) = self.sema.extract_initializer_list_element_type(storage.ty) else {
            return Err(self.internal_error("invalid initializer_list type"));
        };
        let array_offset =
            self.allocate_initializer_array(&elem_ty, storage.elements.len(), ctx)?;
        self.emit_initializer_elements(&elem_ty, storage.elements, array_offset, ctx, code)?;
        self.emit_initializer_list_fields(
            storage.list_offset,
            array_offset,
            storage.elements.len(),
            ctx,
            code,
        )?;
        self.register_initializer_elements(
            &elem_ty,
            storage.elements.len(),
            array_offset,
            ctx,
            storage.lifetime,
        );
        Ok(())
    }

    fn allocate_initializer_array(
        &self,
        elem_ty: &Type,
        count: usize,
        ctx: &mut FunctionContext,
    ) -> Result<u32, CompileError> {
        let size = elem_ty
            .size_of(&self.sema.struct_sizes)
            .checked_mul(count)
            .ok_or_else(|| self.internal_error("initializer_list backing array is too large"))?;
        let size = u32::try_from(size)
            .map_err(|_| self.internal_error("initializer_list backing array is too large"))?;
        let align = u32::try_from(elem_ty.align_of(&self.sema.struct_aligns))
            .unwrap_or(4)
            .max(4);
        ctx.alloc_frame(size, align)
    }

    fn emit_initializer_elements(
        &mut self,
        elem_ty: &Type,
        elements: &[Expr],
        array_offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
            .map_err(|_| self.internal_error("initializer_list element is too large"))?;
        for (index, element) in elements.iter().enumerate() {
            let index = u32::try_from(index)
                .map_err(|_| self.internal_error("initializer_list is too large"))?;
            let offset = index
                .checked_mul(elem_size)
                .and_then(|value| array_offset.checked_add(value))
                .ok_or_else(|| self.internal_error("initializer_list is too large"))?;
            self.compile_aggregate_local(elem_ty, element, offset, ctx, code)?;
        }
        Ok(())
    }

    fn emit_initializer_list_fields(
        &self,
        list_offset: u32,
        array_offset: u32,
        count: usize,
        ctx: &FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        self.emit_local_address(list_offset, ctx, code);
        if count == 0 {
            code.push(OP_I32_CONST);
            write_i32(code, 0);
        } else {
            self.emit_local_address(array_offset, ctx, code);
        }
        self.emit_store(&Type::Pointer(Box::new(Type::Int)), 0, code);
        self.emit_local_address(list_offset, ctx, code);
        code.push(OP_I32_CONST);
        let count = i32::try_from(count)
            .map_err(|_| self.internal_error("initializer_list is too large"))?;
        write_i32(code, count);
        self.emit_store(&Type::UnsignedInt, 4, code);
        Ok(())
    }

    fn register_initializer_elements(
        &self,
        elem_ty: &Type,
        count: usize,
        array_offset: u32,
        ctx: &mut FunctionContext,
        lifetime: InitializerListLifetime,
    ) {
        let Type::Struct(sname) = elem_ty.strip_const() else {
            return;
        };
        if self.lookup_struct_dtor(sname).is_none() {
            return;
        }
        let elem_size = elem_ty.size_of(&self.sema.struct_sizes);
        for index in 0..count {
            let Some(byte_index) = index.checked_mul(elem_size) else {
                return;
            };
            let Ok(byte_index) = u32::try_from(byte_index) else {
                return;
            };
            let Some(offset) = array_offset.checked_add(byte_index) else {
                return;
            };
            match lifetime {
                InitializerListLifetime::Local => {
                    if let Some(scope) = ctx.scope_objects.last_mut() {
                        scope.push((format!("{sname}[{index}]"), elem_ty.clone(), offset));
                    }
                }
                InitializerListLifetime::Temporary => {
                    ctx.initializer_list_temps.push((sname.clone(), offset));
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum InitializerListLifetime {
    Local,
    Temporary,
}

struct InitializerListStorage<'a> {
    ty: &'a Type,
    elements: &'a [Expr],
    list_offset: u32,
    lifetime: InitializerListLifetime,
}
