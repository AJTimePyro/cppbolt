use super::*;

impl Sema {
    pub fn analyze_expr(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let _guard = self
            .depth
            .expressions()
            .enter(self.current_line, self.current_col)?;
        let ty = self.analyze_expr_raw(expr)?;
        Ok(ty.strip_reference().clone())
    }

    /// Analyzes an expression and keeps a reference type.
    ///
    /// The caller needs the reference to tell an lvalue from a prvalue, so
    /// the type is returned before the reference is dropped.
    pub(super) fn analyze_expr_keeping_reference(
        &mut self,
        expr: &mut Expr,
    ) -> Result<Type, CompileError> {
        let _guard = self
            .depth
            .expressions()
            .enter(self.current_line, self.current_col)?;
        self.analyze_expr_raw(expr)
    }
    pub(super) fn analyze_expr_raw(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        match expr {
            Expr::IntLiteral(_, ty) => Ok(ty.clone()),
            Expr::FloatLiteral(_, ty) => Ok(ty.clone()),
            Expr::BoolLiteral(_) => Ok(Type::Bool),
            Expr::CharLiteral(_) => Ok(Type::Char),
            Expr::StringLiteral(s) => Ok(Type::Array(
                Box::new(Type::Const(Box::new(Type::Char))),
                s.len() + 1,
            )),
            Expr::This => {
                if let Some(cls) = &self.current_class {
                    let struct_ty = if self.current_func_is_const {
                        Type::Const(Box::new(Type::Struct(cls.clone())))
                    } else {
                        Type::Struct(cls.clone())
                    };
                    Ok(Type::Pointer(Box::new(struct_ty)))
                } else {
                    Err(CompileError::new(
                        "'this' cannot be used outside of member function",
                        self.current_line,
                        self.current_col,
                    ))
                }
            }
            Expr::New { .. } => self.analyze_new(expr),
            Expr::Delete { expr, .. } => {
                self.analyze_expr(expr)?;
                Ok(Type::Void)
            }
            Expr::Identifier { .. } => self.analyze_identifier(expr),
            Expr::Binary { .. } => self.analyze_binary(expr),
            Expr::Unary { .. } => self.analyze_unary(expr),
            Expr::Assign { .. } => self.analyze_assign_expr(expr),
            Expr::CompoundAssign { .. } => self.analyze_compound_assign(expr),
            Expr::Call { .. } => self.analyze_call(expr),
            Expr::ArraySubscript { .. } => self.analyze_subscript(expr),
            Expr::MemberAccess { .. } => self.analyze_member(expr),
            Expr::Ternary { .. } => self.analyze_ternary(expr),
            Expr::Cast {
                kind,
                target_type,
                expr,
            } => self.analyze_cast(*kind, target_type, expr),
            Expr::SizeofType(_) | Expr::SizeofExpr(_) => Ok(Type::UnsignedInt),
            Expr::InitializerList { .. } | Expr::TypedInitList { .. } => {
                self.analyze_initializer(expr)
            }
            Expr::Lambda { .. } => self.analyze_lambda(expr),
        }
    }

    fn analyze_cast(
        &mut self,
        kind: CastKind,
        target_type: &Type,
        expr: &mut Expr,
    ) -> Result<Type, CompileError> {
        let expr_ty = self.analyze_expr(expr)?;
        if kind == CastKind::Reinterpret {
            if target_type.is_reference() && !expr.is_lvalue() {
                return Err(CompileError::new(
                    "reinterpret_cast to reference type requires an lvalue operand",
                    self.current_line,
                    self.current_col,
                ));
            }
            self.validate_reinterpret_cast(&expr_ty, target_type)?;
            return Ok(target_type.clone());
        }
        if let Type::Struct(sname) = expr_ty.strip_reference().strip_const() {
            let conv_name = format!("operator {}", target_type.to_c_type());
            let is_const = expr_ty.strip_reference().is_const();
            if let Some(sig) = self.find_method_in_class_for_receiver(sname, &conv_name, is_const)
            {
                let mangled = sig.mangled_name.clone();
                self.instantiate_pending_method(&mangled)?;
                let old_expr = expr.clone();
                let pos = ExprPos::new(self.current_line, self.current_col);
                *expr = Expr::Call {
                    callee: Box::new(Expr::MemberAccess {
                        object: Box::new(old_expr),
                        member: conv_name,
                        is_arrow: false,
                        pos,
                    }),
                    args: vec![],
                    pos,
                };
                return Ok(target_type.clone());
            }
        }
        if kind == CastKind::Functional
            && target_type.is_scalar()
            && !self.can_assign_expr(target_type, &expr_ty, Some(expr))
        {
            return Err(CompileError::new(
                format!(
                    "cannot convert '{}' to '{}' in functional cast",
                    expr_ty.to_c_type(),
                    target_type.to_c_type()
                ),
                self.current_line,
                self.current_col,
            ));
        }
        Ok(target_type.clone())
    }

