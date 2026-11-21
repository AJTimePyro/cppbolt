//! Nested-type lookup helpers for class-template specializations.

use super::template_deduce::split_template_call_name;
use super::*;
use crate::ast::template::TemplateClass;
use crate::ast::types::Type;
use crate::name::QualifiedName;
use std::collections::HashSet;

/// The definition selected when this specialization was instantiated. Reading
/// the cache keeps nested-member validation on exactly the same selection path
/// as normal instantiation, including confirmation and ambiguity handling.
pub(super) fn selected_class_definition(sema: &Sema, base: &str) -> Option<TemplateClass> {
    sema.instantiated_class_definitions.get(base).cloned()
}

pub(super) fn declares_member_alias(class: &TemplateClass, member: &str) -> bool {
    class.member_aliases.iter().any(|name| name == member)
}

fn class_bases(sema: &Sema, class_name: &str) -> Vec<String> {
    let mut bases = Vec::new();
    if let Some(layout) = sema.struct_layouts.get(class_name) {
        bases.extend(layout.base_classes.iter().cloned());
    }
    if let Some(class) = sema.instantiated_class_definitions.get(class_name) {
        bases.extend(class.base_classes.iter().cloned());
    } else if let Some((class, _)) = sema
        .class_templates
        .get(QualifiedName::new(class_name).plain())
    {
        bases.extend(class.base_classes.iter().cloned());
    }
    bases
        .into_iter()
        .map(|base| sema.canonical_class_name(&base, Some(class_name)))
        .collect()
}

pub(super) fn nested_type_declarers(sema: &Sema, owner: &str, member: &str) -> Vec<String> {
    let mut visited = HashSet::new();
    let mut queue = vec![owner.to_string()];
    let mut found = Vec::new();
    while let Some(class_name) = queue.pop() {
        if !visited.insert(class_name.clone()) {
            continue;
        }
        let qualified = format!("{}::{}", class_name, member);
        if sema.lookup_type_alias(&qualified).is_some()
            || sema.struct_layouts.contains_key(&qualified)
        {
            found.push(class_name);
            continue;
        }
        // A non-type member declared directly in this class hides any nested
        // type of the same name in a base, so stop the search here.
        if class_declares_nontype_member(sema, &class_name, member) {
            continue;
        }
        queue.extend(class_bases(sema, &class_name));
    }
    found
}

/// True when the class itself declares a non-type member (data field, static
/// member, enumerator or method) named `member`. The layout's `fields` include
/// inherited entries laid out before the class's own, so compare offsets with
/// the end of the direct base subobjects.
fn class_declares_nontype_member(sema: &Sema, class_name: &str, member: &str) -> bool {
    let Some(layout) = sema.struct_layouts.get(class_name) else {
        return false;
    };
    let own_start = layout
        .direct_base_offsets
        .iter()
        .filter_map(|(name, offset)| {
            let base = sema.struct_layouts.get(name)?;
            // An empty base at offset zero does not advance the cursor, so its
            // size must not be counted as space before the derived fields.
            if *offset == 0 && base.fields.is_empty() && !base.has_vtable {
                Some(0)
            } else {
                Some(offset.saturating_add(base.size))
            }
        })
        .max()
        .unwrap_or(0);
    layout
        .fields
        .iter()
        .any(|field| field.name == member && field.offset >= own_start)
        || sema
            .global_vars
            .contains_key(&format!("{}::{}", class_name, member))
        || sema
            .enum_values
            .contains_key(&format!("{}::{}", class_name, member))
        || sema
            .functions
            .contains_key(&format!("{}::{}", class_name, member))
}

pub(super) fn count_base_paths(
    sema: &Sema,
    from: &str,
    target: &str,
    visiting: &mut HashSet<String>,
) -> usize {
    if from == target {
        return 1;
    }
    if !visiting.insert(from.to_string()) {
        return 0;
    }
    let total: usize = class_bases(sema, from)
        .iter()
        .map(|base| count_base_paths(sema, base, target, visiting))
        .sum();
    visiting.remove(from);
    total
}

fn class_declares_nested_struct(class: &TemplateClass, member: &str) -> bool {
    use crate::ast::Decl;
    class.nested_structs.iter().any(|nested| {
        matches!(nested, Decl::StructDecl { name, .. }
            if name.rsplit("::").next().is_some_and(|short| short == member))
    })
}

