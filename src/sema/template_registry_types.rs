//! Type resolution and instantiation for the template registry.

use super::template_nested_lookup::{template_declares_nested_member, type_is_dependent};
use super::*;
use crate::diagnostics::{ErrorKind, SubstitutionFailure, TemplateFailure};
use crate::name::QualifiedName;

impl Sema {
    /// Resolves a member alias target through any chain of dependent nested
    /// types, so a registered alias holds the final type instead of an opaque
    /// `Template<...>::type` spelling.
    pub(super) fn resolve_member_alias_chain(
        &mut self,
        ty: &mut Type,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        // Cycles are detected by revisiting a name; the separate limit only
        // guards aliases that generate ever-growing nested type names.
        const MAX_ALIAS_CHAIN: usize = 500;
        let mut visited = HashSet::new();
        self.resolve_and_instantiate_type(ty, line, col)?;
        while let Type::Struct(name) = &*ty {
            let name = name.clone();
            if !name.contains(">::") || !visited.insert(name) {
                break;
            }
            if visited.len() > MAX_ALIAS_CHAIN {
                return Err(CompileError::internal(
                    "dependent alias chain exceeded the resolution limit",
                    line,
                    col,
                ));
            }
            self.resolve_and_instantiate_type(ty, line, col)?;
        }
        Ok(())
    }

