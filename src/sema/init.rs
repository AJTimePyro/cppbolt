//! Centralized C++17 list-initialization resolver, aggregate classification, and narrowing checks.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitContext {
    DirectList,
    CopyListInitVar,
    CopyListInitReturn,
    CopyListInitArg,
    CopyListAssign,
    ValueInit,
}

impl InitContext {
    pub fn is_copy_list(&self) -> bool {
        matches!(
            self,
            InitContext::CopyListInitVar
                | InitContext::CopyListInitReturn
                | InitContext::CopyListInitArg
                | InitContext::CopyListAssign
        )
    }
}

fn narrowing_err(s_ty: &Type, t_ty: &Type, pos: ExprPos) -> CompileError {
    CompileError::new(
        format!(
            "narrowing conversion from '{}' to '{}' inside braced-init-list",
            s_ty.to_c_type(),
            t_ty.to_c_type()
        ),
        pos.line,
        pos.col,
    )
}

fn extract_const_int(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::IntLiteral(val, _) => Some(*val),
        Expr::Unary {
            op: UnaryOp::Neg,
            expr,
            ..
        } => extract_const_int(expr).map(|v| -v),
        Expr::Unary {
            op: UnaryOp::Plus,
            expr,
            ..
        } => extract_const_int(expr),
        _ => None,
    }
}

fn int_fits_type(val: i64, ty: &Type) -> bool {
    match ty {
        Type::Char => (-128..=127).contains(&val),
        Type::UnsignedChar => (0..=255).contains(&val),
        Type::Short => (-32768..=32767).contains(&val),
        Type::UnsignedShort => (0..=65535).contains(&val),
        Type::Int | Type::Long => (i32::MIN as i64..=i32::MAX as i64).contains(&val),
        Type::UnsignedInt | Type::UnsignedLong => (0..=u32::MAX as i64).contains(&val),
        Type::LongLong => true,
        Type::UnsignedLongLong => val >= 0,
        Type::Bool => val == 0 || val == 1,
        _ => true,
    }
}

impl Sema {
    /// Checks whether converting `source_expr` of type `source_ty` to `target_ty` inside a
    /// braced-init-list is a narrowing conversion ([dcl.init.list] p7).
    pub fn check_narrowing(
        &self,
        source_expr: &Expr,
        source_ty: &Type,
        target_ty: &Type,
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        let s_ty = source_ty.strip_reference().strip_const();
        let t_ty = target_ty.strip_reference().strip_const();

        if s_ty == t_ty {
            return Ok(());
        }

        if s_ty.is_floating() && t_ty.is_integer() {
            return Err(narrowing_err(s_ty, t_ty, pos));
        }

        if s_ty == &Type::Double && t_ty == &Type::Float {
            if let Expr::FloatLiteral(val, _) = source_expr {
                let fval = *val as f32;
                if fval.is_finite() && (fval as f64 - *val).abs() < 1e-7 {
                    return Ok(());
                }
            }
            return Err(narrowing_err(s_ty, t_ty, pos));
        }

        if s_ty.is_integer() && t_ty.is_floating() {
            if let Expr::IntLiteral(val, _) = source_expr {
                let ok = if t_ty == &Type::Float {
                    (*val as f32) as i64 == *val
                } else {
                    (*val as f64) as i64 == *val
                };
                if ok {
                    return Ok(());
                }
            }
            return Err(narrowing_err(s_ty, t_ty, pos));
        }

        if s_ty.is_integer() && t_ty.is_integer() {
            self.check_int_narrowing(source_expr, s_ty, t_ty, pos)?;
        }

        Ok(())
    }

    fn check_int_narrowing(
        &self,
        source_expr: &Expr,
        s_ty: &Type,
        t_ty: &Type,
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        if let Some(val) = extract_const_int(source_expr) {
            if !int_fits_type(val, t_ty) {
                return Err(CompileError::new(
                    format!(
                        "narrowing conversion of constant {} from '{}' to '{}' inside braced-init-list",
                        val,
                        s_ty.to_c_type(),
                        t_ty.to_c_type()
                    ),
                    pos.line,
                    pos.col,
                ));
            }
            return Ok(());
        }

        let s_size = s_ty.size_of(&self.struct_sizes);
        let t_size = t_ty.size_of(&self.struct_sizes);
        if s_size > t_size || s_ty.is_signed() != t_ty.is_signed() {
            return Err(narrowing_err(s_ty, t_ty, pos));
        }
        Ok(())
    }

