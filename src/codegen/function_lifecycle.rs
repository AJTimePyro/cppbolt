//! Constructor and destructor lowering for functions.

use super::*;
use crate::sema::StructLayout;

impl<'a> Codegen<'a> {
    pub(super) fn compile_constructor_prologue(
        &mut self,
        f: &FunctionDecl,
        cls: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let short_cls = cls.split("::").last().unwrap_or(cls);
        if f.name == *cls || f.name == short_cls {
            self.compile_constructor_body(f, cls, ctx, code)?;
        } else if f.name == format!("~{}", cls) || f.name == format!("~{}", short_cls) {
            // Destructor: reinstall this class's vtable in case derived overwrote it
            self.install_class_vtable(cls, code);
        }
        Ok(())
    }

    /// Emits the body of a constructor: base constructor calls, the class
    /// vtable, and the member initializers.
    pub(super) fn compile_constructor_body(
        &mut self,
        f: &FunctionDecl,
        cls: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let short_cls = cls.split("::").last().unwrap_or(cls);
        if let Some((_, target_args)) = f
            .ctor_initializers
            .iter()
            .find(|(name, _)| name == cls || name == short_cls)
        {
            let arg_types: Vec<Type> = target_args
                .iter()
                .map(|a| self.get_expr_type(a, ctx))
                .collect();
            let Some((ctor_idx, sig)) = self.find_constructor_sig(cls, &arg_types) else {
                return Err(CompileError::new(
                    format!("no matching target constructor found in class '{cls}'"),
                    f.line,
                    f.col,
                ));
            };
            code.push(OP_LOCAL_GET);
            write_u32(code, 0); // this
            let full_args = self.constructor_arguments(&sig, target_args);
            for (i, a) in full_args.iter().enumerate() {
                let param_ty = sig.param_types.get(i + 1);
                self.compile_call_arg(a, param_ty, ctx, code)?;
            }
            code.push(OP_CALL);
            write_u32(code, ctor_idx);
            return Ok(());
        }

        self.compile_base_constructor_calls(f, cls, ctx, code)?;
        self.install_class_vtable(cls, code);
        let Some(layout) = self.sema.struct_layouts.get(cls) else {
            return Ok(());
        };
        self.compile_member_initializers(f, layout, ctx, code)
    }

    /// Installs the class's vtable pointer in the object under construction.
    pub(super) fn install_class_vtable(&mut self, cls: &str, code: &mut Vec<u8>) {
        if let Some(&v_off) = self.vtable_offsets.get(cls) {
            code.push(OP_LOCAL_GET);
            write_u32(code, 0); // this
            code.push(OP_I32_CONST);
            write_i32(code, v_off as i32);
            code.push(OP_I32_STORE);
            write_u32(code, 2);
            write_u32(code, 0);
        }
    }

    /// Calls the constructor of every base subobject.
    pub(super) fn compile_base_constructor_calls(
        &mut self,
        f: &FunctionDecl,
        cls: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let Some(layout) = self.sema.struct_layouts.get(cls) else {
            return Ok(());
        };
        for (base_name, base_offset) in &layout.base_offsets {
            let base_args = f
                .ctor_initializers
                .iter()
                .find(|(name, _)| name == base_name)
                .map(|(_, args)| args.as_slice())
                .unwrap_or(&[]);
            let arg_types: Vec<Type> = base_args
                .iter()
                .map(|a| self.get_expr_type(a, ctx))
                .collect();
            if let Some((b_idx, sig)) = self.find_constructor_sig(base_name, &arg_types) {
                code.push(OP_LOCAL_GET);
                write_u32(code, 0); // this
                if *base_offset != 0 {
                    code.push(OP_I32_CONST);
                    write_i32(code, *base_offset as i32);
                    code.push(OP_I32_ADD);
                }
                let full_args = self.constructor_arguments(&sig, base_args);
                for (i, a) in full_args.iter().enumerate() {
                    let param_ty = sig.param_types.get(i + 1);
                    self.compile_call_arg(a, param_ty, ctx, code)?;
                }
                code.push(OP_CALL);
                write_u32(code, b_idx);
            } else if let Some(base_layout) = self.sema.struct_layouts.get(base_name).cloned() {
                for (j, a) in base_args.iter().enumerate() {
                    if let Some(sub_fld) = base_layout.fields.get(j) {
                        code.push(OP_LOCAL_GET);
                        write_u32(code, 0); // this
                        let a_ty = self.get_expr_type(a, ctx);
                        self.compile_expr(a, ctx, code)?;
                        self.emit_conversion(&a_ty, &sub_fld.ty, code);
                        self.emit_store(&sub_fld.ty, (*base_offset + sub_fld.offset) as u32, code);
                    }
                }
            }
        }
        Ok(())
    }

