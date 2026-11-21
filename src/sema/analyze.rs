use super::analyze_helpers::{collect_object_use_classes, method_matches_out_of_class};
use super::*;

impl Sema {
    pub fn analyze_program(&mut self, program: &mut Program) -> Result<(), CompileError> {
        self.object_use_classes = collect_object_use_classes(program);
        self.attach_out_of_class_methods(program);
        self.seed_lifecycle_types(program);
        self.seed_global_symbols(program);
        self.register_types(program)?;
        self.synthesize_destructors(program);
        self.synthesize_constructors(program);
        self.register_declarations(program)?;
        self.analyze_field_initializers(program)?;
        // Namespace aliases can name nested members selected through a
        // `decltype` pattern. Resolve them after methods and function templates
        // are registered; confirmation needs that lookup context.
        self.validate_namespace_aliases(program)?;
        self.validate_virtual_returns(program)?;
        self.analyze_declarations(program)?;
        // Bodies deferred during early template instantiation are retried now
        // that globals and functions are registered. A permanent error here
        // aborts before code generation.
        self.retry_deferred_methods()?;
        program.declarations.append(&mut self.synthesized_decls);
        // Class-template specializations are synthesized after the first
        // initializer pass. Analyze their NSDMIs now so unqualified member
        // calls bind to the specialization's methods before codegen.
        self.analyze_field_initializers(program)?;
        // Member calls in those NSDMIs can instantiate pending methods into
        // `synthesized_decls`; surface them to the program for codegen.
        program.declarations.append(&mut self.synthesized_decls);
        Ok(())
    }

    fn attach_out_of_class_methods(&mut self, program: &mut Program) {
        for i in 0..program.declarations.len() {
            let (plain_class, short_method, params, is_const) = match &program.declarations[i] {
                Decl::Function(f) if f.name.contains("::") && f.body.is_some() => {
                    let Some((class_part, method_name)) = f.name.rsplit_once("::") else {
                        continue;
                    };
                    (
                        QualifiedName::new(class_part).plain().to_string(),
                        method_name.to_string(),
                        f.params.clone(),
                        f.is_const,
                    )
                }
                _ => continue,
            };
            let mut target_idx = None;
            for (j, decl) in program.declarations.iter().enumerate() {
                if i != j
                    && let Decl::StructDecl { name, methods, .. } = decl
                    && QualifiedName::new(name).plain() == plain_class
                    && methods
                        .iter()
                        .any(|m| method_matches_out_of_class(m, &short_method, &params, is_const))
                {
                    target_idx = Some(j);
                    break;
                }
            }
            let Some(j) = target_idx else {
                continue;
            };
            let (body, inits) = match &mut program.declarations[i] {
                Decl::Function(f) => {
                    f.name.clear();
                    (f.body.take(), std::mem::take(&mut f.ctor_initializers))
                }
                _ => continue,
            };
            if let Decl::StructDecl { methods, .. } = &mut program.declarations[j]
                && let Some(existing) = methods
                    .iter_mut()
                    .find(|m| method_matches_out_of_class(m, &short_method, &params, is_const))
            {
                existing.body = body;
                if !inits.is_empty() {
                    existing.ctor_initializers = inits;
                }
            }
        }
        program.declarations.retain(|d| match d {
            Decl::Function(f) => !f.name.is_empty(),
            _ => true,
        });
    }

    /// Registers global symbols before type expressions are resolved, so a
    /// `decltype` operand can name a global in a field or parameter type. The
    /// declared type is refined later by `analyze_global_var`.
    fn seed_global_symbols(&mut self, program: &Program) {
        for decl in &program.declarations {
            let Decl::GlobalVar { name, ty, .. } = decl else {
                continue;
            };
            let symbol = VarSymbol {
                name: name.clone(),
                ty: ty.clone(),
                is_global: true,
                stack_offset: 0,
            };
            self.global_vars.insert(name.clone(), symbol.clone());
            if let Some(alias) = QualifiedName::new(name).implicit_alias() {
                self.global_vars.insert(alias.into_owned(), symbol);
            }
        }
    }