/// Checks the plain (non-template) base classes for a nested type, either
/// under the qualified spelling or the uninstantiated spelling used by member
/// alias templates.
fn plain_base_declares_nested_member(
    sema: &Sema,
    name: &str,
    member: &str,
    selected: bool,
) -> bool {
    let qualified = format!("{}::{}", name, member);
    if sema.struct_layouts.contains_key(&qualified) || sema.lookup_type_alias(&qualified).is_some()
    {
        return true;
    }
    if selected {
        // Only the selected definition counts. Falling back to the plain
        // template spelling would accept an alias declared by the primary
        // even though this specialization does not have that member.
        return false;
    }
    // Member aliases of nested member templates are registered under the
    // uninstantiated spelling, without template argument lists.
    let plain: String = name
        .split("::")
        .map(|part| part.split('<').next().unwrap_or(part))
        .collect::<Vec<_>>()
        .join("::");
    let plain_qualified = format!("{}::{}", plain, member);
    sema.struct_layouts.contains_key(&plain_qualified)
        || sema.lookup_type_alias(&plain_qualified).is_some()
}

/// True when a class template or one of its partial specializations declares
/// a nested member with the given name. Such members are valid types even
/// without a registered type alias.
pub(super) fn template_declares_nested_member(sema: &Sema, base: &str, member: &str) -> bool {
    let (tmpl_name, _) = split_template_call_name(base).unwrap_or((base, ""));
    let selected = selected_class_definition(sema, base);
    if selected
        .as_ref()
        .is_some_and(|class| declares_member_alias(class, member))
    {
        return true;
    }
    let primary = sema.class_templates.values().map(|(class, _)| class);
    let partials = sema
        .partial_specializations
        .values()
        .flatten()
        .map(|spec| &spec.template_class);
    let classes: Vec<_> = if let Some(class) = &selected {
        vec![class]
    } else {
        primary.chain(partials).collect()
    };

    // A member may come from the class itself or from one of its bases.
    let mut visited: Vec<String> = Vec::new();
    let mut queue = vec![selected.as_ref().map_or_else(
        || QualifiedName::new(tmpl_name).plain().to_string(),
        |c| c.name.clone(),
    )];
    while let Some(name) = queue.pop() {
        if visited.contains(&name) {
            continue;
        }
        visited.push(name.clone());
        for class in classes
            .iter()
            .filter(|c| QualifiedName::new(&c.name).plain() == name)
        {
            if class_declares_nested_struct(class, member) {
                return true;
            }
            // A base written without qualification lives in the template's
            // namespace, so search both its raw and namespace-qualified names.
            let namespace = class.name.rsplit_once("::").map(|(n, _)| n).unwrap_or("");
            for base in &class.base_classes {
                queue.push(base.clone());
                queue.push(QualifiedName::new(base).plain().to_string());
                if !namespace.is_empty() && !base.contains("::") {
                    queue.push(format!("{}::{}", namespace, base));
                }
            }
        }
    }
    // Members of plain (non-template) bases live in the layout and alias maps.
    let selected = selected.is_some();
    visited
        .iter()
        .any(|name| plain_base_declares_nested_member(sema, name, member, selected))
}

/// True when a type still mentions template parameters or other dependent
/// nodes. Used to keep dependent member accesses out of the hard-error path
/// for concrete specializations.
pub(super) fn type_is_dependent(ty: &Type) -> bool {
    match ty {
        Type::TypeParam(_)
        | Type::DependentNested(_)
        | Type::DependentNonType(_)
        | Type::DependentArray(..)
        | Type::DependentAlignas(..)
        | Type::Decltype { .. } => true,
        Type::Const(inner)
        | Type::Pointer(inner)
        | Type::Reference(inner)
        | Type::RvalueReference(inner)
        | Type::Array(inner, _)
        | Type::PackExpansion(inner)
        | Type::Aligned(inner, _) => type_is_dependent(inner),
        Type::TemplateInstance { args, .. } => args.iter().any(type_is_dependent),
        Type::Function {
            return_type,
            param_types,
            ..
        } => type_is_dependent(return_type) || param_types.iter().any(type_is_dependent),
        Type::MemberPointer { member_type, .. } | Type::BoundMemberFunction { member_type, .. } => {
            type_is_dependent(member_type)
        }
        _ => false,
    }
}
