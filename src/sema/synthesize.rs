use std::collections::HashSet;

use super::*;

pub(crate) fn insert_type_aliases(set: &mut HashSet<String>, name: &str) {
    set.insert(name.to_string());
    if let Some(short) = name.rsplit("::").next() {
        set.insert(short.to_string());
    }
    let qualified = QualifiedName::new(name);
    set.insert(qualified.std_name().into_owned());
    set.insert(qualified.plain().to_string());
}

pub(crate) fn set_contains_type(set: &HashSet<String>, name: &str) -> bool {
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
        for decl in &program.declarations {
            if let Decl::StructDecl { name, methods, .. } = decl {
                if methods.iter().any(|m| m.name.starts_with('~')) {
                    insert_type_aliases(&mut self.structs_with_dtor, name);
                }
                let short = name.rsplit("::").next().unwrap_or(name.as_str());
                if methods.iter().any(|m| {
                    (m.name == *name || m.name == short)
                        && m.params.len() == 1
                        && m.params[0].ty.is_rvalue_reference()
                }) {
                    insert_type_aliases(&mut self.structs_with_move, name);
                }
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
                self.synthesize_instantiation_lifecycle(name, base_classes, fields, methods);
            }
        }
    }

    pub(crate) fn synthesize_instantiation_lifecycle(
        &mut self,
        inst_name: &str,
        base_classes: &[String],
        fields: &[StructFieldDecl],
        methods: &mut Vec<FunctionDecl>,
    ) {
        let has_custom_dtor = methods.iter().any(|m| m.name.starts_with('~'));
        if has_custom_dtor {
            insert_type_aliases(&mut self.structs_with_dtor, inst_name);
        }
        let has_dtor_members = fields
            .iter()
            .any(|(_, ty, _)| type_has_dtor(ty, &self.structs_with_dtor))
            || base_classes
                .iter()
                .any(|b| set_contains_type(&self.structs_with_dtor, b));
        if has_dtor_members {
            let dtor_name = format!("~{}", inst_name);
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
            insert_type_aliases(&mut self.structs_with_dtor, inst_name);
        }
        let short = inst_name.rsplit("::").next().unwrap_or(inst_name);
        let has_user_ctor = methods
            .iter()
            .any(|m| m.name == inst_name || m.name == short);
        if !has_user_ctor {
            methods.push(FunctionDecl {
                name: inst_name.to_string(),
                return_type: Type::Void,
                params: Vec::new(),
                body: Some(Stmt::new(StmtKind::Compound(Vec::new()), 0, 0)),
                is_virtual: false,
                is_pure_virtual: false,
                is_const: false,
                is_explicit: false,
                ref_qualifier: RefQualifier::None,
                is_static: false,
                ctor_initializers: Vec::new(),
                template_params: Vec::new(),
                line: 0,
                col: 0,
            });
        }
        let has_move_members = fields
            .iter()
            .any(|(_, ty, _)| type_has_move(ty, &self.structs_with_move))
            || base_classes
                .iter()
                .any(|b| set_contains_type(&self.structs_with_move, b));
        if has_dtor_members || has_custom_dtor || has_move_members {
            synthesize_move_operations(inst_name, base_classes, fields, methods);
            synthesize_copy_operations(inst_name, base_classes, fields, methods);
            insert_type_aliases(&mut self.structs_with_move, inst_name);
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
                    .any(|(_, ty, _)| type_ctor_status(ty, &any_ctor, &default_ctor).1);
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
pub(crate) fn type_has_dtor(ty: &Type, structs_with_dtor: &HashSet<String>) -> bool {
    match ty.strip_const() {
        Type::Struct(name) => set_contains_type(structs_with_dtor, name),
        Type::Array(element, _) => type_has_dtor(element, structs_with_dtor),
        _ => false,
    }
}

/// Returns true when a value of `ty` owns a subobject with a move constructor.
pub(crate) fn type_has_move(ty: &Type, structs_with_move: &HashSet<String>) -> bool {
    match ty.strip_const() {
        Type::Struct(name) => set_contains_type(structs_with_move, name),
        Type::Array(element, _) => type_has_move(element, structs_with_move),
        _ => false,
    }
}

pub(crate) use super::synthesize_ops::{synthesize_copy_operations, synthesize_move_operations};
