use super::*;

impl Sema {
    pub fn analyze_program(&mut self, program: &mut Program) -> Result<(), CompileError> {
        // First pass: register structs and enums
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
                    ..
                } => {
                    self.compute_struct_layout(
                        name,
                        *is_class,
                        base_classes,
                        fields,
                        methods,
                        private_members,
                    )?;
                }
                _ => {}
            }
        }

        // Synthesize implicit destructors for structs that contain member objects or bases with destructors
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
                    let has_dtor_members =
                        fields.iter().any(|(_, ty)| {
                            if let Type::Struct(s) = ty.strip_const() {
                                structs_with_dtor.contains(s)
                            } else {
                                false
                            }
                        }) || base_classes.iter().any(|b| structs_with_dtor.contains(b));
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

        // Second pass: register functions, methods, static fields
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
                    };
                    self.register_function(sig);
                }
                Decl::StructDecl {
                    name,
                    methods,
                    static_fields,
                    ..
                } => {
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
                            let this_inner = if m.is_const {
                                Type::Const(Box::new(Type::Struct(name.clone())))
                            } else {
                                Type::Struct(name.clone())
                            };
                            params.insert(
                                0,
                                Param {
                                    name: "this".to_string(),
                                    ty: Type::Pointer(Box::new(this_inner)),
                                    default_val: None,
                                },
                            );
                        }
                        let qual_name = format!("{}::{}", name, m.name);
                        let mangled = self.mangle_name(&qual_name, &params);
                        let sig = FunctionSig {
                            name: m.name.clone(),
                            mangled_name: mangled,
                            return_type: m.return_type.clone(),
                            param_types: params.iter().map(|p| p.ty.clone()).collect(),
                            default_args: params.iter().map(|p| p.default_val.clone()).collect(),
                        };
                        self.register_function(sig.clone());
                        self.functions.entry(qual_name).or_default().push(sig);
                    }
                }
                _ => {}
            }
        }

        // Third pass: analyze declarations & bodies
        for decl in &mut program.declarations {
            match decl {
                Decl::GlobalVar {
                    name,
                    ty,
                    init,
                    line,
                    col,
                } => {
                    self.current_line = *line;
                    self.current_col = *col;
                    if let Type::Struct(sname) = ty.strip_const()
                        && let Some(layout) = self.struct_layouts.get(sname)
                        && layout.is_abstract
                    {
                        return Err(CompileError::new(
                            format!("Variable type '{}' is an abstract class", sname),
                            *line,
                            *col,
                        ));
                    }
                    if let Type::Array(elem, 0) = ty.clone()
                        && let Some(Expr::InitializerList(list)) = init
                    {
                        *ty = Type::Array(elem, list.len());
                    }
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
                                *line,
                                *col,
                            ));
                        }
                    }
                    self.global_vars.insert(
                        name.clone(),
                        VarSymbol {
                            name: name.clone(),
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
                            func_decl.params.insert(
                                0,
                                Param {
                                    name: "this".to_string(),
                                    ty: Type::Pointer(Box::new(Type::Struct(name.clone()))),
                                    default_val: None,
                                },
                            );
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