    pub fn resolve_and_instantiate_type(
        &mut self,
        ty: &mut Type,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        match ty {
            Type::TemplateInstance { name, args } => {
                let resolved = self.resolve_template_instance(name, args, line, col)?;
                *ty = resolved;
            }
            Type::Decltype {
                expr,
                parenthesized,
            } => {
                let resolved = self.resolve_decltype_type(expr, *parenthesized, line, col)?;
                *ty = resolved;
                self.resolve_and_instantiate_type(ty, line, col)?;
            }
            Type::DependentNested(name) => {
                if let Some(resolved) = self.resolve_dependent_nested_type(name, line, col)? {
                    *ty = resolved;
                    self.resolve_and_instantiate_type(ty, line, col)?;
                }
            }
            Type::Struct(sname) => {
                let sname_clone = sname.clone();
                self.resolve_struct_type_alias(ty, &sname_clone, line, col)?;
                if let Type::Struct(resolved_name) = ty {
                    self.check_nested_type_access(&sname_clone, line, col)?;
                    if resolved_name != &sname_clone {
                        self.check_nested_type_access(resolved_name, line, col)?;
                    }
                } else {
                    self.resolve_and_instantiate_type(ty, line, col)?;
                }
            }
            Type::Pointer(inner)
            | Type::Reference(inner)
            | Type::RvalueReference(inner)
            | Type::Const(inner) => {
                self.resolve_and_instantiate_type(inner, line, col)?;
            }
            Type::Array(inner, size) => {
                if *size == usize::MAX {
                    return Err(CompileError::new(
                        "Array dimension must be non-negative integer constant",
                        line,
                        col,
                    ));
                }
                self.resolve_and_instantiate_type(inner, line, col)?;
            }
            Type::MemberPointer {
                class_name,
                member_type,
            } => {
                *class_name = self.resolve_alias_in_qualified_name(class_name);
                self.resolve_and_instantiate_type(member_type, line, col)?;
            }
            Type::Function {
                return_type,
                param_types,
                ..
            } => {
                self.resolve_and_instantiate_type(return_type, line, col)?;
                for p in param_types.iter_mut() {
                    self.resolve_and_instantiate_type(p, line, col)?;
                }
            }
            Type::BoundMemberFunction { member_type, .. } => {
                self.resolve_and_instantiate_type(member_type, line, col)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Resolves a `Template<...>` use: folds variable-template constants,
    /// expands alias templates, and instantiates class templates.
    fn resolve_template_instance(
        &mut self,
        name: &str,
        args: &mut [Type],
        line: usize,
        col: usize,
    ) -> Result<Type, CompileError> {
        let plain = QualifiedName::new(name).plain();
        if self.variable_templates.contains_key(name) || self.variable_templates.contains_key(plain)
        {
            let rendered = format!(
                "{}<{}>",
                name,
                args.iter()
                    .map(|a| a.to_c_type())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if self
                .instantiate_variable_template(&rendered, line, col)
                .ok()
                .flatten()
                .is_some()
                && let Some(value) = self.static_const_values.get(&rendered).copied()
            {
                return Ok(Type::NonType(value));
            }
        }
        for arg in args.iter_mut() {
            self.resolve_and_instantiate_type(arg, line, col)?;
        }
        if let Some(expanded) = self.expand_alias_template(name, args, line, col)? {
            return Ok(expanded);
        }
        let inst_name = self.instantiate_class_template(name, args, line, col)?;
        Ok(Type::Struct(inst_name))
    }

    /// Resolves a dependent qualified name used as a type. A static const
    /// member is a constant rather than a nested type.
    fn resolve_dependent_nested_type(
        &mut self,
        name: &str,
        line: usize,
        col: usize,
    ) -> Result<Option<Type>, CompileError> {
        if let Some(value) = self.resolve_static_const_nontype(name, line, col)? {
            return Ok(Some(Type::NonType(value)));
        }
        let Some((_, Some(resolved))) = self.resolve_nested_type_name(name, line, col)? else {
            let (base, member) = name.rsplit_once("::").unwrap_or(("", name));
            return Err(CompileError::new(
                format!("no type named '{}' in '{}'", member, base),
                line,
                col,
            )
            .with_kind(ErrorKind::Substitution));
        };
        Ok(Some(resolved))
    }

    /// Resolves a signature type: a missing nested type discards the template
    /// candidate, while an error from instantiating another template stays hard.
    pub(super) fn resolve_signature_type(
        &mut self,
        ty: &mut Type,
        line: usize,
        col: usize,
    ) -> Result<(), TemplateFailure> {
        self.resolve_and_instantiate_type(ty, line, col)
            .map_err(|error| {
                if error.kind == ErrorKind::Substitution {
                    TemplateFailure::Substitution(SubstitutionFailure::new(
                        error.message,
                        error.line,
                        error.col,
                    ))
                } else {
                    TemplateFailure::Hard(error)
                }
            })
    }

    fn resolve_struct_type_alias(
        &mut self,
        ty: &mut Type,
        sname: &str,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let owner = match self.resolve_nested_type_name(sname, line, col)? {
            Some((_, Some(alias))) => {
                *ty = alias;
                return Ok(());
            }
            Some((owner, None)) => owner,
            None => String::new(),
        };
        // A member of a concrete class-template specialization must exist:
        // `enable_if<false, T>::type` has no `type` member and is a hard error
        // outside substitution. Dependent arguments are left for the existing
        // substitution machinery to resolve or reject at instantiation.
        let Some(gt_pos) = sname.rfind('>') else {
            let Some((class_part, member)) = sname.rsplit_once("::") else {
                return Ok(());
            };
            let plain = QualifiedName::new(class_part).plain();
            if self.struct_layouts.contains_key(class_part)
                || self.class_templates.contains_key(plain)
                || self.class_specializations.contains_key(plain)
            {
                return Err(CompileError::new(
                    format!("no type named '{}' in '{}'", member, class_part),
                    line,
                    col,
                )
                .with_kind(ErrorKind::Substitution));
            }
            return Ok(());
        };
        if !sname[gt_pos..].starts_with(">::") {
            return Ok(());
        }
        let inst_part = &sname[..gt_pos + 1];
        let Some((_base, args_str)) = super::template_deduce::split_template_call_name(inst_part)
        else {
            return Ok(());
        };
        let known: Vec<String> = self
            .class_templates
            .keys()
            .chain(self.alias_templates.keys())
            .cloned()
            .collect();
        let Ok(args) = super::template_deduce::parse_types_list(args_str, &known) else {
            return Ok(());
        };
        if args.iter().any(type_is_dependent) {
            return Ok(());
        }
        // A nested struct declared by the template is a valid type even when
        // no member alias was registered for it. Only report a hard error when
        // no template definition declares a nested member of that name.
        let check_name = if owner.is_empty() { inst_part } else { &owner };
        if template_declares_nested_member(self, check_name, &sname[gt_pos + 3..]) {
            return Ok(());
        }
        Err(CompileError::new(
            format!(
                "no type named '{}' in '{}'",
                &sname[gt_pos + 3..],
                inst_part
            ),
            line,
            col,
        )
        // This is an invalid type in the immediate context. Outside
        // substitution it remains a normal compile error; inside candidate
        // formation the caller discards the candidate instead.
        .with_kind(ErrorKind::Substitution))
    }
}