    fn register_types(&mut self, program: &mut Program) -> Result<(), CompileError> {
        self.type_aliases = program.type_aliases.clone();
        for decl in &mut program.declarations {
            match decl {
                Decl::EnumDecl {
                    name,
                    is_scoped,
                    variants,
                } => self.register_enum_decl(name, *is_scoped, variants),
                Decl::StructDecl {
                    name,
                    is_class,
                    is_final,
                    base_classes,
                    base_access,
                    fields,
                    methods,
                    private_members,
                    alignment,
                    line,
                    col,
                    ..
                } => {
                    for (_, fty, _) in fields.iter_mut() {
                        self.resolve_and_instantiate_type(fty, *line, *col)?;
                    }
                    // NSDMI expressions are analyzed after member declarations
                    // are registered; see analyze_field_initializers.
                    // A base that is a template specialization must have a
                    // layout before this class can inherit its vptr and slots.
                    for base in base_classes.clone() {
                        let canonical = self.canonical_class_name(&base, Some(name));
                        let _ = self.ensure_class_instantiated(&canonical, *line, *col);
                    }
                    self.compute_struct_layout(RecordLayout {
                        name: name.as_str(),
                        is_class: *is_class,
                        is_final: *is_final,
                        base_classes,
                        base_access,
                        fields,
                        methods,
                        private_members,
                        line: *line,
                        col: *col,
                        alignment: alignment
                            .as_ref()
                            .map(|ty| ty.align_of(&self.struct_aligns)),
                    })?;
                }
                Decl::Template(t) => self.register_template_declaration(t)?,
                _ => {}
            }
        }
        Ok(())
    }

    /// Namespace-scope aliases are parsed straight into `type_aliases` and
    /// never pass through signature analysis. Resolve them once the full lookup
    /// context is registered so an invalid qualified member access is a hard
    /// error here, while dependent forms stay for substitution.
    fn validate_namespace_aliases(&mut self, program: &Program) -> Result<(), CompileError> {
        // Resolve in source order: the instantiation order of alias targets
        // flows into emitted code, so HashMap iteration must not decide it.
        let mut aliases: Vec<(String, Type, (usize, usize))> = program
            .namespace_type_aliases
            .iter()
            .filter_map(|(name, pos)| {
                self.type_aliases
                    .get(name)
                    .map(|ty| (name.clone(), ty.clone(), *pos))
            })
            .collect();
        aliases.sort_by_key(|(_, _, pos)| *pos);
        for (name, mut ty, pos) in aliases {
            let visibility = self.alias_visibility.get(&name).cloned();
            let previous = std::mem::replace(&mut self.active_alias_visibility, visibility.clone());
            let result = self.resolve_and_instantiate_type(&mut ty, pos.0, pos.1);
            self.active_alias_visibility = previous;
            result?;
            if let Type::Decltype { .. } = self.type_aliases.get(&name).unwrap_or(&ty) {
                self.decltype_alias_context
                    .insert(name.clone(), visibility.unwrap_or_default());
            }
        }
        Ok(())
    }

    fn register_template_declaration(
        &mut self,
        t: &crate::ast::TemplateDecl,
    ) -> Result<(), CompileError> {
        match &t.kind {
            TemplateKind::Class(c) => {
                if t.params.is_empty() && c.name.contains('<') {
                    self.register_class_specialization(c.clone(), (t.line, t.col))?;
                } else if !t.params.is_empty() && c.name.contains('<') {
                    self.register_partial_specialization(
                        c.clone(),
                        t.params.clone(),
                        (t.line, t.col),
                    )?;
                } else {
                    self.register_class_template(c.clone(), t.params.clone(), (t.line, t.col))?;
                }
            }
            TemplateKind::Function(f) => {
                if f.name.contains("::") {
                    self.attach_out_of_class_template_method((**f).clone(), &t.params);
                } else {
                    self.register_function_template((**f).clone(), t.params.clone());
                }
            }
            TemplateKind::Alias(name, ty) => {
                self.register_alias_template(name.clone(), ty.clone(), t.params.clone());
            }
            TemplateKind::Variable(name, ty, init) => {
                let entry = (t.params.clone(), ty.clone(), init.clone());
                let plain = crate::QualifiedName::new(name).plain().to_string();
                self.variable_templates.insert(name.clone(), entry.clone());
                self.variable_templates.insert(plain, entry);
            }
        }
        Ok(())
    }

