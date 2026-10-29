//! Initialization of the namespace-scope objects.
//!
//! They are dynamically initialized before `main`'s body; their destructors
//! are queued and run in reverse order.

use super::*;

impl<'a> Codegen<'a> {
    /// Emits the dynamic initializers of the namespace-scope objects.
    pub(super) fn compile_global_initializers(
        &mut self,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let inits = self.global_inits.clone();
        for (gname, gty, init_opt) in inits {
            let Some(&goffset) = self.global_offsets.get(&gname) else {
                continue;
            };
            let global = GlobalInit {
                ty: &gty,
                init: &init_opt,
                offset: goffset,
            };
            if let Type::Struct(sname) = gty.strip_const() {
                self.compile_global_struct_init(&global, sname, ctx, code)?;
            } else if let Type::Array(elem_ty, _) = gty.strip_const() {
                self.compile_global_array_init(&global, elem_ty, ctx, code)?;
            } else if let Some(init_expr) = &init_opt {
                self.compile_global_scalar_init(&global, init_expr, ctx, code)?;
            }
        }
        Ok(())
    }

    /// Emits the initializer of one namespace-scope object.
    fn compile_global_scalar_init(
        &mut self,
        global: &GlobalInit<'_>,
        init_expr: &Expr,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        code.push(OP_I32_CONST);
        write_i32(code, global.offset as i32);
        let init_ty = self.get_expr_type(init_expr, ctx);
        self.compile_expr(init_expr, ctx, code)?;
        self.emit_conversion(&init_ty, global.ty, code);
        self.emit_store(global.ty, 0, code);
        Ok(())
    }

    /// Emits the initializer of one namespace-scope array.
    fn compile_global_array_init(
        &mut self,
        global: &GlobalInit<'_>,
        elem_ty: &Type,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let list = match global.init.as_ref() {
            Some(Expr::InitializerList { elements, .. })
            | Some(Expr::TypedInitList { elements, .. }) => elements,
            _ => return Ok(()),
        };
        let elem_size = elem_ty.size_of(&self.sema.struct_sizes) as u32;
        for (i, item) in list.iter().enumerate() {
            let item_offset = (global.offset + (i as u32) * elem_size) as i32;
            if let (
                Type::Struct(sname),
                Expr::InitializerList { elements: sublist, .. }
                | Expr::TypedInitList { elements: sublist, .. },
            ) = (elem_ty.strip_const(), item)
            {
                let Some(layout) = self.sema.struct_layouts.get(sname) else {
                    continue;
                };
                for (j, sub_item) in sublist.iter().enumerate() {
                    if let Some(fld) = layout.fields.get(j) {
                        code.push(OP_I32_CONST);
                        write_i32(code, item_offset + fld.offset as i32);
                        self.compile_expr(sub_item, ctx, code)?;
                        self.emit_store(&fld.ty, 0, code);
                    }
                }
            } else {
                let item_ty = self.get_expr_type(item, ctx);
                code.push(OP_I32_CONST);
                write_i32(code, item_offset);
                self.compile_expr(item, ctx, code)?;
                self.emit_conversion(&item_ty, elem_ty, code);
                self.emit_store(elem_ty, 0, code);
            }
        }
        Ok(())
    }

    /// Emits the constructor call or the field-wise initializer of one
    /// namespace-scope object.
    ///
    /// The attempts keep the original order: an explicit constructor call,
    /// then a constructor for the initializer list, then the default
    /// constructor, and finally the field-wise initialization.
    fn compile_global_struct_init(
        &mut self,
        global: &GlobalInit<'_>,
        sname: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        self.global_dtors.push((sname.to_string(), global.offset));
        if self.compile_global_ctor_call(global, sname, ctx, code)? {
            return Ok(());
        }
        if self.compile_global_list_ctor(global, sname, ctx, code)? {
            return Ok(());
        }
        if self.compile_global_default_ctor(global, sname, code)? {
            return Ok(());
        }
        self.compile_global_field_init(global, sname, ctx, code)
    }

    /// Emits a constructor call for an explicitly initialized object.
    fn emit_global_ctor(
        &mut self,
        global: &GlobalInit<'_>,
        sname: &str,
        args: &[Expr],
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<bool, CompileError> {
        let arg_types: Vec<Type> = args
            .iter()
            .map(|arg| self.get_expr_type(arg, ctx))
            .collect();
        let Some((ctor_idx, sig)) = self.find_constructor_sig(sname, &arg_types) else {
            return Ok(false);
        };
        code.push(OP_I32_CONST);
        write_i32(code, global.offset as i32);
        let full_args = self.constructor_arguments(&sig, args);
        for (i, arg) in full_args.iter().enumerate() {
            self.compile_call_arg(arg, sig.param_types.get(i + 1), ctx, code)?;
        }
        code.push(OP_CALL);
        write_u32(code, ctor_idx);
        Ok(true)
    }

    /// Emits the constructor call of an explicitly initialized object.
    fn compile_global_ctor_call(
        &mut self,
        global: &GlobalInit<'_>,
        sname: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<bool, CompileError> {
        let Some(Expr::Call { callee, args, .. }) = global.init.as_ref() else {
            return Ok(false);
        };
        if !matches!(&**callee, Expr::Identifier(cname) if cname == sname) {
            return Ok(false);
        }
        self.emit_global_ctor(global, sname, args, ctx, code)
    }

    /// Emits the constructor call for an initializer list.
    fn compile_global_list_ctor(
        &mut self,
        global: &GlobalInit<'_>,
        sname: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<bool, CompileError> {
        let list = match global.init.as_ref() {
            Some(Expr::InitializerList { elements, .. })
            | Some(Expr::TypedInitList { elements, .. }) => elements,
            _ => return Ok(false),
        };
        self.emit_global_ctor(global, sname, list, ctx, code)
    }

    /// Emits the default constructor call of an object.
    fn compile_global_default_ctor(
        &mut self,
        global: &GlobalInit<'_>,
        sname: &str,
        code: &mut Vec<u8>,
    ) -> Result<bool, CompileError> {
        let arg_types = Vec::new();
        let Some(ctor_idx) = self.find_constructor(sname, &arg_types) else {
            return Ok(false);
        };
        code.push(OP_I32_CONST);
        write_i32(code, global.offset as i32);
        code.push(OP_CALL);
        write_u32(code, ctor_idx);
        Ok(true)
    }

    /// Emits the field-wise initialization of one object.
    fn compile_global_field_init(
        &mut self,
        global: &GlobalInit<'_>,
        sname: &str,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let list = match global.init.as_ref() {
            Some(Expr::InitializerList { elements, .. })
            | Some(Expr::TypedInitList { elements, .. }) => elements,
            _ => return Ok(()),
        };
        let Some(layout) = self.sema.struct_layouts.get(sname) else {
            return Ok(());
        };
        for (i, item) in list.iter().enumerate() {
            if let Some(fld) = layout.fields.get(i) {
                let item_ty = self.get_expr_type(item, ctx);
                code.push(OP_I32_CONST);
                write_i32(code, (global.offset + fld.offset as u32) as i32);
                self.compile_expr(item, ctx, code)?;
                self.emit_conversion(&item_ty, &fld.ty, code);
                self.emit_store(&fld.ty, 0, code);
            }
        }
        Ok(())
    }
}
