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
            if self.sema.is_initializer_list_type(&gty) {
                self.compile_global_initializer_list(&global, ctx, code)?;
            } else if let Type::Struct(sname) = gty.strip_const() {
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
        match init_expr {
            Expr::InitializerList { elements, .. } | Expr::TypedInitList { elements, .. } => {
                if elements.is_empty() {
                    self.emit_zero_value(global.ty, code);
                } else if elements.len() == 1 {
                    let init_ty = self.get_expr_type(&elements[0], ctx);
                    self.compile_expr(&elements[0], ctx, code)?;
                    self.emit_conversion(&init_ty, global.ty, code);
                }
            }
            _ => {
                let init_ty = self.get_expr_type(init_expr, ctx);
                self.compile_expr(init_expr, ctx, code)?;
                self.emit_conversion(&init_ty, global.ty, code);
            }
        }
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
        let Type::Array(_, total_count) = global.ty.strip_const() else {
            return Ok(());
        };
        let elem_size = elem_ty.size_of(&self.sema.struct_sizes) as u32;
        let mut cursor = GlobalCursor { list, idx: 0 };
        for i in 0..*total_count {
            let item_offset = global.offset + (i as u32) * elem_size;
            self.compile_global_field_elided(elem_ty, &mut cursor, item_offset, ctx, code)?;
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
        if global.init.is_none() && self.compile_global_default_ctor(global, sname, code)? {
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
        let Some(layout) = self.sema.struct_layouts.get(sname).cloned() else {
            return Ok(());
        };
        let mut field_idx = 0;
        let mut cursor = GlobalCursor { list, idx: 0 };
        if let Some(base_name) = layout.base_classes.first()
            && let Some(base_layout) = self.sema.struct_layouts.get(base_name).cloned()
            && matches!(
                cursor.peek(),
                Some(Expr::InitializerList { .. } | Expr::TypedInitList { .. })
            )
        {
            let base_ty = Type::Struct(base_name.clone());
            self.compile_global_field_elided(&base_ty, &mut cursor, global.offset, ctx, code)?;
            field_idx += base_layout.fields.len();
        }
        while field_idx < layout.fields.len() {
            let fld = &layout.fields[field_idx];
            let field_offset = global.offset + fld.offset as u32;
            if !cursor.is_empty() {
                self.compile_global_field_elided(&fld.ty, &mut cursor, field_offset, ctx, code)?;
            } else if let Some(default_init) = &fld.default_init {
                let item_ty = self.get_expr_type(default_init, ctx);
                code.push(OP_I32_CONST);
                write_i32(code, field_offset as i32);
                self.compile_expr(default_init, ctx, code)?;
                self.emit_conversion(&item_ty, &fld.ty, code);
                self.emit_store(&fld.ty, 0, code);
            } else {
                code.push(OP_I32_CONST);
                write_i32(code, field_offset as i32);
                self.emit_zero_value(&fld.ty, code);
                self.emit_store(&fld.ty, 0, code);
            }
            field_idx += 1;
        }
        Ok(())
    }

    fn compile_global_field_scalar(
        &mut self,
        target_ty: &Type,
        init: &Expr,
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let mut cur = init;
        while let Expr::InitializerList { elements, .. }
        | Expr::TypedInitList { elements, .. } = cur
        {
            if let Some(first) = elements.first() {
                cur = first;
            } else {
                code.push(OP_I32_CONST);
                write_i32(code, offset as i32);
                self.emit_zero_value(target_ty, code);
                self.emit_store(target_ty, 0, code);
                return Ok(());
            }
        }
        let item_ty = self.get_expr_type(cur, ctx);
        code.push(OP_I32_CONST);
        write_i32(code, offset as i32);
        self.compile_expr(cur, ctx, code)?;
        self.emit_conversion(&item_ty, target_ty, code);
        self.emit_store(target_ty, 0, code);
        Ok(())
    }

    fn compile_global_field_braced(
        &mut self,
        target_ty: &Type,
        elements: &[Expr],
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        let mut sub = GlobalCursor { list: elements, idx: 0 };
        if let Type::Array(elem_ty, len) = target_ty.strip_const() {
            let elem_size = elem_ty.size_of(&self.sema.struct_sizes) as u32;
            for i in 0..*len {
                let elem_offset = offset + (i as u32) * elem_size;
                self.compile_global_field_elided(elem_ty, &mut sub, elem_offset, ctx, code)?;
            }
        } else if let Type::Struct(sname) = target_ty.strip_const()
            && let Some(layout) = self.sema.struct_layouts.get(sname).cloned()
        {
            let mut f_idx = 0;
            if let Some(base_name) = layout.base_classes.first()
                && let Some(base_layout) = self.sema.struct_layouts.get(base_name).cloned()
                && matches!(
                    sub.peek(),
                    Some(Expr::InitializerList { .. } | Expr::TypedInitList { .. })
                )
            {
                let base_ty = Type::Struct(base_name.clone());
                self.compile_global_field_elided(&base_ty, &mut sub, offset, ctx, code)?;
                f_idx += base_layout.fields.len();
            }
            while f_idx < layout.fields.len() {
                let fld = &layout.fields[f_idx];
                let fld_offset = offset + fld.offset as u32;
                self.compile_global_field_elided(&fld.ty, &mut sub, fld_offset, ctx, code)?;
                f_idx += 1;
            }
        }
        Ok(())
    }

    fn compile_global_field_elided(
        &mut self,
        target_ty: &Type,
        cursor: &mut GlobalCursor<'_>,
        offset: u32,
        ctx: &mut FunctionContext,
        code: &mut Vec<u8>,
    ) -> Result<(), CompileError> {
        if cursor.is_empty() {
            code.push(OP_I32_CONST);
            write_i32(code, offset as i32);
            self.emit_zero_value(target_ty, code);
            self.emit_store(target_ty, 0, code);
            return Ok(());
        }
        if let Type::Array(elem_ty, len) = target_ty.strip_const()
            && let Some(peeked) = cursor.peek()
            && self.sema.is_subaggregate_element_init(elem_ty, peeked)
        {
            let elem_size = elem_ty.size_of(&self.sema.struct_sizes) as u32;
            for i in 0..*len {
                let elem_offset = offset + (i as u32) * elem_size;
                self.compile_global_field_elided(elem_ty, cursor, elem_offset, ctx, code)?;
            }
            return Ok(());
        }
        if let Some(
            Expr::InitializerList { elements, .. } | Expr::TypedInitList { elements, .. },
        ) = cursor.peek()
        {
            if !matches!(target_ty.strip_const(), Type::Array(..) | Type::Struct(..))
                && let Some(item) = cursor.next()
            {
                return self.compile_global_field_scalar(target_ty, item, offset, ctx, code);
            }
            let res = self.compile_global_field_braced(target_ty, elements, offset, ctx, code);
            cursor.next();
            return res;
        }
        if let Type::Array(elem_ty, len) = target_ty.strip_const() {
            let elem_size = elem_ty.size_of(&self.sema.struct_sizes) as u32;
            for i in 0..*len {
                let elem_offset = offset + (i as u32) * elem_size;
                self.compile_global_field_elided(elem_ty, cursor, elem_offset, ctx, code)?;
            }
            return Ok(());
        }
        if let Type::Struct(sname) = target_ty.strip_const()
            && self.sema.is_cxx17_aggregate(sname)
            && let Some(layout) = self.sema.struct_layouts.get(sname).cloned()
        {
            for fld in &layout.fields {
                let fld_offset = offset + fld.offset as u32;
                self.compile_global_field_elided(&fld.ty, cursor, fld_offset, ctx, code)?;
            }
            return Ok(());
        }
        if let Some(item) = cursor.next() {
            self.compile_global_field_scalar(target_ty, item, offset, ctx, code)?;
        } else {
            code.push(OP_I32_CONST);
            write_i32(code, offset as i32);
            self.emit_zero_value(target_ty, code);
            self.emit_store(target_ty, 0, code);
        }
        Ok(())
    }
}

struct GlobalCursor<'a> {
    list: &'a [Expr],
    idx: usize,
}

impl<'a> GlobalCursor<'a> {
    fn is_empty(&self) -> bool {
        self.idx >= self.list.len()
    }

    fn peek(&self) -> Option<&'a Expr> {
        self.list.get(self.idx)
    }

    fn next(&mut self) -> Option<&'a Expr> {
        let item = self.list.get(self.idx)?;
        self.idx += 1;
        Some(item)
    }
}
