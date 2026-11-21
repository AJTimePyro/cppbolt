//! Argument-dependent lookup.

use super::*;

#[derive(Default)]
struct AssociatedEntities {
    namespaces: Vec<String>,
    classes: Vec<String>,
    seen: HashSet<String>,
}

/// Innermost enclosing namespace of a qualified name. `::` inside template
/// arguments does not separate namespace components.
fn namespace_of(name: &str) -> Option<&str> {
    let bytes = name.as_bytes();
    let mut depth = 0i32;
    let mut found = None;
    for i in 0..bytes.len().saturating_sub(1) {
        match bytes[i] {
            b'<' => depth += 1,
            b'>' => depth -= 1,
            b':' if bytes[i + 1] == b':' && depth == 0 => found = Some(i),
            _ => {}
        }
    }
    found.map(|index| &name[..index])
}

impl Sema {
    /// Adds a class or enum name and everything associated with it: its
    /// innermost enclosing namespace, its base classes, and (for a
    /// specialization) the entities of its template type arguments.
    fn associate_name(&mut self, name: &str, out: &mut AssociatedEntities) {
        let plain = QualifiedName::new(name).plain().to_string();
        let key = self.canonical_class_name(&plain, None);
        if !out.seen.insert(key.clone()) {
            return;
        }
        if let Some(namespace) = namespace_of(&key)
            && !out.namespaces.iter().any(|ns| ns == namespace)
        {
            out.namespaces.push(namespace.to_string());
        }
        out.classes.push(key.clone());
        let plain = QualifiedName::new(&key).plain().to_string();
        if plain != key && !out.classes.contains(&plain) {
            out.classes.push(plain.clone());
        }
        if let Some(layout) = self.struct_layouts.get(&key).cloned() {
            for base in layout.base_classes {
                self.associate_name(&base, out);
            }
        }
        if let Some((template_class, _)) = self.class_templates.get(&plain).cloned() {
            for base in template_class.base_classes {
                self.associate_name(&base, out);
            }
        }
    }

    fn associate_type(&mut self, ty: &Type, out: &mut AssociatedEntities) {
        match ty {
            Type::Const(inner)
            | Type::Pointer(inner)
            | Type::Reference(inner)
            | Type::RvalueReference(inner) => self.associate_type(inner, out),
            Type::Array(inner, _) | Type::DependentArray(inner, _) => {
                self.associate_type(inner, out)
            }
            Type::Struct(name) | Type::Enum(name) => self.associate_name(name, out),
            Type::TemplateInstance { name, args } => {
                self.associate_name(name, out);
                for arg in args {
                    self.associate_type(arg, out);
                }
            }
            Type::Function {
                return_type,
                param_types,
                ..
            } => {
                self.associate_type(return_type, out);
                for param in param_types {
                    self.associate_type(param, out);
                }
            }
            Type::MemberPointer { member_type, .. }
            | Type::BoundMemberFunction { member_type, .. } => {
                self.associate_type(member_type, out)
            }
            _ => {}
        }
    }

    fn associated_entities(&mut self, arg_types: &[Type]) -> AssociatedEntities {
        let mut out = AssociatedEntities::default();
        for arg in arg_types {
            self.associate_type(arg, &mut out);
        }
        out
    }

    /// Namespace-scope functions found by ADL for an unqualified call.
    pub(super) fn adl_candidates(&mut self, name: &str, arg_types: &[Type]) -> Vec<FunctionSig> {
        let out = self.associated_entities(arg_types);
        let mut found = Vec::new();
        for namespace in out.namespaces {
            let qualified = format!("{}::{}", namespace, name);
            if let Some(sigs) = self.functions.get(&qualified).cloned() {
                found.extend(sigs);
            }
        }
        for class_name in out.classes {
            if let Some(friends) = self.hidden_friends.get(&class_name).cloned() {
                found.extend(friends.into_iter().filter(|sig| sig.name == name));
            }
        }
        found
    }

    /// Friend function templates associated with the argument types, found
    /// only through ADL. Each specialisation registers its own signature.
    pub(super) fn adl_friend_templates(
        &mut self,
        name: &str,
        arg_types: &[Type],
    ) -> Vec<(FunctionDecl, Vec<TemplateParam>)> {
        let entities = self.associated_entities(arg_types);
        let mut found: Vec<(FunctionDecl, Vec<TemplateParam>)> = Vec::new();
        for class in entities.classes {
            let Some(list) = self.hidden_friend_templates.get(&class).cloned() else {
                continue;
            };
            for item in list {
                let plain = QualifiedName::new(&item.0.name).plain();
                if (item.0.name == name || plain == name)
                    && !found.iter().any(|(decl, _)| {
                        decl.params == item.0.params && decl.return_type == item.0.return_type
                    })
                {
                    found.push(item);
                }
            }
        }
        found
    }
}