    /// Resolves list-initialization of `target_ty` with `init_elements` in `context`.
    pub fn resolve_list_init(
        &mut self,
        target_ty: &Type,
        init_elements: &mut [Expr],
        context: InitContext,
        pos: ExprPos,
    ) -> Result<Type, CompileError> {
        let clean_target = target_ty.strip_reference().strip_const().clone();

        if let Type::Reference(inner) = target_ty {
            if !inner.is_const() && context != InitContext::DirectList {
                return Err(CompileError::new(
                    format!(
                        "non-const lvalue reference to type '{}' cannot bind to an initializer list temporary",
                        target_ty.to_c_type()
                    ),
                    pos.line,
                    pos.col,
                ));
            }
            return self.resolve_list_init(inner, init_elements, InitContext::DirectList, pos);
        }

        if clean_target.is_integer()
            || clean_target.is_floating()
            || clean_target.is_pointer()
            || matches!(clean_target, Type::Bool | Type::Char | Type::Enum(_))
        {
            return self.resolve_scalar_list_init(&clean_target, init_elements, pos);
        }

        if let Type::Array(elem_ty, array_len) = &clean_target {
            return self.resolve_array_list_init(elem_ty, *array_len, init_elements, context, pos);
        }

        if let Type::Struct(ref sname) = clean_target {
            if sname == "initializer_list" || sname == "std::initializer_list" {
                return Err(CompileError::new(
                    "std::initializer_list requires template support which is not yet available",
                    pos.line,
                    pos.col,
                ));
            }

            if self.is_cxx17_aggregate(sname) {
                return self.resolve_aggregate_list_init(sname, init_elements, context, pos);
            } else {
                return self.resolve_class_list_init(sname, init_elements, context, pos);
            }
        }

        for elem in init_elements.iter_mut() {
            self.analyze_expr(elem)?;
        }
        Ok(clean_target)
    }

    fn resolve_scalar_list_init(
        &mut self,
        target: &Type,
        init_elements: &mut [Expr],
        pos: ExprPos,
    ) -> Result<Type, CompileError> {
        if init_elements.is_empty() {
            return Ok(target.clone());
        }
        if init_elements.len() == 1 {
            let elem = &mut init_elements[0];
            let elem_ty = self.analyze_expr(elem)?;
            self.check_narrowing(elem, &elem_ty, target, pos)?;
            if !self.can_assign(target, &elem_ty) {
                return Err(CompileError::new(
                    format!(
                        "cannot initialize scalar of type '{}' with expression of type '{}'",
                        target.to_c_type(),
                        elem_ty.to_c_type()
                    ),
                    pos.line,
                    pos.col,
                ));
            }
            return Ok(target.clone());
        }
        Err(CompileError::new(
            "scalar initializer has multiple expressions",
            pos.line,
            pos.col,
        ))
    }

    fn resolve_array_list_init(
        &mut self,
        elem_ty: &Type,
        array_len: usize,
        init_elements: &mut [Expr],
        context: InitContext,
        pos: ExprPos,
    ) -> Result<Type, CompileError> {
        if array_len > 0 && init_elements.len() > array_len {
            return Err(CompileError::new(
                "excess elements in array initializer list",
                pos.line,
                pos.col,
            ));
        }
        for elem in init_elements.iter_mut() {
            if let Expr::InitializerList {
                elements: sub_elements,
                pos: sub_pos,
            } = elem
            {
                self.resolve_list_init(elem_ty, sub_elements, context, *sub_pos)?;
            } else {
                let elem_actual_ty = self.analyze_expr(elem)?;
                self.check_narrowing(elem, &elem_actual_ty, elem_ty, pos)?;
                if !self.can_assign(elem_ty, &elem_actual_ty) {
                    return Err(CompileError::new(
                        format!(
                            "cannot initialize array element of type '{}' with expression of type '{}'",
                            elem_ty.to_c_type(),
                            elem_actual_ty.to_c_type()
                        ),
                        pos.line,
                        pos.col,
                    ));
                }
            }
        }
        Ok(Type::Array(Box::new(elem_ty.clone()), array_len))
    }

    fn resolve_aggregate_base_class(
        &mut self,
        layout: &StructLayout,
        init_elements: &mut [Expr],
        elem_idx: &mut usize,
        field_idx: &mut usize,
    ) -> Result<(), CompileError> {
        if let Some(base_name) = layout.base_classes.first()
            && let Some(base_layout) = self.struct_layouts.get(base_name).cloned()
        {
            let base_field_count = base_layout.fields.len();
            if *elem_idx < init_elements.len()
                && let Expr::InitializerList {
                    elements: sub_elements,
                    pos: sub_pos,
                } = &mut init_elements[*elem_idx]
            {
                let base_ty = Type::Struct(base_name.clone());
                self.resolve_list_init(&base_ty, sub_elements, InitContext::DirectList, *sub_pos)?;
                *elem_idx += 1;
                *field_idx += base_field_count;
            }
        }
        Ok(())
    }

