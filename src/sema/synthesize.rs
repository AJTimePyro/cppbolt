use std::collections::HashSet;

use super::*;

fn insert_type_aliases(set: &mut HashSet<String>, name: &str) {
    set.insert(name.to_string());
    if let Some(short) = name.rsplit("::").next() {
        set.insert(short.to_string());
    }
    let qualified = QualifiedName::new(name);
    set.insert(qualified.std_name().into_owned());
    set.insert(qualified.plain().to_string());
}

fn set_contains_type(set: &HashSet<String>, name: &str) -> bool {
    if set.contains(name) {
        return true;
    }
    if let Some(short) = name.rsplit("::").next()
        && set.contains(short)
    {
        return true;
    }
    let qualified = QualifiedName::new(name);
    set.contains(qualified.std_name().as_ref()) || set.contains(qualified.plain())
}

impl Sema {
    pub(super) fn synthesize_destructors(&mut self, program: &mut Program) {
        let mut structs_with_dtor = HashSet::new();
        for decl in &program.declarations {
            if let Decl::StructDecl { name, methods, .. } = decl
                && methods.iter().any(|m| m.name.starts_with('~'))
            {
                insert_type_aliases(&mut structs_with_dtor, name);
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
                let has_dtor_members = fields
                    .iter()
                    .any(|(_, ty)| type_has_dtor(ty, &structs_with_dtor))
                    || base_classes
                        .iter()
                        .any(|b| set_contains_type(&structs_with_dtor, b));
                if has_dtor_members {
                    let dtor_name = format!("~{}", name);
                    if !methods.iter().any(|m| m.name == dtor_name) {
                        methods.push(FunctionDecl {
                            name: dtor_name,
                            return_type: Type::Void,
                            params: vec![],
                            body: Some(Stmt::new(StmtKind::Compound(vec![]), 0, 0)),
                            is_virtual: false,
                            is_pure_virtual: false,
                            is_const: false,
                            is_explicit: false,
                            ref_qualifier: RefQualifier::None,
                            is_static: false,
                            ctor_initializers: vec![],
                            template_params: vec![],
                            line: 0,
                            col: 0,
                        });
                    }
                    synthesize_copy_operations(name, fields, methods);
                }
            }
        }
    }

