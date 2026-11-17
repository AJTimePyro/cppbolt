use super::constexpr_globals::collect_constexpr_globals;
use super::*;

fn method_matches_out_of_class(
    m: &FunctionDecl,
    short_method: &str,
    params: &[Param],
    is_const: bool,
) -> bool {
    (m.name == short_method || m.name == format!("~{}", short_method))
        && m.params.len() == params.len()
        && m.is_const == is_const
        && m.body.is_none()
        && m.params.iter().zip(params).all(|(p1, p2)| p1.ty == p2.ty)
}

impl Sema {
    pub fn analyze_program(&mut self, program: &mut Program) -> Result<(), CompileError> {
        self.attach_out_of_class_methods(program);
        self.register_types(program)?;
        self.synthesize_destructors(program);
        self.synthesize_constructors(program);
        self.register_declarations(program)?;
        self.validate_virtual_returns(program)?;
        self.analyze_declarations(program)?;
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

    fn register_types(&mut self, program: &mut Program) -> Result<(), CompileError> {
        self.type_aliases = program.type_aliases.clone();
        // Make class aliases refer to the same record layout as their target,
        // so lookups through either spelling see the same vptrs and slots.
        let aliases: Vec<(String, String)> = program
            .type_aliases
            .iter()
            .filter_map(|(alias, ty)| match ty {
                Type::Struct(name) => Some((alias.clone(), name.clone())),
                Type::TemplateInstance { name, args } => Some((
                    alias.clone(),
                    format!(
                        "{}<{}>",
                        name,
                        args.iter()
                            .map(|a| a.to_c_type())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )),
                _ => None,
            })
            .collect();
        for (alias, target) in aliases {
            if let Some(layout) = self.struct_layouts.get(&target).cloned() {
                self.struct_layouts.insert(alias, layout);
            }
        }
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
                        self.ensure_class_instantiated(&base, *line, *col);
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
                Decl::Template(t) => match &t.kind {
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
                            self.register_class_template(
                                c.clone(),
                                t.params.clone(),
                                (t.line, t.col),
                            )?;
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
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn register_declarations(&mut self, program: &mut Program) -> Result<(), CompileError> {
        let mut defined_functions = std::collections::HashSet::new();
        for decl in &mut program.declarations {
            match decl {
                Decl::UsingNamespace(ns) => {
                    let resolved = self.resolve_alias_in_qualified_name(ns);
                    self.open_namespaces.push(resolved);
                }
                Decl::UsingDecl(name) => {
                    let resolved = self.resolve_alias_in_qualified_name(name);
                    let short_name = resolved.split("::").last().unwrap_or(&resolved).to_string();
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
                    self.register_function(sig);
                }
                Decl::StructDecl {
                    name,
                    methods,
                    static_fields,
                    ..
                } => {
                    self.register_struct_declarations(name.as_str(), methods, static_fields)?;
                }
                _ => {}
            }
        }
        Ok(())
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
        if let Type::Array(elem, 0) = ty.clone()
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
        let constants = collect_constexpr_globals(&program.declarations, &self.struct_sizes);
        for decl in &mut program.declarations {
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
                        Some(&constants),
                        None,
                    );
                    // Fail on a definite false. When the integral evaluator
                    // cannot fold the condition (for example a comparison of
                    // member pointers), leave it accepted as before.
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
                }
                Decl::StructDecl { name, methods, .. } => {
                    self.current_class = Some(name.to_string());
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