    fn base_field_count(&self, layout: &StructLayout) -> usize {
        layout
            .base_classes
            .iter()
            .filter_map(|b| self.sema.struct_layouts.get(b))
            .map(|bl| bl.fields.len())
            .sum()
    }

    /// Emits the member initializers in declaration order. Base subobjects
    /// are initialized by their own constructors.
    pub(super) fn compile_member_initializers(
        &mut self,
        f: &FunctionDecl,
        layout: &StructLayout,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let base_field_count = self.base_field_count(layout);
        for fld in layout.fields.iter().skip(base_field_count) {
            let init_args = f
                .ctor_initializers
                .iter()
                .find(|(name, _)| name == &fld.name)
                .map(|(_, args)| args.as_slice());
            if let Type::Struct(sname) = fld.ty.strip_const() {
                let arg_types: Vec<Type> = init_args
                    .map(|args| args.iter().map(|a| self.get_expr_type(a, ctx)).collect())
                    .unwrap_or_default();
                if let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, 0); // this
                    code.push(OP_I32_CONST);
                    write_i32(code, fld.offset as i32);
                    code.push(OP_I32_ADD);
                    if let Some(args) = init_args {
                        let full_args = self.constructor_arguments(&sig, args);
                        for (i, a) in full_args.iter().enumerate() {
                            let param_ty = sig.param_types.get(i + 1);
                            self.compile_call_arg(a, param_ty, ctx, code)?;
                        }
                    }
                    code.push(OP_CALL);
                    write_u32(code, ctor_idx);
                } else if let Some(layout) = self.sema.struct_layouts.get(sname).cloned()
                    && let Some(args) = init_args
                {
                    for (j, a) in args.iter().enumerate() {
                        if let Some(sub_fld) = layout.fields.get(j) {
                            code.push(OP_LOCAL_GET);
                            write_u32(code, 0); // this
                            let a_ty = self.get_expr_type(a, ctx);
                            self.compile_expr(a, ctx, code)?;
                            self.emit_conversion(&a_ty, &sub_fld.ty, code);
                            self.emit_store(
                                &sub_fld.ty,
                                (fld.offset + sub_fld.offset) as u32,
                                code,
                            );
                        }
                    }
                }
            } else if let Type::Array(elem_ty, count) = fld.ty.strip_const()
                && let Type::Struct(sname) = elem_ty.strip_const()
                && init_args.is_none_or(|a| a.is_empty())
                && let Some(ctor_idx) = self.find_constructor(sname, &[])
            {
                let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
                    .map_err(|_| crate::internal_error!(f.line, f.col, "member is too large"))?;
                for i in 0..*count {
                    let member_offset = (fld.offset as u32)
                        .checked_add((i as u32).checked_mul(elem_size).ok_or_else(|| {
                            crate::internal_error!(f.line, f.col, "member is too large")
                        })?)
                        .ok_or_else(|| {
                            crate::internal_error!(f.line, f.col, "member is too large")
                        })?;
                    code.push(OP_LOCAL_GET);
                    write_u32(code, 0); // this
                    code.push(OP_I32_CONST);
                    write_i32(code, member_offset as i32);
                    code.push(OP_I32_ADD);
                    code.push(OP_CALL);
                    write_u32(code, ctor_idx);
                }
            } else if let Some(args) = init_args
                && !args.is_empty()
            {
                self.compile_member_init_expr(fld, args, ctx, code)?;
            }
        }
        Ok(())
    }

    fn compile_member_init_expr(
        &mut self,
        fld: &crate::sema::StructField,
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if fld.ty.is_reference() {
            code.push(OP_LOCAL_GET);
            write_u32(code, 0); // this
            self.compile_lvalue_addr(&args[0], ctx, code)?;
            self.emit_store(&fld.ty, fld.offset as u32, code);
        } else if matches!(fld.ty.strip_const(), Type::Array(_, _)) {
            self.compile_expr(&args[0], ctx, code)?;
            code.push(OP_LOCAL_SET);
            write_u32(code, ctx.scratch1);
            let size = fld.ty.size_of(&self.sema.struct_sizes);
            crate::codegen::raii::emit_struct_byte_copy(
                size,
                fld.offset as u32,
                |code| {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, 0);
                },
                |code| {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, ctx.scratch1);
                },
                code,
            );
        } else {
            code.push(OP_LOCAL_GET);
            write_u32(code, 0); // this
            let item_ty = self.get_expr_type(&args[0], ctx);
            self.compile_expr(&args[0], ctx, code)?;
            self.emit_conversion(&item_ty, &fld.ty, code);
            self.emit_store(&fld.ty, fld.offset as u32, code);
        }
        Ok(())
    }

    pub(super) fn emit_member_destructors(
        &mut self,
        f: &FunctionDecl,
        cls: &str,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if f.name != format!("~{}", cls) {
            return Ok(());
        }
        let Some(layout) = self.sema.struct_layouts.get(cls) else {
            return Ok(());
        };
        let base_field_count = self.base_field_count(layout);
        for fld in layout.fields.iter().skip(base_field_count).rev() {
            if let Type::Struct(sname) = fld.ty.strip_const() {
                let member_dtor = format!("{}_~{}", sname, sname);
                if let Some(&d_idx) = self.func_indices.get(&member_dtor) {
                    code.push(OP_LOCAL_GET);
                    write_u32(code, 0); // this
                    code.push(OP_I32_CONST);
                    write_i32(code, fld.offset as i32);
                    code.push(OP_I32_ADD);
                    code.push(OP_CALL);
                    write_u32(code, d_idx);
                }
            } else if let Type::Array(elem_ty, count) = fld.ty.strip_const()
                && let Type::Struct(sname) = elem_ty.strip_const()
            {
                let member_dtor = format!("{}_~{}", sname, sname);
                let Some(&d_idx) = self.func_indices.get(&member_dtor) else {
                    continue;
                };
                let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
                    .map_err(|_| crate::internal_error!(f.line, f.col, "member is too large"))?;
                for i in (0..*count).rev() {
                    let member_offset = (fld.offset as u32)
                        .checked_add((i as u32).checked_mul(elem_size).ok_or_else(|| {
                            crate::internal_error!(f.line, f.col, "member is too large")
                        })?)
                        .ok_or_else(|| {
                            crate::internal_error!(f.line, f.col, "member is too large")
                        })?;
                    code.push(OP_LOCAL_GET);
                    write_u32(code, 0); // this
                    code.push(OP_I32_CONST);
                    write_i32(code, member_offset as i32);
                    code.push(OP_I32_ADD);
                    code.push(OP_CALL);
                    write_u32(code, d_idx);
                }
            }
        }
        Ok(())
    }

    pub(super) fn emit_base_destructors(&self, ctx: &FunctionContext, code: &mut Vec<u8>) {
        if !ctx.is_dtor {
            return;
        }
        let Some(cls) = ctx.current_class.as_deref() else {
            return;
        };
        let Some(layout) = self.sema.struct_layouts.get(cls) else {
            return;
        };
        for (base_name, base_offset) in layout.base_offsets.iter().rev() {
            let base_dtor = format!("{}_~{}", base_name, base_name);
            if let Some(&b_idx) = self.func_indices.get(&base_dtor) {
                code.push(OP_LOCAL_GET);
                write_u32(code, 0); // this
                if *base_offset != 0 {
                    code.push(OP_I32_CONST);
                    write_i32(code, *base_offset as i32);
                    code.push(OP_I32_ADD);
                }
                code.push(OP_CALL);
                write_u32(code, b_idx);
            }
        }
    }

    pub(super) fn emit_global_destructors(&self, code: &mut Vec<u8>) {
        // Falling out of main destroys namespace-scope objects in reverse
        // construction order.
        for (sname, goffset) in self.global_dtors.iter().rev() {
            let dtor_name = format!("{}_~{}", sname, sname);
            if let Some(&dtor_idx) = self.func_indices.get(&dtor_name) {
                code.push(OP_I32_CONST);
                write_i32(code, *goffset as i32);
                code.push(OP_CALL);
                write_u32(code, dtor_idx);
            }
        }
    }
}
