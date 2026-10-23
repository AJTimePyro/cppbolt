use super::*;

impl Sema {
    pub fn analyze_program(&mut self, program: &mut Program) -> Result<(), CompileError> {
        self.register_types(program)?;
        self.synthesize_destructors(program);
        self.synthesize_constructors(program);
        self.register_declarations(program)?;
        self.analyze_declarations(program)?;
        Ok(())
    }

    fn register_types(&mut self, program: &Program) -> Result<(), CompileError> {
        for decl in &program.declarations {
            match decl {
                Decl::EnumDecl { name: _, variants } => {
                    for (vname, val) in variants {
                        self.enum_values.insert(vname.clone(), *val);
                    }
                }
                Decl::StructDecl {
                    name,
                    is_class,
                    base_classes,
                    fields,
                    methods,
                    private_members,
                    line,
                    col,
                    ..
                } => {
                    self.compute_struct_layout(RecordLayout {
                        name: name.as_str(),
                        is_class: *is_class,
                        base_classes,
                        fields,
                        methods,
                        private_members,
                        line: *line,
                        col: *col,
                    })?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn synthesize_destructors(&mut self, program: &mut Program) {
        let mut structs_with_dtor = std::collections::HashSet::new();
        for decl in &program.declarations {
            if let Decl::StructDecl { name, methods, .. } = decl
                && methods.iter().any(|m| m.name.starts_with("~"))
            {
                structs_with_dtor.insert(name.clone());
            }
        }

        for decl in &mut program.declarations {
            if let Decl::StructDecl {
                name,
                base_classes,
                fields,
                methods,
                ..
            } = decl
            {
                let dtor_name = format!("~{}", name);
                if !methods.iter().any(|m| m.name == dtor_name) {
                    let has_dtor_members = fields
                        .iter()
                        .any(|(_, ty)| type_has_dtor(ty, &structs_with_dtor))
                        || base_classes.iter().any(|b| structs_with_dtor.contains(b));
                    if has_dtor_members {
                        methods.push(FunctionDecl {
                            name: dtor_name,
                            return_type: Type::Void,
                            params: vec![],
                            body: Some(Stmt::new(StmtKind::Compound(vec![]), 0, 0)),
                            is_virtual: false,
                            is_pure_virtual: false,
                            is_const: false,
                            is_static: false,
                            ctor_initializers: vec![],
                            line: 0,
                            col: 0,
                        });
                    }
                }
            }
        }
    }

    fn synthesize_constructors(&mut self, program: &mut Program) {
        // A default constructor is synthesized when the members require
        // construction, and for polymorphic classes that need their vtable
        // installed by a constructor. Iterate to a fixed point so a class
        // becomes constructible once its dependencies do.
        let polymorphic: std::collections::HashSet<String> = self
            .struct_layouts
            .iter()
            .filter(|(_, layout)| layout.has_vtable)
            .map(|(name, _)| name.clone())
            .collect();
        let (mut any_ctor, mut default_ctor) = self.collect_explicit_constructors(program);

        loop {
            let mut changed = false;
            for decl in &mut program.declarations {
                let Decl::StructDecl {
                    name,
                    base_classes,
                    fields,
                    methods,
                    ..
                } = decl
                else {
                    continue;
                };
                if any_ctor.contains(name) {
                    continue;
                }
                let short = name.rsplit("::").next().unwrap_or(name);
                let has_ctor = methods.iter().any(|m| m.name == *name || m.name == short);
                if has_ctor {
                    continue;
                }

                let base_needs = base_classes.iter().any(|base| default_ctor.contains(base));
                let base_blocks = base_classes
                    .iter()
                    .any(|base| any_ctor.contains(base) && !default_ctor.contains(base));
                let (field_needs, field_blocks) =
                    fields
                        .iter()
                        .fold((false, false), |(needs, blocks), (_, ty)| {
                            let (needs_status, blocks_status) =
                                type_ctor_status(ty, &any_ctor, &default_ctor);
                            (needs || needs_status, blocks || blocks_status)
                        });
                let needs_ctor = base_needs || field_needs || polymorphic.contains(name);
                if needs_ctor && !base_blocks && !field_blocks {
                    methods.push(FunctionDecl {
                        name: name.clone(),
                        return_type: Type::Void,
                        params: vec![],
                        body: Some(Stmt::new(StmtKind::Compound(vec![]), 0, 0)),
                        is_virtual: false,
                        is_pure_virtual: false,
                        is_const: false,
                        is_static: false,
                        ctor_initializers: vec![],
                        line: 0,
                        col: 0,
                    });
                    any_ctor.insert(name.clone());
                    default_ctor.insert(name.clone());
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    /// Collects the classes that declare a constructor, and those whose
    /// constructor takes no arguments.
    fn collect_explicit_constructors(
        &self,
        program: &Program,
    ) -> (
        std::collections::HashSet<String>,
        std::collections::HashSet<String>,
    ) {
        let mut any_ctor = std::collections::HashSet::new();
        let mut default_ctor = std::collections::HashSet::new();
        for decl in &program.declarations {
            if let Decl::StructDecl { name, methods, .. } = decl {
                let short = name.rsplit("::").next().unwrap_or(name);
                for m in methods {
                    if m.name == *name || m.name == short {
                        any_ctor.insert(name.clone());
                        if m.params.is_empty() {
                            default_ctor.insert(name.clone());
                        }
                    }
                }
            }
        }

        (any_ctor, default_ctor)
    }

    fn register_declarations(&mut self, program: &Program) -> Result<(), CompileError> {
        let mut defined_functions = std::collections::HashSet::new();
        for decl in &program.declarations {
            match decl {
                Decl::UsingNamespace(ns) => {
                    self.open_namespaces.push(ns.clone());
                }
                Decl::UsingDecl(name) => {
                    let short_name = name.split("::").last().unwrap_or(name).to_string();
                    self.using_decls.insert(short_name, name.clone());
                }
                Decl::Function(f) => {
                    let mangled = self.mangle_name(&f.name, &f.params);
                    if f.body.is_some() && !defined_functions.insert(mangled.clone()) {
                        return Err(CompileError::new(
                            format!("Redefinition of function '{}'", f.name),
                            f.line,
                            f.col,
                        ));
                    }
                    let sig = FunctionSig {
                        name: f.name.clone(),
                        mangled_name: mangled,
                        return_type: f.return_type.clone(),
                        param_types: f.params.iter().map(|p| p.ty.clone()).collect(),
                        default_args: f.params.iter().map(|p| p.default_val.clone()).collect(),
                        is_static: false,
                    };
                    self.register_function(sig);
                }
                Decl::StructDecl {
                    name,
                    methods,
                    static_fields,
                    ..
                } => {
                    self.register_struct_declarations(name.as_str(), methods, static_fields);
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Registers the static fields and methods of a class.
    fn register_struct_declarations(
        &mut self,
        name: &str,
        methods: &[FunctionDecl],
        static_fields: &[(String, Type, Option<Expr>)],
    ) {
        for sf in static_fields {
            let qual_name = format!("{}::{}", name, sf.0);
            self.global_vars.insert(
                qual_name.clone(),
                VarSymbol {
                    name: qual_name.clone(),
                    ty: sf.1.clone(),
                    is_global: true,
                    stack_offset: 0,
                },
            );
            if let Some(alias) = QualifiedName::new(&qual_name).implicit_alias() {
                self.global_vars.insert(
                    alias.to_string(),
                    VarSymbol {
                        name: alias.to_string(),
                        ty: sf.1.clone(),
                        is_global: true,
                        stack_offset: 0,
                    },
                );
            }
        }
        for m in methods {
            let mut params = m.params.clone();
            if !m.is_static {
                params.insert(0, this_param(name, m.is_const));
            }
            let qual_name = format!("{}::{}", name, m.name);
            let mangled = self.mangle_name(&qual_name, &params);
            let sig = FunctionSig {
                name: m.name.clone(),
                mangled_name: mangled,
                return_type: m.return_type.clone(),
                param_types: params.iter().map(|p| p.ty.clone()).collect(),
                default_args: params.iter().map(|p| p.default_val.clone()).collect(),
                is_static: m.is_static,
            };
            self.register_function(sig.clone());
            self.functions.entry(qual_name).or_default().push(sig);
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
            && let Some(Expr::InitializerList(list)) = init
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
        self.check_and_complete_var_type(ty, init, line, col)?;
        if let Some(init_expr) = init {
            let init_ty = self.analyze_expr(init_expr)?;
            if !self.can_assign(ty, &init_ty) {
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
                Decl::StructDecl { name, methods, .. } => {
                    self.current_class = Some(name.to_string());
                    for m in methods {
                        let mut func_decl = m.clone();
                        if !m.is_static {
                            func_decl.params.insert(0, this_param(name, m.is_const));
                        }
                        self.analyze_function(&mut func_decl)?;
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
fn this_param(class: &str, is_const: bool) -> Param {
    let inner = if is_const {
        Type::Const(Box::new(Type::Struct(class.to_string())))
    } else {
        Type::Struct(class.to_string())
    };
    Param {
        name: "this".to_string(),
        ty: Type::Pointer(Box::new(inner)),
        default_val: None,
    }
}

/// Reports whether a field/base type needs a synthesized default constructor
/// and whether an existing non-default constructor blocks synthesis.
fn type_ctor_status(
    ty: &Type,
    any_ctor: &std::collections::HashSet<String>,
    default_ctor: &std::collections::HashSet<String>,
) -> (bool, bool) {
    match ty.strip_const() {
        Type::Struct(name) => (
            default_ctor.contains(name),
            any_ctor.contains(name) && !default_ctor.contains(name),
        ),
        Type::Array(element, _) => type_ctor_status(element, any_ctor, default_ctor),
        _ => (false, false),
    }
}

/// Returns true when a value of `ty` owns a subobject with a destructor.
fn type_has_dtor(ty: &Type, structs_with_dtor: &std::collections::HashSet<String>) -> bool {
    match ty.strip_const() {
        Type::Struct(name) => structs_with_dtor.contains(name),
        Type::Array(element, _) => type_has_dtor(element, structs_with_dtor),
        _ => false,
    }
}
