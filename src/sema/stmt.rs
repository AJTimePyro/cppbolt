use super::*;

impl Sema {
    pub(super) fn analyze_function(&mut self, func: &mut FunctionDecl) -> Result<(), CompileError> {
        self.current_func_return_type = Some(func.return_type.clone());
        self.current_func_name = Some(func.name.clone());
        self.current_frame_size = 0;
        self.enter_scope();

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
                return Err(crate::internal_error!(
                    func.line,
                    func.col,
                    "missing function scope"
                ));
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

    /// Analyzes one statement.
    ///
    /// Every statement kind is handled by its own helper so that this
    /// function stays a dispatcher.
    pub(super) fn analyze_stmt(&mut self, stmt: &mut Stmt) -> Result<(), CompileError> {
        // A compound block has its own budget, so a nested `if` with its
        // block costs one statement level and one block level.
        if let StmtKind::Compound(stmts) = &mut stmt.kind {
            let _guard = self.depth.blocks().enter(stmt.line, stmt.col)?;
            self.current_line = stmt.line;
            self.current_col = stmt.col;
            return self.analyze_compound(stmts);
        }
        let _guard = self.depth.statements().enter(stmt.line, stmt.col)?;
        self.current_line = stmt.line;
        self.current_col = stmt.col;
        if self.analyze_simple_stmt(&mut stmt.kind)? {
            return Ok(());
        }
        match &mut stmt.kind {
            StmtKind::Compound(stmts) => self.analyze_compound(stmts)?,
            StmtKind::VarDecl {
                name,
                ty,
                init,
                is_static,
            } => {
                self.analyze_var_decl(VarDecl {
                    name,
                    ty,
                    init,
                    is_static: *is_static,
                    line: stmt.line,
                    col: stmt.col,
                })?;
            }
            StmtKind::If { .. }
            | StmtKind::While { .. }
            | StmtKind::DoWhile { .. }
            | StmtKind::For { .. }
            | StmtKind::Switch { .. } => {
                self.analyze_flow_stmt(&mut stmt.kind, stmt.line, stmt.col)?;
            }
            StmtKind::Break => {
                self.analyze_break(stmt.line, stmt.col)?;
            }
            StmtKind::Continue => {
                self.analyze_continue(stmt.line, stmt.col)?;
            }
            StmtKind::Case(_)
            | StmtKind::Default
            | StmtKind::Empty
            | StmtKind::UsingNamespace(_)
            | StmtKind::UsingDecl(_)
            | StmtKind::Expr(_) => {}
            StmtKind::Return(expr_opt) => {
                self.analyze_return(expr_opt, stmt.line, stmt.col)?;
            }
        }
        Ok(())
    }

    /// Handles the control-flow statements.
    ///
    /// Returns true when the statement was handled here.
    fn analyze_flow_stmt(
        &mut self,
        kind: &mut StmtKind,
        line: usize,
        col: usize,
    ) -> Result<bool, CompileError> {
        match kind {
            StmtKind::If {
                branches,
                else_stmt,
            } => {
                self.analyze_if(branches, else_stmt)?;
            }
            StmtKind::While { cond, body } => {
                self.analyze_while(cond, body)?;
            }
            StmtKind::DoWhile { body, cond } => {
                self.analyze_do_while(body, cond)?;
            }
            StmtKind::For {
                init,
                cond,
                step,
                body,
            } => {
                self.analyze_for(init, cond, step, body)?;
            }
            StmtKind::Switch { expr, body } => {
                self.analyze_switch(expr, body, line, col)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// Handles the statements that need no dedicated analysis.
    ///
    /// Returns true when the statement was handled here.
    fn analyze_simple_stmt(&mut self, kind: &mut StmtKind) -> Result<bool, CompileError> {
        match kind {
            StmtKind::UsingNamespace(ns) => {
                self.open_namespaces.push(ns.clone());
            }
            StmtKind::UsingDecl(name) => {
                let short = name.split("::").last().unwrap_or(name).to_string();
                self.using_decls.insert(short, name.clone());
            }
            StmtKind::Expr(expr) => {
                self.analyze_expr(expr)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// Analyzes the statements of a compound statement.
    fn analyze_compound(&mut self, stmts: &mut [Stmt]) -> Result<(), CompileError> {
        self.enter_scope();
        for s in stmts {
            self.analyze_stmt(s)?;
        }
        self.exit_scope();
        Ok(())
    }

    /// Analyzes an `if` / `else if` chain.
    fn analyze_if(
        &mut self,
        branches: &mut Vec<(Expr, Box<Stmt>)>,
        else_stmt: &mut Option<Box<Stmt>>,
    ) -> Result<(), CompileError> {
        for (cond, then_stmt) in branches {
            self.analyze_expr(cond)?;
            self.analyze_stmt(then_stmt)?;
        }
        if let Some(el) = else_stmt {
            self.analyze_stmt(el)?;
        }
        Ok(())
    }

    /// Analyzes a `while` loop.
    fn analyze_while(&mut self, cond: &mut Expr, body: &mut Stmt) -> Result<(), CompileError> {
        self.analyze_expr(cond)?;
        self.loop_depth += 1;
        let res = self.analyze_stmt(body);
        self.loop_depth -= 1;
        res
    }

    /// Analyzes a `do` / `while` loop.
    fn analyze_do_while(&mut self, body: &mut Stmt, cond: &mut Expr) -> Result<(), CompileError> {
        self.loop_depth += 1;
        let res = self.analyze_stmt(body);
        self.loop_depth -= 1;
        res?;
        self.analyze_expr(cond)?;
        Ok(())
    }

    /// Analyzes a `for` loop.
    fn analyze_for(
        &mut self,
        init: &mut Option<Box<Stmt>>,
        cond: &mut Option<Expr>,
        step: &mut Option<Expr>,
        body: &mut Stmt,
    ) -> Result<(), CompileError> {
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
        res
    }

    /// Analyzes a `switch` statement and its body.
    fn analyze_switch(
        &mut self,
        expr: &mut Expr,
        body: &mut Stmt,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let ty = self.analyze_expr(expr)?;
        if !ty.is_integer() {
            return Err(CompileError::new(
                "Switch expression must be an integer",
                line,
                col,
            ));
        }
        self.switch_depth += 1;
        let res = self.analyze_stmt(body);
        self.switch_depth -= 1;
        res
    }

    /// Checks that a `break` appears inside a loop or a switch.
    fn analyze_break(&mut self, line: usize, col: usize) -> Result<(), CompileError> {
        if self.loop_depth == 0 && self.switch_depth == 0 {
            return Err(CompileError::new(
                "'break' statement not in loop or switch statement",
                line,
                col,
            ));
        }
        Ok(())
    }

    /// Checks that a `continue` appears inside a loop.
    fn analyze_continue(&mut self, line: usize, col: usize) -> Result<(), CompileError> {
        if self.loop_depth == 0 {
            return Err(CompileError::new(
                "'continue' statement not in loop statement",
                line,
                col,
            ));
        }
        Ok(())
    }

    /// Checks the returned expression against the current return type.
    fn analyze_return(
        &mut self,
        expr_opt: &mut Option<Expr>,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
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
                    line,
                    col,
                ));
            }
        }
        Ok(())
    }
}
