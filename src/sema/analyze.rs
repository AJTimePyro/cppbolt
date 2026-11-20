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
        self.validate_virtual_returns(program)?;
        self.analyze_declarations(program)?;
        // Bodies deferred during early template instantiation are retried now
        // that globals and functions are registered. A permanent error here
        // aborts before code generation.
        self.retry_deferred_methods()?;
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
                    for (fname, fty, finit) in fields.iter_mut() {
                        self.resolve_and_instantiate_type(fty, *line, *col)?;
                        if let Some((init_expr, fline, fcol)) = finit {
                            self.check_field_default_init(fname, fty, init_expr, (*fline, *fcol))?;
                        }
                    }
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
        // Namespace-scope aliases are parsed straight into `type_aliases` and
        // never pass through signature analysis. Resolve them now that class
        // templates are registered so an invalid qualified member access is a
        // hard error here, while dependent forms stay for substitution.
        let aliases: Vec<Type> = program
            .namespace_type_aliases
            .iter()
            .filter_map(|name| self.type_aliases.get(name).cloned())
            .collect();
        for mut ty in aliases {
            self.resolve_and_instantiate_type(&mut ty, 0, 0)?;
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

    fn register_declarations(&mut self, program: &mut Program) -> Result<(), CompileError> {
        let mut defined_functions = std::collections::HashSet::new();
        let mut visible_usings: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        // Track the overloads visible at each source position. Function
        // templates snapshot this set so their bodies keep definition-time
        // ordinary bindings when instantiated later in the file.
        let mut visible_functions: std::collections::HashSet<String> = self
            .functions
            .values()
            .flatten()
            .map(|sig| sig.mangled_name.clone())
            .collect();
        for decl in &mut program.declarations {
            match decl {
                Decl::UsingNamespace(ns) => {
                    let resolved = self.resolve_alias_in_qualified_name(ns);
                    self.open_namespaces.push(resolved);
                }
                Decl::UsingDecl(name) => {
                    let resolved = self.resolve_alias_in_qualified_name(name);
                    let short_name = resolved.split("::").last().unwrap_or(&resolved).to_string();
                    visible_usings.insert(short_name.clone());
                    self.using_decls.insert(short_name, resolved);
                }
                Decl::NamespaceAlias {
                    alias,
                    target,
                    line,
                    col,
                } => {
                    self.record_local_namespace_alias(alias, target, *line, *col)?;
                }
                Decl::Function(f) => {
                    self.resolve_and_instantiate_type(&mut f.return_type, f.line, f.col)?;
                    for p in &mut f.params {
                        self.resolve_and_instantiate_type(&mut p.ty, f.line, f.col)?;
                    }
                    let mangled = self.mangle_name(&f.name, &f.params);
                    if f.body.is_some() && !defined_functions.insert(mangled.clone()) {
                        return Err(CompileError::new(
                            format!("Redefinition of function '{}'", f.name),
                            f.line,
                            f.col,
                        ));
                    }
                    let sig = FunctionSig {
                        qualifiers: FunctionQualifiers::default(),
                        name: f.name.clone(),
                        mangled_name: mangled,
                        return_type: f.return_type.clone(),
                        param_types: f.params.iter().map(|p| p.ty.clone()).collect(),
                        default_args: f.params.iter().map(|p| p.default_val.clone()).collect(),
                        is_static: false,
                        is_explicit: f.is_explicit,
                        ref_qualifier: f.ref_qualifier,
                    };
                    visible_functions.insert(sig.mangled_name.clone());
                    self.register_function(sig);
                }
                Decl::FriendFunction { class_name, func } => {
                    if func.template_params.is_empty() {
                        let (mangled, f) =
                            self.register_friend_function(class_name, func.clone())?;
                        visible_functions.insert(mangled);
                        *decl = Decl::Function(f);
                    } else {
                        // A friend template has no body to emit until a call
                        // deduces its parameters. Leave the declaration in
                        // place and expose it only through ADL.
                        self.register_hidden_friend_template(
                            class_name,
                            func.clone(),
                            func.template_params.clone(),
                        );
                    }
                }
                Decl::Template(t) => match &t.kind {
                    crate::ast::template::TemplateKind::Function(f) => {
                        let identity = super::template_registry::function_template_identity(
                            &f.name, f.line, f.col,
                        );
                        // A template can call itself, so it sees its own identity.
                        visible_functions.insert(format!("T:{identity}"));
                        let positions = super::template_call_dependence::dependent_call_positions(
                            f, &t.params, None,
                        );
                        self.dependent_calls.insert(identity.clone(), positions);
                        self.template_definition_usings
                            .insert(identity.clone(), visible_usings.clone());
                        self.template_definition_namespaces
                            .insert(identity.clone(), self.open_namespaces.clone());
                        self.template_definition_functions
                            .insert(identity, visible_functions.clone());
                    }
                    crate::ast::template::TemplateKind::Class(c) => {
                        for method in &c.methods {
                            let identity = super::template_registry::function_template_identity(
                                &method.name,
                                method.line,
                                method.col,
                            );
                            let positions =
                                super::template_call_dependence::dependent_call_positions(
                                    method,
                                    &t.params,
                                    Some(&c.name),
                                );
                            self.dependent_calls.insert(identity.clone(), positions);
                            self.template_definition_usings
                                .insert(identity.clone(), visible_usings.clone());
                            self.template_definition_namespaces
                                .insert(identity.clone(), self.open_namespaces.clone());
                            self.template_definition_functions
                                .insert(identity, visible_functions.clone());
                        }
                    }
                    _ => {}
                },
                Decl::StructDecl {
                    name,
                    methods,
                    static_fields,
                    ..
                } => {
                    for method in methods.iter() {
                        let identity = super::template_registry::function_template_identity(
                            &method.name,
                            method.line,
                            method.col,
                        );
                        visible_functions.insert(format!("T:{identity}"));
                        let positions = super::template_call_dependence::dependent_call_positions(
                            method,
                            &method.template_params,
                            None,
                        );
                        self.dependent_calls.insert(identity.clone(), positions);
                        self.template_definition_usings
                            .insert(identity.clone(), visible_usings.clone());
                        self.template_definition_namespaces
                            .insert(identity.clone(), self.open_namespaces.clone());
                        self.template_definition_functions
                            .insert(identity, visible_functions.clone());
                    }
                    self.register_struct_declarations(name.as_str(), methods, static_fields)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Registers a friend function template for ADL on one class. The dedup
    /// keeps repeated declarations and instantiations from adding the same
    /// candidate twice.
    pub(super) fn register_hidden_friend_template(
        &mut self,
        class_name: &str,
        decl: FunctionDecl,
        params: Vec<TemplateParam>,
    ) {
        let plain = QualifiedName::new(class_name).plain().to_string();
        for key in [class_name.to_string(), plain] {
            let list = self.hidden_friend_templates.entry(key).or_default();
            if !list
                .iter()
                .any(|(d, _)| d.params == decl.params && d.return_type == decl.return_type)
            {
                list.push((decl.clone(), params.clone()));
            }
        }
    }

    /// Registers one friend function under its mangled name and associates it
    /// with the given class for ADL. Returns the mangled name and the resolved
    /// declaration so the caller can emit its body.
    pub(super) fn register_friend_function(
        &mut self,
        class_name: &str,
        mut func: FunctionDecl,
    ) -> Result<(String, FunctionDecl), CompileError> {
        self.resolve_and_instantiate_type(&mut func.return_type, func.line, func.col)?;
        for p in &mut func.params {
            self.resolve_and_instantiate_type(&mut p.ty, func.line, func.col)?;
        }
        let mangled = self.mangle_name(&func.name, &func.params);
        let sig = FunctionSig {
            qualifiers: FunctionQualifiers::default(),
            name: func.name.clone(),
            mangled_name: mangled.clone(),
            return_type: func.return_type.clone(),
            param_types: func.params.iter().map(|p| p.ty.clone()).collect(),
            default_args: func.params.iter().map(|p| p.default_val.clone()).collect(),
            is_static: false,
            is_explicit: false,
            ref_qualifier: func.ref_qualifier,
        };
        // Only the mangled spelling is reachable by ordinary lookup; the
        // source name is reserved for ADL.
        self.functions
            .entry(mangled.clone())
            .or_default()
            .push(sig.clone());
        let plain = crate::QualifiedName::new(class_name).plain().to_string();
        let keys = if plain == class_name {
            vec![class_name.to_string()]
        } else {
            vec![class_name.to_string(), plain]
        };
        for key in keys {
            self.hidden_friends
                .entry(key)
                .or_default()
                .push(sig.clone());
        }
        Ok((mangled, func))
    }

    /// Registers the static fields and methods of a class.
    ///
    /// Non-template method signatures are completed like free function
    /// signatures, so template types in their parameter and return types are
    /// instantiated before overload lookup sees them.
    fn register_struct_declarations(
        &mut self,
        name: &str,
        methods: &mut [FunctionDecl],
        static_fields: &[(String, Type, Option<Expr>)],
    ) -> Result<(), CompileError> {
        for sf in static_fields {
            let qual_name = format!("{}::{}", name, sf.0);
            let sym = VarSymbol {
                name: qual_name.clone(),
                ty: sf.1.clone(),
                is_global: true,
                stack_offset: 0,
            };
            self.global_vars.insert(qual_name.clone(), sym.clone());
            if let Some(alias) = QualifiedName::new(&qual_name).implicit_alias() {
                self.global_vars.insert(alias.to_string(), sym);
            }
        }
        let previous_class = self.current_class.replace(name.to_string());
        for m in methods.iter_mut() {
            if !m.template_params.is_empty() {
                self.member_templates
                    .entry(name.to_string())
                    .or_default()
                    .push(m.clone());
                let plain = QualifiedName::new(name).plain();
                if plain != name {
                    self.member_templates
                        .entry(plain.to_string())
                        .or_default()
                        .push(m.clone());
                }
                continue;
            }
            self.resolve_and_instantiate_type(&mut m.return_type, m.line, m.col)?;
            for param in &mut m.params {
                self.resolve_and_instantiate_type(&mut param.ty, m.line, m.col)?;
            }
            self.register_struct_method(name, m);
        }
        self.current_class = previous_class;
        Ok(())
    }

    pub(super) fn register_struct_method(&mut self, class_name: &str, m: &FunctionDecl) {
        let mut params = m.params.clone();
        if !m.is_static {
            params.insert(0, this_param(class_name, m.is_const));
        }
        let qual_name = format!("{}::{}", class_name, m.name);
        let mut mangled = self.mangle_name(&qual_name, &params);
        match m.ref_qualifier {
            RefQualifier::LValue => mangled.push_str("_R"),
            RefQualifier::RValue => mangled.push_str("_O"),
            RefQualifier::None => {}
        }
        let mut quals = m.qualifiers;
        quals.is_const = m.is_const;
        let sig = FunctionSig {
            qualifiers: quals,
            name: m.name.clone(),
            mangled_name: mangled,
            return_type: m.return_type.clone(),
            param_types: params.iter().map(|p| p.ty.clone()).collect(),
            default_args: params.iter().map(|p| p.default_val.clone()).collect(),
            is_static: m.is_static,
            is_explicit: m.is_explicit,
            ref_qualifier: m.ref_qualifier,
        };
        self.register_function(sig.clone());
        self.functions
            .entry(qual_name)
            .or_default()
            .push(sig.clone());
        let short = QualifiedName::new(class_name).class_short_name();
        if m.name == class_name {
            let short_qual = format!("{}::{}", class_name, short);
            self.functions.entry(short_qual).or_default().push(sig);
        } else if m.name == format!("~{}", class_name) {
            let short_qual = format!("{}::~{}", class_name, short);
            self.functions.entry(short_qual).or_default().push(sig);
        }
    }

    /// Rejects abstract variable types and completes a zero-length array type
    /// from an initializer list.
    pub(super) fn check_and_complete_var_type(
        &self,
        ty: &mut Type,
        init: &Option<Expr>,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        if let Type::Struct(sname) = ty.strip_const()
            && let Some(layout) = self.struct_layouts.get(sname)
            && layout.is_abstract
        {
            return Err(CompileError::new(
                format!("Variable type '{}' is an abstract class", sname),
                line,
                col,
            ));
        }
        if let Type::Array(elem, crate::ast::types::UNBOUNDED_ARRAY) = ty.clone()
            && let Some(
                Expr::InitializerList { elements: list, .. }
                | Expr::TypedInitList { elements: list, .. },
            ) = init
        {
            *ty = Type::Array(elem, list.len());
        }
        Ok(())
    }

    /// Checks and records a global variable and its initializer.
    fn analyze_global_var(
        &mut self,
        name: &str,
        ty: &mut Type,
        init: &mut Option<Expr>,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        self.current_line = line;
        self.current_col = col;
        self.resolve_and_instantiate_type(ty, line, col)?;
        self.check_and_complete_var_type(ty, init, line, col)?;
        if let Some(init_expr) = init {
            let init_ty = match init_expr {
                Expr::InitializerList { .. } | Expr::TypedInitList { .. } => {
                    self.resolve_var_init_type(ty, init_expr)?.0
                }
                _ => self.analyze_expr_keeping_reference(init_expr)?,
            };
            if !self.can_assign_expr(ty, &init_ty, Some(init_expr)) {
                return Err(CompileError::new(
                    format!(
                        "Type mismatch in global '{}': cannot assign {} to {}",
                        name,
                        init_ty.to_c_type(),
                        ty.to_c_type()
                    ),
                    line,
                    col,
                ));
            }
        }
        self.global_vars.insert(
            name.to_string(),
            VarSymbol {
                name: name.to_string(),
                ty: ty.clone(),
                is_global: true,
                stack_offset: 0,
            },
        );
        if let Some(alias) = QualifiedName::new(name).implicit_alias() {
            self.global_vars.insert(
                alias.to_string(),
                VarSymbol {
                    name: alias.to_string(),
                    ty: ty.clone(),
                    is_global: true,
                    stack_offset: 0,
                },
            );
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
                    let value = crate::sema::constexpr_eval::eval_constexpr_with_context(
                        condition,
                        Some(&self.struct_sizes),
                        Some(&all_constants),
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
                    // Fail on a definite false. An unevaluable condition that
                    // is not even well-formed is rejected below; a well-formed
                    // one (for example a member-pointer comparison) keeps the
                    // existing accepted behavior.
                    if matches!(value, Ok(0)) {
                        let suffix = message
                            .as_ref()
                            .map(|m| format!(": {m}"))
                            .unwrap_or_default();
                        return Err(CompileError::new(
                            format!("static assertion failed{suffix}"),
                            *line,
                            *col,
                        ));
                    }
                    // If the condition is not even well-formed, its truth
                    // cannot be established; reject instead of accepting it.
                    // A well-formed but unevaluable condition (for example a
                    // member-pointer comparison) keeps the accepted behavior.
                    if value.is_err() {
                        let mut probe = self.clone();
                        let mut condition_copy = condition.clone();
                        if probe.analyze_expr(&mut condition_copy).is_err() {
                            return Err(CompileError::new(
                                "static assertion condition is not a constant expression",
                                *line,
                                *col,
                            ));
                        }
                    }
                }
                Decl::StructDecl {
                    name,
                    methods,
                    static_fields,
                    base_classes,
                    ..
                } => {
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
                }
                _ => {}
            }
        }
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
