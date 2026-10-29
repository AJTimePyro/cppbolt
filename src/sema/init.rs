//! Centralized C++17 list-initialization resolver, aggregate classification, and narrowing checks.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitContext {
    DirectList,         // e.g. T x{...}, T{...}, : member{...}
    CopyListInitVar,    // e.g. T x = {...}
    CopyListInitReturn, // e.g. return {...} in a function returning T
    CopyListInitArg,    // e.g. fn({...}) for parameter T
    CopyListAssign,     // e.g. target = {...} for existing lvalue of type T
    ValueInit,          // e.g. T x{}, T{}
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

        // 1. Floating-point to integer
        if s_ty.is_floating() && t_ty.is_integer() {
            return Err(CompileError::new(
                format!(
                    "narrowing conversion from '{}' to '{}' inside braced-init-list",
                    s_ty.to_c_type(),
                    t_ty.to_c_type()
                ),
                pos.line,
                pos.col,
            ));
        }

        // 2. Floating-point to narrower floating-point (double to float)
        if s_ty == &Type::Double && t_ty == &Type::Float {
            if let Expr::FloatLiteral(val, _) = source_expr {
                let fval = *val as f32;
                if fval.is_finite() && (fval as f64 - *val).abs() < 1e-7 {
                    return Ok(());
                }
            }
            return Err(CompileError::new(
                format!(
                    "narrowing conversion from '{}' to '{}' inside braced-init-list",
                    s_ty.to_c_type(),
                    t_ty.to_c_type()
                ),
                pos.line,
                pos.col,
            ));
        }

        // 3. Integer to floating-point
        if s_ty.is_integer() && t_ty.is_floating() {
            if let Expr::IntLiteral(val, _) = source_expr {
                // If it can be represented exactly
                if t_ty == &Type::Float {
                    let fval = *val as f32;
                    if fval as i64 == *val {
                        return Ok(());
                    }
                } else {
                    let dval = *val as f64;
                    if dval as i64 == *val {
                        return Ok(());
                    }
                }
            }
            return Err(CompileError::new(
                format!(
                    "narrowing conversion from '{}' to '{}' inside braced-init-list",
                    s_ty.to_c_type(),
                    t_ty.to_c_type()
                ),
                pos.line,
                pos.col,
            ));
        }

        // 4. Integer to integer with sign change or smaller range
        if s_ty.is_integer() && t_ty.is_integer() {
            let const_val = match source_expr {
                Expr::IntLiteral(val, _) => Some(*val),
                Expr::Unary {
                    op: UnaryOp::Neg,
                    expr,
                    ..
                } => {
                    if let Expr::IntLiteral(val, _) = &**expr {
                        Some(-val)
                    } else {
                        None
                    }
                }
                Expr::Unary {
                    op: UnaryOp::Plus,
                    expr,
                    ..
                } => {
                    if let Expr::IntLiteral(val, _) = &**expr {
                        Some(*val)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(val) = const_val {
                // Check if constant fits in target_ty range
                let fits = match t_ty {
                    Type::Char => (-128..=127).contains(&val),
                    Type::UnsignedChar => (0..=255).contains(&val),
                    Type::Short => (-32768..=32767).contains(&val),
                    Type::UnsignedShort => (0..=65535).contains(&val),
                    Type::Int => (i32::MIN as i64..=i32::MAX as i64).contains(&val),
                    Type::UnsignedInt => (0..=u32::MAX as i64).contains(&val),
                    Type::Long => (i32::MIN as i64..=i32::MAX as i64).contains(&val),
                    Type::UnsignedLong => (0..=u32::MAX as i64).contains(&val),
                    Type::LongLong => true,
                    Type::UnsignedLongLong => val >= 0,
                    Type::Bool => val == 0 || val == 1,
                    _ => true,
                };
                if !fits {
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

            // Non-constant integer to integer
            let s_signed = s_ty.is_signed();
            let t_signed = t_ty.is_signed();
            let s_size = s_ty.size_of(&self.struct_sizes);
            let t_size = t_ty.size_of(&self.struct_sizes);

            if s_size > t_size || s_signed != t_signed {
                return Err(CompileError::new(
                    format!(
                        "narrowing conversion from '{}' to '{}' inside braced-init-list",
                        s_ty.to_c_type(),
                        t_ty.to_c_type()
                    ),
                    pos.line,
                    pos.col,
                ));
            }
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

        // 1. Reference initialization
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

        // 2. Scalars and Enums
        if clean_target.is_integer()
            || clean_target.is_floating()
            || clean_target.is_pointer()
            || matches!(clean_target, Type::Bool | Type::Char | Type::Enum(_))
        {
            if init_elements.is_empty() {
                // Value-initialization to zero
                return Ok(clean_target);
            }
            if init_elements.len() == 1 {
                let elem = &mut init_elements[0];
                let elem_ty = self.analyze_expr(elem)?;
                self.check_narrowing(elem, &elem_ty, &clean_target, pos)?;
                if !self.can_assign(&clean_target, &elem_ty) {
                    return Err(CompileError::new(
                        format!(
                            "cannot initialize scalar of type '{}' with expression of type '{}'",
                            clean_target.to_c_type(),
                            elem_ty.to_c_type()
                        ),
                        pos.line,
                        pos.col,
                    ));
                }
                return Ok(clean_target);
            }
            return Err(CompileError::new(
                "scalar initializer has multiple expressions",
                pos.line,
                pos.col,
            ));
        }

        // 3. Arrays
        if let Type::Array(elem_ty, array_len) = &clean_target {
            if *array_len > 0 && init_elements.len() > *array_len {
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
            return Ok(clean_target);
        }

        // 4. Structs / Classes
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

        // Fallback for any other type
        for elem in init_elements.iter_mut() {
            self.analyze_expr(elem)?;
        }
        Ok(clean_target)
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
            // Value-initialization of aggregate
            return Ok(Type::Struct(sname.to_string()));
        }

        let fields = &layout.fields;
        let mut field_idx = 0;
        let mut elem_idx = 0;

        // Base subobject check if first base class exists and first element is a braced list
        if let Some(base_name) = layout.base_classes.first()
            && let Some(base_layout) = self.struct_layouts.get(base_name).cloned()
        {
            let base_field_count = base_layout.fields.len();
            if elem_idx < init_elements.len()
                && let Expr::InitializerList {
                    elements: sub_elements,
                    pos: sub_pos,
                } = &mut init_elements[elem_idx]
            {
                // Explicitly braced base subobject
                let base_ty = Type::Struct(base_name.clone());
                self.resolve_list_init(&base_ty, sub_elements, InitContext::DirectList, *sub_pos)?;
                elem_idx += 1;
                field_idx += base_field_count;
            }
        }

        // Initialize remaining fields
        while elem_idx < init_elements.len() && field_idx < fields.len() {
            let fld = &fields[field_idx];
            let elem = &mut init_elements[elem_idx];

            if let Expr::InitializerList {
                elements: sub_elements,
                pos: sub_pos,
            } = elem
            {
                self.resolve_list_init(&fld.ty, sub_elements, context, *sub_pos)?;
                elem_idx += 1;
                field_idx += 1;
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
                elem_idx += 1;
                field_idx += 1;
            }
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
            // Default constructor call
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

        // Check narrowing for parameters (skipping the implicit this parameter at index 0)
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

        // No private or protected non-static data members
        if !layout.private_members.is_empty() {
            return false;
        }

        // No virtual functions
        if layout.has_vtable {
            return false;
        }

        // No user-provided, explicit, or inherited constructors
        if layout.has_user_declared_constructors {
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
mod tests {
    use super::*;

    #[test]
    fn test_narrowing_float_to_int_rejected() {
        let sema = Sema::new();
        let expr = Expr::FloatLiteral(3.25, Type::Double);
        let res = sema.check_narrowing(&expr, &Type::Double, &Type::Int, ExprPos::new(1, 1));
        assert!(matches!(res, Err(err) if err.to_string().contains("narrowing conversion")));
    }

    #[test]
    fn test_narrowing_negative_to_unsigned_rejected() {
        let sema = Sema::new();
        let expr = Expr::IntLiteral(-1, Type::Int);
        let res = sema.check_narrowing(&expr, &Type::Int, &Type::UnsignedInt, ExprPos::new(1, 1));
        assert!(matches!(res, Err(err) if err.to_string().contains("narrowing conversion")));
    }

    #[test]
    fn test_narrowing_overflow_constant_rejected() {
        let sema = Sema::new();
        let expr = Expr::IntLiteral(300, Type::Int);
        let res = sema.check_narrowing(&expr, &Type::Int, &Type::Char, ExprPos::new(1, 1));
        assert!(matches!(res, Err(err) if err.to_string().contains("narrowing conversion")));
    }

    #[test]
    fn test_fitting_constant_accepted() {
        let sema = Sema::new();
        let expr = Expr::IntLiteral(42, Type::Int);
        let res = sema.check_narrowing(&expr, &Type::Int, &Type::Char, ExprPos::new(1, 1));
        assert!(res.is_ok());
    }

    #[test]
    fn test_fitting_negative_constant_accepted() {
        let sema = Sema::new();
        let expr = Expr::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expr::IntLiteral(1, Type::Int)),
            pos: ExprPos::new(1, 1),
        };
        let res = sema.check_narrowing(&expr, &Type::Int, &Type::Char, ExprPos::new(1, 1));
        assert!(res.is_ok());
    }

    #[test]
    fn test_user_declared_default_ctor_disqualifies_aggregate() {
        let mut sema = Sema::new();
        let layout = StructLayout {
            is_class: false,
            has_user_declared_constructors: true,
            base_classes: vec![],
            base_offsets: vec![],
            fields: vec![],
            size: 4,
            align: 4,
            has_vtable: false,
            vtable_slots: vec![],
            private_members: HashSet::new(),
            is_abstract: false,
            pure_virtuals: HashSet::new(),
        };
        sema.struct_layouts.insert("HasDefCtor".to_string(), layout);
        assert!(!sema.is_cxx17_aggregate("HasDefCtor"));
    }

    #[test]
    fn test_scalar_excess_elements_rejected() {
        let mut sema = Sema::new();
        let mut elems = vec![
            Expr::IntLiteral(1, Type::Int),
            Expr::IntLiteral(2, Type::Int),
        ];
        let res = sema.resolve_list_init(
            &Type::Int,
            &mut elems,
            InitContext::DirectList,
            ExprPos::new(1, 1),
        );
        assert!(
            matches!(res, Err(err) if err.to_string().contains("scalar initializer has multiple expressions"))
        );
    }

    #[test]
    fn test_explicit_constructor_in_copy_list_rejected() {
        let mut sema = Sema::new();
        // Register struct Foo with explicit constructor Foo(int)
        let layout = StructLayout {
            is_class: true,
            has_user_declared_constructors: true,
            base_classes: vec![],
            base_offsets: vec![],
            fields: vec![],
            size: 4,
            align: 4,
            has_vtable: false,
            vtable_slots: vec![],
            private_members: HashSet::new(),
            is_abstract: false,
            pure_virtuals: HashSet::new(),
        };
        sema.struct_layouts.insert("Foo".to_string(), layout);
        let sig = FunctionSig {
            name: "Foo::Foo".to_string(),
            mangled_name: "_ZN3FooC1Ei".to_string(),
            return_type: Type::Void,
            param_types: vec![
                Type::Pointer(Box::new(Type::Struct("Foo".to_string()))),
                Type::Int,
            ],
            default_args: vec![None, None],
            is_static: false,
            is_explicit: true,
            ref_qualifier: RefQualifier::None,
        };
        sema.functions.insert("Foo::Foo".to_string(), vec![sig]);

        let mut elems = vec![Expr::IntLiteral(42, Type::Int)];
        // DirectList should succeed
        let direct_res = sema.resolve_list_init(
            &Type::Struct("Foo".to_string()),
            &mut elems,
            InitContext::DirectList,
            ExprPos::new(1, 1),
        );
        assert!(direct_res.is_ok());

        // CopyList should fail
        let copy_res = sema.resolve_list_init(
            &Type::Struct("Foo".to_string()),
            &mut elems,
            InitContext::CopyListInitVar,
            ExprPos::new(1, 1),
        );
        assert!(
            matches!(copy_res, Err(err) if err.to_string().contains("is explicit in copy-list-initialization"))
        );
    }
}