    pub(super) fn synthesize_constructors(&mut self, program: &mut Program) {
        // A default constructor is synthesized for classes without explicit
        // constructors unless blocked by subobject constructors. Iterate to a
        // fixed point so a class becomes constructible once its dependencies do.
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
                if set_contains_type(&any_ctor, name) {
                    continue;
                }
                let short = name.rsplit("::").next().unwrap_or(name.as_str());
                let has_ctor = methods.iter().any(|m| m.name == *name || m.name == short);
                if has_ctor {
                    continue;
                }

                let base_blocks = base_classes.iter().any(|base| {
                    set_contains_type(&any_ctor, base) && !set_contains_type(&default_ctor, base)
                });
                let field_blocks = fields
                    .iter()
                    .any(|(_, ty)| type_ctor_status(ty, &any_ctor, &default_ctor).1);
                if !base_blocks && !field_blocks {
                    methods.push(FunctionDecl {
                        name: name.clone(),
                        return_type: Type::Void,
                        params: vec![],
                        body: Some(Stmt::new(StmtKind::Compound(vec![]), 0, 0)),
                        is_virtual: false,
                        is_pure_virtual: false,
                        is_const: false,
                        is_explicit: false,
                        ref_qualifier: RefQualifier::None,
                        is_static: false,
                        ctor_initializers: vec![],
                        template_params: vec![],
                        line: 0,
                        col: 0,
                    });
                    insert_type_aliases(&mut any_ctor, name);
                    insert_type_aliases(&mut default_ctor, name);
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
    ) -> (HashSet<String>, HashSet<String>) {
        let mut any_ctor = HashSet::new();
        let mut default_ctor = HashSet::new();
        for decl in &program.declarations {
            if let Decl::StructDecl { name, methods, .. } = decl {
                let short = name.rsplit("::").next().unwrap_or(name.as_str());
                for m in methods {
                    if m.name == *name || m.name == short {
                        insert_type_aliases(&mut any_ctor, name);
                        if m.params.is_empty() {
                            insert_type_aliases(&mut default_ctor, name);
                        }
                    }
                }
            }
        }

        (any_ctor, default_ctor)
    }
}

/// Reports whether a field/base type needs a synthesized default constructor
/// and whether an existing non-default constructor blocks synthesis.
fn type_ctor_status(
    ty: &Type,
    any_ctor: &HashSet<String>,
    default_ctor: &HashSet<String>,
) -> (bool, bool) {
    match ty.strip_const() {
        Type::Struct(name) => (
            set_contains_type(default_ctor, name),
            set_contains_type(any_ctor, name) && !set_contains_type(default_ctor, name),
        ),
        Type::Array(element, _) => type_ctor_status(element, any_ctor, default_ctor),
        _ => (false, false),
    }
}

/// Returns true when a value of `ty` owns a subobject with a destructor.
fn type_has_dtor(ty: &Type, structs_with_dtor: &HashSet<String>) -> bool {
    match ty.strip_const() {
        Type::Struct(name) => set_contains_type(structs_with_dtor, name),
        Type::Array(element, _) => type_has_dtor(element, structs_with_dtor),
        _ => false,
    }
}

fn synthesize_copy_operations(
    name: &str,
    fields: &[(String, Type)],
    methods: &mut Vec<FunctionDecl>,
) {
    synthesize_copy_constructor(name, fields, methods);
    synthesize_copy_assignment(name, fields, methods);
}

fn synthesize_copy_constructor(
    name: &str,
    fields: &[(String, Type)],
    methods: &mut Vec<FunctionDecl>,
) {
    let short = name.rsplit("::").next().unwrap_or(name);
    let has_copy_ctor = methods.iter().any(|m| {
        (m.name == name || m.name == short)
            && m.params.len() == 1
            && matches!(m.params[0].ty.strip_const(), Type::Reference(inner) if matches!(inner.strip_const(), Type::Struct(s) if s == name || s == short))
    });
    if !has_copy_ctor {
        let param_ty = Type::Reference(Box::new(Type::Const(Box::new(Type::Struct(name.to_string())))));
        let initializers = fields
            .iter()
            .map(|(fname, _)| {
                (
                    fname.clone(),
                    vec![Expr::MemberAccess {
                        object: Box::new(Expr::Identifier("o".to_string())),
                        member: fname.clone(),
                        is_arrow: false,
                        pos: ExprPos::new(0, 0),
                    }],
                )
            })
            .collect();
        methods.push(FunctionDecl {
            name: name.to_string(),
            return_type: Type::Void,
            params: vec![Param {
                name: "o".to_string(),
                ty: param_ty,
                default_val: None,
            }],
            body: Some(Stmt::new(StmtKind::Compound(vec![]), 0, 0)),
            is_virtual: false,
            is_pure_virtual: false,
            is_const: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
            is_static: false,
            ctor_initializers: initializers,
            template_params: vec![],
            line: 0,
            col: 0,
        });
    }
}

fn synthesize_copy_assignment(
    name: &str,
    fields: &[(String, Type)],
    methods: &mut Vec<FunctionDecl>,
) {
    let short = name.rsplit("::").next().unwrap_or(name);
    let has_copy_assign = methods.iter().any(|m| {
        m.name == "operator="
            && m.params.len() == 1
            && matches!(m.params[0].ty.strip_const(), Type::Reference(inner) if matches!(inner.strip_const(), Type::Struct(s) if s == name || s == short))
    });
    if !has_copy_assign {
        let param_ty = Type::Reference(Box::new(Type::Const(Box::new(Type::Struct(name.to_string())))));
        let mut body_stmts = Vec::new();
        for (fname, _) in fields {
            body_stmts.push(Stmt::new(
                StmtKind::Expr(Expr::Assign {
                    target: Box::new(Expr::MemberAccess {
                        object: Box::new(Expr::This),
                        member: fname.clone(),
                        is_arrow: true,
                        pos: ExprPos::new(0, 0),
                    }),
                    value: Box::new(Expr::MemberAccess {
                        object: Box::new(Expr::Identifier("o".to_string())),
                        member: fname.clone(),
                        is_arrow: false,
                        pos: ExprPos::new(0, 0),
                    }),
                    pos: ExprPos::new(0, 0),
                }),
                0,
                0,
            ));
        }
        body_stmts.push(Stmt::new(
            StmtKind::Return(Some(Expr::Unary {
                op: UnaryOp::Deref,
                expr: Box::new(Expr::This),
                pos: ExprPos::new(0, 0),
            })),
            0,
            0,
        ));
        methods.push(FunctionDecl {
            name: "operator=".to_string(),
            return_type: Type::Reference(Box::new(Type::Struct(name.to_string()))),
            params: vec![Param {
                name: "o".to_string(),
                ty: param_ty,
                default_val: None,
            }],
            body: Some(Stmt::new(StmtKind::Compound(body_stmts), 0, 0)),
            is_virtual: false,
            is_pure_virtual: false,
            is_const: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
            is_static: false,
            ctor_initializers: vec![],
            template_params: vec![],
            line: 0,
            col: 0,
        });
    }
}
