use crate::ast::types::Type;
use crate::diagnostics::CompileError;
use crate::sema::pack_expansion::TemplateArg;
use std::collections::HashMap;

pub fn evaluate_dependent_alignas(
    pack_name: &str,
    mapping: &HashMap<String, TemplateArg>,
    struct_aligns: &HashMap<String, usize>,
) -> usize {
    if let Some(arg) = mapping.get(pack_name) {
        match arg {
            TemplateArg::TypePack(types) => types
                .iter()
                .map(|t| t.align_of(struct_aligns))
                .max()
                .unwrap_or(1),
            TemplateArg::Type(ty) => ty.align_of(struct_aligns),
            _ => 1,
        }
    } else {
        1
    }
}

pub fn substitute_dependent_alignas(
    inner: &Type,
    pack_name: &str,
    mapping: &HashMap<String, TemplateArg>,
    injected_class: Option<(&str, &Type)>,
) -> Result<Type, CompileError> {
    let subst_inner = super::template_subst::substitute_type(inner, mapping, injected_class)?;
    let empty_aligns = HashMap::new();
    let align = evaluate_dependent_alignas(pack_name, mapping, &empty_aligns);
    Ok(Type::Aligned(Box::new(subst_inner), align))
}

pub fn substitute_type_param(
    name: &str,
    ty: &Type,
    mapping: &HashMap<String, TemplateArg>,
) -> Result<Type, CompileError> {
    match mapping.get(name) {
        Some(TemplateArg::Type(t)) => Ok(t.clone()),
        Some(TemplateArg::NonType(n)) => Ok(Type::NonType(*n)),
        Some(TemplateArg::TypePack(_)) | Some(TemplateArg::NonTypePack(_)) => Err(CompileError::new(
            format!("parameter pack '{}' used unexpanded", name),
            0,
            0,
        )),
        _ => Ok(ty.clone()),
    }
}

pub fn collapse_reference(outer_is_rvalue: bool, inner: Type) -> Type {
    if inner.is_reference() {
        let is_lvalue = inner.is_lvalue_reference();
        let target = inner.deref_type().cloned().unwrap_or(inner);
        let ctor = if !outer_is_rvalue || is_lvalue {
            Type::Reference
        } else {
            Type::RvalueReference
        };
        ctor(Box::new(target))
    } else if outer_is_rvalue {
        Type::RvalueReference(Box::new(inner))
    } else {
        Type::Reference(Box::new(inner))
    }
}

pub fn substitute_base_classes(
    bases: &[String],
    mapping: &HashMap<String, TemplateArg>,
    line: usize,
    col: usize,
) -> Result<Vec<String>, CompileError> {
    let mut base_classes = Vec::new();
    for base in bases {
        if let Some(stripped) = base.strip_suffix("...") {
            if let Some(TemplateArg::TypePack(types)) = mapping.get(stripped) {
                for ty in types {
                    base_classes.push(ty.to_c_type());
                }
            } else {
                base_classes.push(super::template_name::substitute_template_name(base, mapping));
            }
        } else if let Some(TemplateArg::Type(target)) = mapping.get(base) {
            base_classes.push(target.to_c_type());
        } else if base.contains('<') {
            base_classes.push(super::template_name::substitute_template_name(base, mapping));
        } else {
            base_classes.push(base.clone());
        }
    }
    let mut seen = std::collections::HashSet::new();
    for b in &base_classes {
        if !seen.insert(b.clone()) {
            return Err(CompileError::new(
                format!("base class '{b}' specified more than once as a direct base class"),
                line,
                col,
            ));
        }
    }
    Ok(base_classes)
}