    /// Analyze default member initializers once the class's methods are
    /// registered. Field types and layouts are already resolved; only the
    /// initializer expressions and their stored layout copies are updated.
    fn analyze_field_initializers(&mut self, program: &mut Program) -> Result<(), CompileError> {
        for decl in &mut program.declarations {
            let Decl::StructDecl { name, fields, .. } = decl else {
                continue;
            };
            let visibility = self.class_field_visibility.get(name).cloned().or_else(|| {
                let plain = QualifiedName::new(name).plain();
                self.class_field_visibility.get(plain).cloned()
            });
            let class_name = name.clone();
            let field_count = fields.len();
            let previous_visibility =
                std::mem::replace(&mut self.active_alias_visibility, visibility);
            let previous_class = self.current_class.replace(class_name.clone());
            let result = (|| {
                for (fname, fty, finit) in fields.iter_mut() {
                    if let Some((init_expr, line, col)) = finit {
                        self.check_field_default_init(fname, fty, init_expr, (*line, *col))?;
                    }
                }
                Ok(())
            })();
            self.current_class = previous_class;
            self.active_alias_visibility = previous_visibility;
            result?;
            if let Some(layout) = self.struct_layouts.get_mut(&class_name) {
                let start = layout.fields.len().saturating_sub(field_count);
                for (slot, (_, _, finit)) in layout.fields[start..].iter_mut().zip(fields.iter()) {
                    slot.default_init = finit.as_ref().map(|(expr, _, _)| expr.clone());
                }
            }
        }
        Ok(())
    }