    /// Resolves aggregate initialization with support for C++17 base classes and brace elision.
    fn resolve_aggregate_list_init(
        &mut self,
        sname: &str,
        init_elements: &mut [Expr],
        context: InitContext,
        pos: ExprPos,
    ) -> Result<Type, CompileError> {
        let Some(layout) = self.struct_layouts.get(sname).cloned() else {
            return Err(CompileError::new(
                format!("unknown struct type '{}'", sname),
                pos.line,
                pos.col,
            ));
        };

        if init_elements.is_empty() {
            return Ok(Type::Struct(sname.to_string()));
        }

        let fields = &layout.fields;
        let mut field_idx = 0;
        let mut elem_idx = 0;

        self.resolve_aggregate_base_class(&layout, init_elements, &mut elem_idx, &mut field_idx)?;

        while elem_idx < init_elements.len() && field_idx < fields.len() {
            let fld = &fields[field_idx];
            let elem = &mut init_elements[elem_idx];

            if let Expr::InitializerList {
                elements: sub_elements,
                pos: sub_pos,
            } = elem
            {
                self.resolve_list_init(&fld.ty, sub_elements, context, *sub_pos)?;
            } else {
                let elem_ty = self.analyze_expr(elem)?;
                self.check_narrowing(elem, &elem_ty, &fld.ty, pos)?;
                if !self.can_assign(&fld.ty, &elem_ty) {
                    return Err(CompileError::new(
                        format!(
                            "cannot initialize field '{}' of type '{}' with expression of type '{}'",
                            fld.name,
                            fld.ty.to_c_type(),
                            elem_ty.to_c_type()
                        ),
                        pos.line,
                        pos.col,
                    ));
                }
            }
            elem_idx += 1;
            field_idx += 1;
        }

        if elem_idx < init_elements.len() {
            return Err(CompileError::new(
                format!("too many initializers for aggregate '{}'", sname),
                pos.line,
                pos.col,
            ));
        }

        Ok(Type::Struct(sname.to_string()))
    }

    /// Resolves class constructor initialization via list-init.
    fn resolve_class_list_init(
        &mut self,
        sname: &str,
        init_elements: &mut [Expr],
        context: InitContext,
        pos: ExprPos,
    ) -> Result<Type, CompileError> {
        if init_elements.is_empty() {
            let empty_args: Vec<Type> = Vec::new();
            if let Some(sig) = self.find_constructor_for(sname, &empty_args) {
                if sig.is_explicit && context.is_copy_list() {
                    return Err(CompileError::new(
                        format!(
                            "chosen default constructor for '{}' is explicit in copy-list-initialization",
                            sname
                        ),
                        pos.line,
                        pos.col,
                    ));
                }
                return Ok(Type::Struct(sname.to_string()));
            }
            return Err(CompileError::new(
                format!("no matching default constructor for '{}'", sname),
                pos.line,
                pos.col,
            ));
        }

        let mut arg_types = Vec::new();
        for elem in init_elements.iter_mut() {
            arg_types.push(self.analyze_expr(elem)?);
        }

        let Some(sig) = self.find_constructor_for(sname, &arg_types) else {
            return Err(CompileError::new(
                format!("no matching constructor for initialization of '{}'", sname),
                pos.line,
                pos.col,
            ));
        };

        if sig.is_explicit && context.is_copy_list() {
            return Err(CompileError::new(
                format!(
                    "chosen constructor for '{}' is explicit in copy-list-initialization",
                    sname
                ),
                pos.line,
                pos.col,
            ));
        }

        for (i, elem) in init_elements.iter().enumerate() {
            if let Some(param_ty) = sig.param_types.get(i + 1) {
                self.check_narrowing(elem, &arg_types[i], param_ty, pos)?;
            }
        }

        Ok(Type::Struct(sname.to_string()))
    }

    /// Determines if a class is an aggregate under C++17 rules ([dcl.init.aggr]).
    pub fn is_cxx17_aggregate(&self, sname: &str) -> bool {
        let Some(layout) = self.struct_layouts.get(sname) else {
            return false;
        };

        if !layout.private_members.is_empty()
            || layout.has_vtable
            || layout.has_user_declared_constructors
        {
            return false;
        }

        let short = sname.split("::").last().unwrap_or(sname);
        let ctor_name = format!("{}::{}", sname, short);
        if let Some(ctors) = self.functions.get(&ctor_name) {
            for ctor in ctors {
                if ctor.is_explicit {
                    return false;
                }
            }
        }

        true
    }
}

#[cfg(test)]
#[path = "init_tests.rs"]
mod tests;