    fn validate_reinterpret_cast(&self, src: &Type, target: &Type) -> Result<(), CompileError> {
        let clean_src = src.strip_const().strip_reference();
        let clean_tgt = target.strip_const().strip_reference();
        let is_ref = target.is_reference() || src.is_reference();
        let invalid_float = !is_ref
            && (matches!(clean_src, Type::Float | Type::Double)
                || matches!(clean_tgt, Type::Float | Type::Double));
        let invalid_struct = !is_ref
            && (matches!(clean_src, Type::Struct(_)) || matches!(clean_tgt, Type::Struct(_)));
        let valid = (clean_src.is_pointer() || clean_src == &Type::Nullptr)
            && (clean_tgt.is_pointer() || clean_tgt.is_integer())
            || clean_src.is_integer() && clean_tgt.is_pointer()
            || target.is_reference();

        if invalid_float || invalid_struct || !valid {
            return Err(CompileError::new(
                format!(
                    "reinterpret_cast from '{}' to '{}' is not allowed",
                    src.to_c_type(),
                    target.to_c_type()
                ),
                self.current_line,
                self.current_col,
            ));
        }
        Ok(())
    }

    pub(super) fn analyze_new(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::New {
            ty,
            args,
            is_array,
            array_size,
            placement,
        } = expr
        else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        if let Some(p) = placement {
            self.analyze_expr(p)?;
        }
        if let Type::Struct(sname) = ty.strip_const()
            && let Some(layout) = self.struct_layouts.get(sname)
            && layout.is_abstract
        {
            return Err(CompileError::new(
                format!("Cannot allocate an object of abstract type '{}'", sname),
                self.current_line,
                self.current_col,
            ));
        }
        let pos = ExprPos::new(self.current_line, self.current_col);
        if *is_array {
            if let Some(s) = array_size {
                self.analyze_expr(s)?;
                if let Expr::IntLiteral(count, _) = &**s
                    && *count >= 0
                    && args.len() > *count as usize
                {
                    return Err(CompileError::new(
                        format!(
                            "excess elements in array initializer: expected at most {}, have {}",
                            count,
                            args.len()
                        ),
                        pos.line,
                        pos.col,
                    ));
                }
            }
            for a in args.iter_mut() {
                let a_ty = self.analyze_expr(a)?;
                self.check_narrowing(a, &a_ty, ty, pos)?;
            }
            Ok(Type::Pointer(Box::new(ty.clone())))
        } else {
            let mut arg_types = Vec::new();
            for a in args.iter_mut() {
                arg_types.push(self.analyze_expr(a)?);
            }
            if let Type::Struct(sname) = ty.strip_const() {
                if self.is_cxx17_aggregate(sname) {
                    self.resolve_list_init(ty, args, InitContext::DirectList, pos)?;
                } else {
                    let _ = self.analyze_constructor_call(sname, args, &arg_types, pos)?;
                }
            }
            Ok(Type::Pointer(Box::new(ty.clone())))
        }
    }

