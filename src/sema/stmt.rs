use super::*;

impl Sema {
    pub(super) fn analyze_function(&mut self, func: &mut FunctionDecl) -> Result<(), CompileError> {
        self.current_func_return_type = Some(func.return_type.clone());
        self.current_func_name = Some(func.name.clone());
        self.current_frame_size = 0;
        self.enter_scope();

        // Register parameters in function scope
        let mut param_offset = 0;
        for param in &func.params {
            param_offset += 4;
            let sym = VarSymbol {
                name: param.name.clone(),
                ty: param.ty.clone(),
                is_global: false,
                stack_offset: param_offset,
            };
            let Some(scope) = self.scopes.last_mut() else {
                return Err(CompileError::new(
                    "internal compiler error: missing function scope",
                    func.line,
                    func.col,
                )
                .with_kind(crate::diagnostics::ErrorKind::Internal));
            };
            scope.vars.insert(param.name.clone(), sym);
        }

        if let Some(body) = &mut func.body {
            self.analyze_stmt(body)?;
        }

        self.exit_scope();
        self.current_func_return_type = None;
        self.current_func_name = None;
        Ok(())
    }

    pub(super) fn analyze_stmt(&mut self, stmt: &mut Stmt) -> Result<(), CompileError> {
        let _guard = self.depth.enter(stmt.line, stmt.col)?;
        self.current_line = stmt.line;
        self.current_col = stmt.col;
        match &mut stmt.kind {
            StmtKind::UsingNamespace(ns) => {
                self.open_namespaces.push(ns.clone());
            }
            StmtKind::UsingDecl(name) => {
                let short = name.split("::").last().unwrap_or(name).to_string();
                self.using_decls.insert(short, name.clone());
            }
            StmtKind::Compound(stmts) => {
                self.enter_scope();
                for s in stmts {
                    self.analyze_stmt(s)?;
                }
                self.exit_scope();
            }
            StmtKind::VarDecl {
                name,
                ty,
                init,
                is_static,
            } => {
                self.current_line = stmt.line;
                self.current_col = stmt.col;
                if let Type::Struct(sname) = ty.strip_const()
                    && let Some(layout) = self.struct_layouts.get(sname)
                    && layout.is_abstract
                {
                    return Err(CompileError::new(
                        format!("Variable type '{}' is an abstract class", sname),
                        stmt.line,
                        stmt.col,
                    ));
                }
                if let Some(scope) = self.scopes.last()
                    && scope.vars.contains_key(name)
                {
                    return Err(CompileError::new(
                        format!("Redefinition of variable '{}'", name),
                        stmt.line,
                        stmt.col,
                    ));
                }
                if let Type::Array(elem, 0) = ty.clone()
                    && let Some(Expr::InitializerList(list)) = init
                {
                    *ty = Type::Array(elem, list.len());
                }
                if let Some(init_expr) = init {
                    let init_ty = self.analyze_expr(init_expr)?;
                    if ty == &Type::Auto {
                        *ty = init_ty.clone();
                    } else if let Type::Pointer(inner) = ty
                        && **inner == Type::Auto
                    {
                        *ty = Type::Pointer(Box::new(
                            init_ty.deref_type().cloned().unwrap_or(init_ty.clone()),
                        ));
                    }
                    if !self.can_assign(ty, &init_ty) {
                        return Err(CompileError::new(
                            format!(
                                "cannot initialize variable '{}' of type {} with {}",
                                name,
                                ty.to_c_type(),
                                init_ty.to_c_type()
                            ),
                            stmt.line,
                            stmt.col,
                        ));
                    }
                }

                if *is_static {
                    let function = self.current_func_name.as_deref().unwrap_or("");
                    let global_name =
                        crate::name::static_local_name(function, stmt.line, stmt.col, name);
                    let sym = VarSymbol {
                        name: global_name.clone(),
                        ty: ty.clone(),
                        is_global: true,
                        stack_offset: 0,
                    };
                    self.global_vars.insert(global_name.clone(), sym.clone());
                    let Some(scope) = self.scopes.last_mut() else {
                        return Err(CompileError::new(
                            "internal compiler error: missing variable scope",
                            stmt.line,
                            stmt.col,
                        )
                        .with_kind(crate::diagnostics::ErrorKind::Internal));
                    };
                    scope.vars.insert(name.clone(), sym);
                } else {
                    let var_size = ty.size_of(&self.struct_sizes) as i32;
                    self.current_frame_size += (var_size + 3) & !3; // Align to 4
                    let sym = VarSymbol {
                        name: name.clone(),
                        ty: ty.clone(),
                        is_global: false,
                        stack_offset: -self.current_frame_size,
                    };
                    let Some(scope) = self.scopes.last_mut() else {
                        return Err(CompileError::new(
                            "internal compiler error: missing variable scope",
                            stmt.line,
                            stmt.col,
                        )
                        .with_kind(crate::diagnostics::ErrorKind::Internal));
                    };
                    scope.vars.insert(name.clone(), sym);
                }
            }
            StmtKind::Expr(expr) => {
                self.analyze_expr(expr)?;
            }
            StmtKind::If {
                branches,
                else_stmt,
            } => {
                for (cond, then_stmt) in branches {
                    self.analyze_expr(cond)?;
                    self.analyze_stmt(then_stmt)?;
                }
                if let Some(el) = else_stmt {
                    self.analyze_stmt(el)?;
                }
            }
            StmtKind::While { cond, body } => {
                self.analyze_expr(cond)?;
                self.loop_depth += 1;
                let res = self.analyze_stmt(body);
                self.loop_depth -= 1;
                res?;
            }
            StmtKind::DoWhile { body, cond } => {
                self.loop_depth += 1;
                let res = self.analyze_stmt(body);
                self.loop_depth -= 1;
                res?;
                self.analyze_expr(cond)?;
            }
            StmtKind::For {
                init,
                cond,
                step,
                body,
            } => {
                self.enter_scope();
                if let Some(init_s) = init {
                    self.analyze_stmt(init_s)?;
                }
                if let Some(cond_e) = cond {
                    self.analyze_expr(cond_e)?;
                }
                if let Some(step_e) = step {
                    self.analyze_expr(step_e)?;
                }
                self.loop_depth += 1;
                let res = self.analyze_stmt(body);
                self.loop_depth -= 1;
                self.exit_scope();
                res?;
            }
            StmtKind::Switch { expr, body } => {
                let ty = self.analyze_expr(expr)?;
                if !ty.is_integer() {
                    return Err(CompileError::new(
                        "Switch expression must be an integer",
                        stmt.line,
                        stmt.col,
                    ));
                }
                self.switch_depth += 1;
                let res = self.analyze_stmt(body);
                self.switch_depth -= 1;
                res?;
            }
            StmtKind::Break => {
                if self.loop_depth == 0 && self.switch_depth == 0 {
                    return Err(CompileError::new(
                        "'break' statement not in loop or switch statement",
                        stmt.line,
                        stmt.col,
                    ));
                }
            }
            StmtKind::Continue => {
                if self.loop_depth == 0 {
                    return Err(CompileError::new(
                        "'continue' statement not in loop statement",
                        stmt.line,
                        stmt.col,
                    ));
                }
            }
            StmtKind::Case(_) | StmtKind::Default | StmtKind::Empty => {}
            StmtKind::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let ret_ty = self.analyze_expr(expr)?;
                    if let Some(expected) = &self.current_func_return_type
                        && !self.can_assign(expected, &ret_ty)
                    {
                        return Err(CompileError::new(
                            format!(
                                "Return type mismatch: expected {}, found {}",
                                expected.to_c_type(),
                                ret_ty.to_c_type()
                            ),
                            stmt.line,
                            stmt.col,
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
