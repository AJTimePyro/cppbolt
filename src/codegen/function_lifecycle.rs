//! Constructor and destructor lowering for functions.

use super::*;
use crate::sema::StructLayout;

struct BaseCopyTarget<'a> {
    name: &'a str,
    layout: &'a StructLayout,
    offset: u32,
}

impl<'a> Codegen<'a> {
    pub(super) fn compile_constructor_prologue(
        &mut self,
        f: &FunctionDecl,
        cls: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let short_cls = QualifiedName::new(cls).class_short_name();
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
        let short_cls = QualifiedName::new(cls).class_short_name();
        if let Some((_, target_args)) = f
            .ctor_initializers
            .iter()
            .find(|(name, _)| name == cls || name == short_cls)
        {
            let arg_types: Vec<Type> = target_args
                .iter()
                .map(|a| self.get_expr_type(a, ctx))
                .collect();
            let Some((ctor_idx, sig)) =
                self.find_constructor_sig_with_args(cls, &arg_types, Some(target_args))
            else {
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

    /// Returns `(vptr_offset, vtable_address)` for every polymorphic
    /// subobject of `cls`.
    pub(super) fn vtable_installs(&self, cls: &str) -> Vec<(u32, u32)> {
        self.vtable_offsets.get(cls).cloned().unwrap_or_default()
    }
    /// Installs all of the class's vtable pointers in the object under
    /// construction. `this` is local 0, as in every constructor body.
    pub(super) fn install_class_vtable(&mut self, cls: &str, code: &mut Vec<u8>) {
        for (vptr_offset, v_off) in self.vtable_installs(cls) {
            code.push(OP_LOCAL_GET);
            write_u32(code, 0); // this
            code.push(OP_I32_CONST);
            write_i32(code, v_off as i32);
            code.push(OP_I32_STORE);
            write_u32(code, 2);
            write_u32(code, vptr_offset);
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
        for (base_name, base_offset) in &layout.direct_base_offsets {
            let base_args = f
                .ctor_initializers
                .iter()
                .find(|(name, _)| self.sema.canonical_class_name(name, Some(cls)) == *base_name)
                .map(|(_, args)| args.as_slice())
                .unwrap_or(&[]);
            let arg_types: Vec<Type> = base_args
                .iter()
                .map(|a| self.get_expr_type(a, ctx))
                .collect();
            if let Some((b_idx, sig)) =
                self.find_constructor_sig_with_args(base_name, &arg_types, Some(base_args))
            {
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
                let whole_object = base_args.len() == 1
                    && matches!(
                        self.get_expr_type(&base_args[0], ctx)
                            .strip_reference()
                            .strip_const(),
                        Type::Struct(_)
                    );
                if whole_object {
                    self.emit_base_subobject_copy(
                        BaseCopyTarget {
                            name: base_name,
                            layout: &base_layout,
                            offset: *base_offset as u32,
                        },
                        &base_args[0],
                        ctx,
                        code,
                    )?;
                } else {
                    self.emit_field_initializers(
                        &base_layout,
                        base_args,
                        *base_offset as u32,
                        ctx,
                        code,
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Copies a whole base subobject from a synthesized copy/move constructor
    /// argument. The argument is an object, not a field value, so it must be
    /// copied bytewise; treating it as a field value would store the object's
    /// address with the field's width (e.g. `i64.store` of an `i32`).
    fn emit_base_subobject_copy(
        &mut self,
        target: BaseCopyTarget<'_>,
        src: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let base_name = target.name;
        let base_layout = target.layout;
        let base_offset = target.offset;
        // Destination: this + base_offset, held in scratch2.
        code.push(OP_LOCAL_GET);
        write_u32(code, 0);
        if base_offset != 0 {
            code.push(OP_I32_CONST);
            write_i32(code, base_offset as i32);
            code.push(OP_I32_ADD);
        }
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch2);

        // Source: the argument may already point at the base subobject (move
        // constructors cast to Base&&), or at the complete derived object.
        self.compile_expr(src, ctx, code)?;
        let src_is_base = matches!(
            self.get_expr_type(src, ctx).strip_reference().strip_const(),
            Type::Struct(name) if name == base_name
        );
        if !src_is_base && base_offset != 0 {
            code.push(OP_I32_CONST);
            write_i32(code, base_offset as i32);
            code.push(OP_I32_ADD);
        }
        code.push(OP_LOCAL_SET);
        write_u32(code, ctx.scratch1);

        for i in 0..base_layout.size {
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch2);
            code.push(OP_LOCAL_GET);
            write_u32(code, ctx.scratch1);
            code.push(OP_I32_LOAD8_U);
            write_u32(code, 0);
            write_u32(code, i as u32);
            code.push(OP_I32_STORE8);
            write_u32(code, 0);
            write_u32(code, i as u32);
        }
        Ok(())
    }

    pub(super) fn emit_field_initializers(
        &mut self,
        layout: &StructLayout,
        args: &[Expr],
        base_offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        for (j, a) in args.iter().enumerate() {
            if let Some(sub_fld) = layout.fields.get(j) {
                code.push(OP_LOCAL_GET);
                write_u32(code, 0); // this
                let a_ty = self.get_expr_type(a, ctx);
                self.compile_expr(a, ctx, code)?;
                self.emit_conversion(&a_ty, &sub_fld.ty, code);
                self.emit_store(&sub_fld.ty, base_offset + sub_fld.offset as u32, code);
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
            let user_args = f
                .ctor_initializers
                .iter()
                .find(|(name, _)| name == &fld.name)
                .map(|(_, args)| args.as_slice());
            let init_args =
                user_args.or_else(|| fld.default_init.as_ref().map(std::slice::from_ref));
            if let Type::Struct(sname) = fld.ty.strip_const() {
                self.compile_struct_field_init(sname, fld.offset, init_args, ctx, code)?;
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

    pub(super) fn emit_member_destructors(
        &mut self,
        func_name: &str,
        cls: &str,
        line: usize,
        col: usize,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let short_cls = QualifiedName::new(cls).class_short_name();
        if func_name != format!("~{}", cls) && func_name != format!("~{}", short_cls) {
            return Ok(());
        }
        let Some(layout) = self.sema.struct_layouts.get(cls) else {
            return Ok(());
        };
        let base_field_count = self.base_field_count(layout);
        for fld in layout.fields.iter().skip(base_field_count).rev() {
            if let Type::Struct(sname) = fld.ty.strip_const() {
                if let Some(d_idx) = self.lookup_struct_dtor(sname) {
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
                let Some(d_idx) = self.lookup_struct_dtor(sname) else {
                    continue;
                };
                let elem_size = u32::try_from(elem_ty.size_of(&self.sema.struct_sizes))
                    .map_err(|_| crate::internal_error!(line, col, "member is too large"))?;
                for i in (0..*count).rev() {
                    let member_offset = (fld.offset as u32)
                        .checked_add((i as u32).checked_mul(elem_size).ok_or_else(|| {
                            crate::internal_error!(line, col, "member is too large")
                        })?)
                        .ok_or_else(|| {
                            crate::internal_error!(line, col, "member is too large")
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
        for (base_name, base_offset) in layout.direct_base_offsets.iter().rev() {
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
