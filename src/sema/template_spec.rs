//! Template specialization storage, lookup, and registration.

use super::*;
use crate::ast::template::TemplateClass;
use crate::diagnostics::CompileError;
use crate::name::QualifiedName;
use crate::sema::pack_expansion::TemplateArg;
use std::collections::HashSet;

pub struct InstantiationTarget<'a> {
    pub name: &'a str,
    pub inst_name: &'a str,
    pub template_params: &'a [TemplateParam],
    pub effective_args: &'a [Type],
    pub pos: (usize, usize),
}

impl Sema {
    pub fn register_class_template(
        &mut self,
        decl: TemplateClass,
        params: Vec<TemplateParam>,
        pos: (usize, usize),
    ) -> Result<(), CompileError> {
        let known_packs: HashSet<String> = params
            .iter()
            .filter(|p| p.is_pack)
            .map(|p| p.name.clone())
            .collect();
        if !known_packs.is_empty() {
            Self::validate_template_member_packs(&decl, &known_packs, pos)?;
        }
        super::template_registry::insert_template_aliases(
            &mut self.class_templates,
            decl.name.clone(),
            (decl, params),
        );
        Ok(())
    }

    fn validate_template_member_packs(
        decl: &TemplateClass,
        known_packs: &HashSet<String>,
        pos: (usize, usize),
    ) -> Result<(), CompileError> {
        for (_, f_ty, _) in &decl.fields {
            if let Some(pack) =
                super::pack_expansion::find_unexpanded_packs_in_type(f_ty, known_packs).first()
            {
                return Err(CompileError::new(
                    format!("data member type contains unexpanded parameter pack '{pack}'"),
                    pos.0,
                    pos.1,
                ));
            }
        }
        for m in &decl.methods {
            for p in &m.params {
                if !p.is_pack
                    && let Some(pack) =
                        super::pack_expansion::find_unexpanded_packs_in_type(&p.ty, known_packs)
                            .first()
                {
                    return Err(CompileError::new(
                        format!("declaration type contains unexpanded parameter pack '{pack}'"),
                        pos.0,
                        pos.1,
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn register_class_specialization(
        &mut self,
        decl: TemplateClass,
        pos: (usize, usize),
    ) -> Result<(), CompileError> {
        let plain = QualifiedName::new(&decl.name).plain().to_string();
        if self.class_specializations.contains_key(&plain)
            || self.class_specializations.contains_key(&decl.name)
        {
            return Err(CompileError::new(
                format!(
                    "redefinition of explicit template specialization '{}'",
                    plain
                ),
                pos.0,
                pos.1,
            ));
        }
        self.class_specializations
            .insert(decl.name.clone(), decl.clone());
        self.class_specializations.insert(plain, decl);
        Ok(())
    }

    pub fn lookup_class_specialization(&self, name: &str) -> Option<TemplateClass> {
        self.class_specializations.get(name).cloned().or_else(|| {
            let plain = QualifiedName::new(name).plain();
            self.class_specializations.get(plain).cloned()
        })
    }

    pub fn resolve_instantiation_target(
        &mut self,
        target: InstantiationTarget<'_>,
        template_class: TemplateClass,
    ) -> Result<(TemplateClass, HashMap<String, TemplateArg>), CompileError> {
        if let Some(spec) = self.lookup_class_specialization(target.inst_name) {
            let m = self.prepare_instantiation_mapping(
                &spec,
                target.template_params,
                target.effective_args,
                target.inst_name,
                target.pos,
            );
            Ok((spec, m))
        } else if let Some((spec, mut m)) =
            self.select_partial_specialization(target.name, target.effective_args, target.pos)?
        {
            self.register_instantiation_nested_and_aliases(
                &spec,
                target.inst_name,
                &mut m,
                target.pos,
            );
            Ok((spec, m))
        } else {
            let m = self.prepare_instantiation_mapping(
                &template_class,
                target.template_params,
                target.effective_args,
                target.inst_name,
                target.pos,
            );
            Ok((template_class, m))
        }
    }

    pub fn ensure_class_instantiated(&mut self, sname: &str, line: usize, col: usize) {
        if self.struct_layouts.contains_key(sname) {
            return;
        }
        let plain = QualifiedName::new(sname).plain();
        if let Some(Type::TemplateInstance { name, args }) = self
            .type_aliases
            .get(sname)
            .or_else(|| self.type_aliases.get(plain))
            .cloned()
        {
            let mut args = args;
            for arg in &mut args {
                let _ = self.resolve_and_instantiate_type(arg, line, col);
            }
            if let Ok(inst) = self.instantiate_class_template(&name, &args, line, col) {
                self.alias_static_fields(&inst, sname);
                self.alias_static_fields(&inst, plain);
            }
            return;
        }
        if let Some((base, args_str)) = super::template_deduce::split_template_call_name(sname) {
            let known: Vec<String> = self
                .class_templates
                .keys()
                .chain(self.alias_templates.keys())
                .cloned()
                .collect();
            if let Ok(mut args) = super::template_deduce::parse_types_list(args_str, &known) {
                for arg in &mut args {
                    let _ = self.resolve_and_instantiate_type(arg, line, col);
                }
                let _ = self.instantiate_class_template(base, &args, line, col);
            }
        }
    }

    pub fn alias_static_fields(&mut self, source: &str, target: &str) {
        let prefix = format!("{source}::");
        let matching: Vec<(String, VarSymbol)> = self
            .global_vars
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(k, v)| (k[prefix.len()..].to_string(), v.clone()))
            .collect();
        for (fld, var) in matching {
            let qual = format!("{target}::{fld}");
            self.global_vars.insert(
                qual.clone(),
                VarSymbol {
                    name: var.name.clone(),
                    ty: var.ty.clone(),
                    is_global: true,
                    stack_offset: 0,
                },
            );
        }
    }
}