    fn analyze_declarations(&mut self, program: &mut Program) -> Result<(), CompileError> {
        // static_assert folds against every constexpr in the program; NTTP
        // folding below stays declaration-ordered via register_constexpr_global.
        let all_constants = self.collect_constexpr_globals(&program.declarations);
        for decl in &mut program.declarations {
            // A namespace-scope constant is visible only after its declaration
            // is reached; earlier constants are available to later initializers.
            self.register_constexpr_global(decl);
            match decl {
                Decl::GlobalVar {
                    name,
                    ty,
                    init,
                    line,
                    col,
                } => {
                    self.analyze_global_var(name.as_str(), ty, init, *line, *col)?;
                }
                Decl::Function(func) => {
                    self.current_line = func.line;
                    self.current_col = func.col;
                    self.analyze_function(func)?;
                }
                Decl::StaticAssert {
                    condition,
                    message,
                    line,
                    col,
                } => {
                    self.analyze_static_assert(condition, message, *line, *col, &all_constants)?;
                }
                Decl::StructDecl {
                    name,
                    methods,
                    static_fields,
                    base_classes,
                    ..
                } => {
                    self.analyze_struct_decl(name, methods, static_fields, base_classes)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Returns whether an unevaluated expression in a static_assert consists
    /// exclusively of constant operands (constexpr globals, constants, literals).
    fn expr_is_constant_candidate(&self, expr: &Expr) -> bool {
        match expr {
            Expr::IntLiteral(..)
            | Expr::FloatLiteral(..)
            | Expr::BoolLiteral(..)
            | Expr::CharLiteral(..)
            | Expr::StringLiteral(..) => true,
            Expr::Identifier(name) => {
                if let Some(var) = self.global_vars.get(name) {
                    var.ty.is_const()
                } else {
                    self.enum_values.contains_key(name)
                        || self.static_const_values.contains_key(name)
                }
            }
            Expr::Binary { left, right, .. } => {
                self.expr_is_constant_candidate(left) && self.expr_is_constant_candidate(right)
            }
            Expr::Unary { expr, .. } => self.expr_is_constant_candidate(expr),
            Expr::SizeofExpr(..) | Expr::SizeofType(..) | Expr::AlignofType(..) => true,
            Expr::Ternary {
                cond,
                then_expr,
                else_expr,
            } => {
                self.expr_is_constant_candidate(cond)
                    && self.expr_is_constant_candidate(then_expr)
                    && self.expr_is_constant_candidate(else_expr)
            }
            _ => false,
        }
    }

    fn analyze_static_assert(
        &mut self,
        condition: &Expr,
        message: &Option<String>,
        line: usize,
        col: usize,
        all_constants: &HashMap<String, i128>,
    ) -> Result<(), CompileError> {
        let value = crate::sema::constexpr_eval::eval_constexpr_with_context(
            condition,
            Some(&self.struct_sizes),
            Some(all_constants),
            None,
        );
        // The pre-pass snapshot misses class static members folded
        // later and variable templates not yet instantiated. Render
        // the condition so those references can be folded into the
        // live constant map before trying again.
        let value = if value.is_ok() {
            value
        } else {
            match crate::parser::types_array::expr_to_dependent_str(condition)
                .and_then(|spelling| self.eval_dependent_nontype(&spelling))
            {
                Some(resolved) => Ok(resolved),
                None => value,
            }
        };
        // Point at the condition, not the `static_assert` keyword, so the
        // diagnostic names the expression that failed.
        let pos = condition.pos();
        let (error_line, error_col) = if pos.line == 0 {
            (line, col)
        } else {
            (pos.line, pos.col)
        };
        match value {
            Ok(0) => {
                let suffix = message
                    .as_ref()
                    .map(|m| format!(": {m}"))
                    .unwrap_or_default();
                Err(CompileError::new(
                    format!("static assertion failed{suffix}"),
                    error_line,
                    error_col,
                ))
            }
            Ok(_) => Ok(()),
            Err(_) => {
                if self.expr_is_constant_candidate(condition) {
                    Ok(())
                } else {
                    Err(CompileError::new(
                        "static assertion condition is not a constant expression",
                        error_line,
                        error_col,
                    ))
                }
            }
        }
    }

    fn analyze_struct_decl(
        &mut self,
        name: &str,
        methods: &mut [FunctionDecl],
        static_fields: &[(String, Type, Option<Expr>)],
        base_classes: &[String],
    ) -> Result<(), CompileError> {
        self.current_class = Some(name.to_string());
        // A static const integral member of a plain class can be
        // used as a non-type template argument. Fold it here, once
        // earlier namespace constants are in scope.
        for sf in static_fields {
            self.fold_static_const_member(name, sf);
        }
        // Inherited constants are reachable through the derived
        // class name, as for class-template specializations.
        for base in base_classes {
            self.alias_static_fields(base, name);
        }
        for m in methods {
            if !m.template_params.is_empty() {
                continue;
            }
            let mut func_decl = m.clone();
            if !m.is_static {
                func_decl.params.insert(0, this_param(name, m.is_const));
            }
            self.analyze_function(&mut func_decl)?;
            m.body = func_decl.body;
            m.return_type = func_decl.return_type;
            self.update_registered_method_return_type(name, m);
        }
        self.current_class = None;
        Ok(())
    }

    /// Folds one const static data member into `static_const_values` under
    /// `Owner::member` when its initializer is a valid integral constant
    /// expression. Shared by plain classes and class-template specializations.
    pub(super) fn fold_static_const_member(
        &mut self,
        owner: &str,
        sf: &(String, Type, Option<Expr>),
    ) {
        if !sf.1.is_const() {
            return;
        }
        let Some(init) = &sf.2 else {
            return;
        };
        let mut init_clone = init.clone();
        let _ = self.analyze_expr(&mut init_clone);
        let constants = self.static_const_values.clone();
        if let Ok(value) = super::constexpr_eval::eval_constexpr_with_context(
            &init_clone,
            Some(&self.struct_sizes),
            Some(&constants),
            None,
        ) {
            self.static_const_values
                .insert(format!("{}::{}", owner, sf.0), value);
        }
    }
}

/// Builds the implicit `this` parameter of a non-static method.
pub(super) fn this_param(class: &str, is_const: bool) -> Param {
    let inner = if is_const {
        Type::Const(Box::new(Type::Struct(class.to_string())))
    } else {
        Type::Struct(class.to_string())
    };
    Param {
        name: "this".to_string(),
        ty: Type::Pointer(Box::new(inner)),
        default_val: None,
        is_pack: false,
    }
}
