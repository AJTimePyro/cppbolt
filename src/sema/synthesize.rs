use std::collections::HashSet;

use super::*;

impl Sema {
    pub(super) fn synthesize_destructors(&mut self, program: &mut Program) {
        let mut structs_with_dtor = HashSet::new();
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
                if any_ctor.contains(name.as_str()) {
                    continue;
                }
                let short = name.rsplit("::").next().unwrap_or(name.as_str());
                let has_ctor = methods.iter().any(|m| m.name == *name || m.name == short);
                if has_ctor {
                    continue;
                }

                let base_blocks = base_classes
                    .iter()
                    .any(|base| any_ctor.contains(base.as_str()) && !default_ctor.contains(base.as_str()));
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
    ) -> (HashSet<String>, HashSet<String>) {
        let mut any_ctor = HashSet::new();
        let mut default_ctor = HashSet::new();
        for decl in &program.declarations {
            if let Decl::StructDecl { name, methods, .. } = decl {
                let short = name.rsplit("::").next().unwrap_or(name.as_str());
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
            default_ctor.contains(name.as_str()),
            any_ctor.contains(name.as_str()) && !default_ctor.contains(name.as_str()),
        ),
        Type::Array(element, _) => type_ctor_status(element, any_ctor, default_ctor),
        _ => (false, false),
    }
}

/// Returns true when a value of `ty` owns a subobject with a destructor.
fn type_has_dtor(ty: &Type, structs_with_dtor: &HashSet<String>) -> bool {
    match ty.strip_const() {
        Type::Struct(name) => structs_with_dtor.contains(name.as_str()),
        Type::Array(element, _) => type_has_dtor(element, structs_with_dtor),
        _ => false,
    }
}