    /// Reports a name that is visible through more than one using-directive
    /// or through a using-declaration and a using-directive at the same time.
    pub(super) fn check_ambiguous_name(&self, name: &str) -> Result<(), CompileError> {
        let mut providers: Vec<&str> = Vec::new();
        if self.using_decls.contains_key(name) {
            providers.push("a using-declaration");
        }
        let mut seen: Vec<&str> = Vec::new();
        for ns in &self.open_namespaces {
            if seen.contains(&ns.as_str()) {
                continue;
            }
            if self.global_vars.contains_key(&format!("{}::{}", ns, name)) {
                seen.push(ns);
                providers.push(ns);
            }
        }
        if providers.len() > 1 {
            return Err(CompileError::new(
                format!("reference to '{}' is ambiguous", name),
                self.current_line,
                self.current_col,
            ));
        }
        Ok(())
    }

    pub(super) fn analyze_identifier(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::Identifier(name) = expr else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };
        let resolved = self.resolve_alias_in_qualified_name(name);
        if resolved != *name {
            *name = resolved;
        }

        if let Some(global_name) = name.strip_prefix("::") {
            return self.analyze_global_identifier(name, global_name);
        }
        self.check_ambiguous_name(name)?;
        if let Some((class_part, _)) = name.rsplit_once("::") {
            self.ensure_class_instantiated(class_part, self.current_line, self.current_col);
        }
        if let Some(var) = self.lookup_var(name) {
            Ok(var.ty.clone())
        } else if let Some(&val) = self.enum_values.get(name) {
            let enum_ty = self
                .enum_types
                .get(name)
                .map(|t| Type::Enum(t.clone()))
                .unwrap_or(Type::Int);
            *expr = Expr::IntLiteral(val, enum_ty.clone());
            Ok(enum_ty)
        } else if let Some(cls) = &self.current_class {
            if let Some(fld) = self.find_field_in_class(cls, name) {
                *expr = Expr::MemberAccess {
                    object: Box::new(Expr::This),
                    member: name.to_string(),
                    is_arrow: true,
                    pos: ExprPos::new(self.current_line, self.current_col),
                };
                let mut ret_ty = fld.ty.clone();
                if self.current_func_is_const && !ret_ty.is_reference() && !ret_ty.is_const() {
                    ret_ty = Type::Const(Box::new(ret_ty));
                }
                Ok(ret_ty)
            } else if let Some(var) = self.lookup_var(&format!("{}::{}", cls, name)) {
                Ok(var.ty.clone())
            } else if self.find_method_in_class(cls, name).is_some()
                || self.functions.contains_key(name)
                || self
                    .functions
                    .contains_key(QualifiedName::new(name).alternate().as_ref())
            {
                Ok(Type::Function {
                    return_type: Box::new(Type::Int),
                    param_types: Vec::new(),
                })
            } else {
                Err(CompileError::new(
                    format!("Undeclared variable or identifier '{}'", name),
                    self.current_line,
                    self.current_col,
                ))
            }
        } else if let Some(sigs) = self.functions.get(name) {
            Ok(Type::Pointer(Box::new(Type::Function {
                return_type: Box::new(sigs[0].return_type.clone()),
                param_types: sigs[0].param_types.clone(),
            })))
        } else {
            Err(CompileError::new(
                format!("Undeclared variable or identifier '{}'", name),
                self.current_line,
                self.current_col,
            ))
        }
    }

    fn analyze_global_identifier(&self, raw: &str, name: &str) -> Result<Type, CompileError> {
        if let Some(var) = self.global_vars.get(name) {
            Ok(var.ty.clone())
        } else if let Some(var) = self.lookup_var(name) {
            Ok(var.ty.clone())
        } else if let Some(sigs) = self.functions.get(name) {
            Ok(Type::Pointer(Box::new(Type::Function {
                return_type: Box::new(sigs[0].return_type.clone()),
                param_types: sigs[0].param_types.clone(),
            })))
        } else {
            Err(CompileError::new(
                format!("Undeclared variable or identifier '{}'", raw),
                self.current_line,
                self.current_col,
            ))
        }
    }

    pub(super) fn analyze_initializer(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        match expr {
            Expr::InitializerList { elements, .. } => {
                for item in elements {
                    self.analyze_expr(item)?;
                }
                Ok(Type::Void)
            }
            Expr::TypedInitList { ty, elements, pos } => {
                let explicit_ty = ty.clone();
                let p = *pos;
                self.resolve_list_init(&explicit_ty, elements, InitContext::DirectList, p)
            }
            _ => Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            )),
        }
    }
}
