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
            Expr::StringLiteral(_) => Ok(Type::Pointer(Box::new(Type::Char))),
            Expr::This => {
                if let Some(cls) = &self.current_class {
                    Ok(Type::Pointer(Box::new(Type::Struct(cls.clone()))))
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
            Expr::Cast { target_type, expr } => {
                self.analyze_expr(expr)?;
                Ok(target_type.clone())
            }
            Expr::SizeofType(_) | Expr::SizeofExpr(_) => Ok(Type::UnsignedInt),
            Expr::InitializerList { .. } => self.analyze_initializer(expr),
        }
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
        if *is_array {
            if let Some(s) = array_size {
                self.analyze_expr(s)?;
            }
            Ok(Type::Pointer(Box::new(ty.clone())))
        } else {
            for a in args {
                self.analyze_expr(a)?;
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

        if let Some(global_name) = name.strip_prefix("::") {
            if let Some(var) = self.global_vars.get(global_name) {
                return Ok(var.ty.clone());
            } else if let Some(var) = self.lookup_var(global_name) {
                return Ok(var.ty.clone());
            } else if let Some(sigs) = self.functions.get(global_name) {
                return Ok(Type::Pointer(Box::new(Type::Function {
                    return_type: Box::new(sigs[0].return_type.clone()),
                    param_types: sigs[0].param_types.clone(),
                })));
            } else {
                return Err(CompileError::new(
                    format!("Undeclared variable or identifier '{}'", name),
                    self.current_line,
                    self.current_col,
                ));
            }
        }
        self.check_ambiguous_name(name)?;
        if let Some(var) = self.lookup_var(name) {
            Ok(var.ty.clone())
        } else if let Some(&val) = self.enum_values.get(name) {
            *expr = Expr::IntLiteral(val, Type::Int);
            Ok(Type::Int)
        } else if let Some(cls) = &self.current_class {
            if let Some(fld) = self.find_field_in_class(cls, name) {
                *expr = Expr::MemberAccess {
                    object: Box::new(Expr::This),
                    member: name.to_string(),
                    is_arrow: true,
                    pos: ExprPos::new(self.current_line, self.current_col),
                };
                Ok(fld.ty.clone())
            } else if let Some(var) = self.lookup_var(&format!("{}::{}", cls, name)) {
                Ok(var.ty.clone())
            } else if self.find_method_in_class(cls, name).is_some()
                || self.functions.contains_key(name)
                || self
                    .functions
                    .contains_key(QualifiedName::new(name).std_name().as_ref())
                || self
                    .functions
                    .contains_key(QualifiedName::new(name).plain())
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

    pub(super) fn analyze_initializer(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::InitializerList(list) = expr else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        for item in list {
            self.analyze_expr(item)?;
        }
        Ok(Type::Void)
    }
}
