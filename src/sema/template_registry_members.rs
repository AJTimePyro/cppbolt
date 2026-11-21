//! Instantiated member method setup.

use super::analyze::this_param;
use super::template_registry::function_template_identity;
use super::template_subst::substitute_type;
use super::*;
use crate::ast::decl::FunctionDecl;
use crate::ast::expr::Expr;
use crate::diagnostics::ErrorKind;
use crate::name::QualifiedName;
use std::collections::HashMap;

impl Sema {
    pub(super) fn add_synthesized_method(&mut self, class_name: &str, method: FunctionDecl) {
        for decl in &mut self.synthesized_decls {
            if let Decl::StructDecl { name, methods, .. } = decl
                && name == class_name
            {
                methods.push(method);
                return;
            }
        }
    }

    pub(super) fn setup_instantiated_methods(
        &mut self,
        inst_name: &str,
        all_methods: Vec<FunctionDecl>,
    ) -> Result<(), CompileError> {
        all_methods
            .iter()
            .filter(|m| m.template_params.is_empty())
            .for_each(|m| self.register_struct_method(inst_name, m));
        for m in &all_methods {
            if !m.template_params.is_empty() {
                self.member_templates
                    .entry(inst_name.to_string())
                    .or_default()
                    .push(m.clone());
                let plain = QualifiedName::new(inst_name).plain();
                if plain != inst_name {
                    self.member_templates
                        .entry(plain.to_string())
                        .or_default()
                        .push(m.clone());
                }
            } else if !m.is_virtual && !m.name.starts_with('~') {
                let qual_name = format!("{}::{}", inst_name, m.name);
                let mut params = m.params.clone();
                if !m.is_static {
                    params.insert(0, this_param(inst_name, m.is_const));
                }
                let mut mangled = self.mangle_name(&qual_name, &params);
                match m.ref_qualifier {
                    RefQualifier::LValue => mangled.push_str("_R"),
                    RefQualifier::RValue => mangled.push_str("_O"),
                    RefQualifier::None => {}
                }
                self.pending_template_methods
                    .insert(mangled, (inst_name.to_string(), m.clone()));
            }
        }
        for m in all_methods {
            if m.template_params.is_empty() && (m.is_virtual || m.name.starts_with('~')) {
                self.analyze_instantiated_virtual_method(inst_name, m)?;
            }
        }
        Ok(())
    }

    fn analyze_instantiated_virtual_method(
        &mut self,
        inst_name: &str,
        mut m: FunctionDecl,
    ) -> Result<(), CompileError> {
        let prev_class = self.current_class.take();
        self.current_class = Some(inst_name.to_string());
        let mut vmethod = m.clone();
        if !vmethod.is_static {
            vmethod
                .params
                .insert(0, this_param(inst_name, vmethod.is_const));
        }
        let saved_identity = self.current_template_identity.take();
        self.current_template_identity = Some(
            super::template_registry::function_template_identity(&m.name, m.line, m.col),
        );
        let res = self.analyze_function(&mut vmethod);
        self.current_template_identity = saved_identity;
        self.current_class = prev_class;
        match res {
            Ok(()) => {
                m.body = vmethod.body;
                m.ctor_initializers = vmethod.ctor_initializers;
                self.add_synthesized_method(inst_name, m);
            }
            Err(e) if e.kind == ErrorKind::UnresolvedName => {
                // Globals and functions are registered in a later pass.
                // Hold the substituted body for a bounded retry; never
                // emit a body that failed analysis.
                self.deferred_methods.push(DeferredMethod {
                    class_name: inst_name.to_string(),
                    method: m,
                });
            }
            Err(e)
                if e.kind == ErrorKind::Internal || self.object_use_classes.contains(inst_name) =>
            {
                return Err(e);
            }
            Err(_) => {
                // Only a pointer reaches this specialization: Clang
                // never instantiates its body, so neither do we.
            }
        }
        Ok(())
    }

    pub(super) fn register_instantiated_static_fields(
        &mut self,
        inst_name: &str,
        static_fields: &[(String, Type, Option<Expr>)],
        base_classes: &[String],
        line: usize,
        col: usize,
    ) {
        for sf in static_fields.iter() {
            let qual_name = format!("{}::{}", inst_name, sf.0);
            self.global_vars.insert(
                qual_name.clone(),
                VarSymbol {
                    name: qual_name.clone(),
                    ty: sf.1.clone(),
                    is_global: true,
                    stack_offset: 0,
                },
            );
            if sf.2.is_some() {
                self.fold_static_const_member(inst_name, sf);
            }
        }
        for base in base_classes {
            let _ = self.ensure_class_instantiated(base, line, col);
            self.alias_static_fields(base, inst_name);
        }
    }

    pub(super) fn register_instantiated_member_aliases(
        &mut self,
        tmpl_name: &str,
        inst_name: &str,
        mapping: &HashMap<String, crate::sema::pack_expansion::TemplateArg>,
        line: usize,
        col: usize,
    ) {
        let plain_tmpl = QualifiedName::new(tmpl_name).plain();
        let plain_inst = QualifiedName::new(inst_name).plain();
        let is_std =
            QualifiedName::new(inst_name).in_std() || QualifiedName::new(tmpl_name).in_std();
        let mut new_aliases = Vec::new();
        let existing: Vec<(String, Type)> = self
            .type_aliases
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (alias_name, ty) in &existing {
            let Some((owner, member)) = alias_name.rsplit_once("::") else {
                continue;
            };
            if QualifiedName::new(owner).plain() == plain_tmpl
                && let Ok(mut subst_ty) = substitute_type(ty, mapping, None)
            {
                // Path 2 is a best-effort re-registration; a resolution error
                // leaves the partially resolved type as it did before.
                let _ = self.resolve_member_alias_chain(&mut subst_ty, line, col);
                new_aliases.push((format!("{inst_name}::{member}"), subst_ty.clone()));
                if is_std {
                    new_aliases.push((format!("std::{plain_inst}::{member}"), subst_ty.clone()));
                }
                if plain_inst != inst_name {
                    new_aliases.push((format!("{plain_inst}::{member}"), subst_ty));
                }
            }
        }
        for (name, ty) in new_aliases {
            self.type_aliases.insert(name, ty);
        }
    }

    pub(super) fn retry_deferred_methods(&mut self) -> Result<(), CompileError> {
        const MAX_PASSES: usize = 8;
        for _ in 0..MAX_PASSES {
            if self.deferred_methods.is_empty() {
                return Ok(());
            }
            // Take the batch so retry-triggered instantiations can queue more.
            let batch = std::mem::take(&mut self.deferred_methods);
            for deferred in batch {
                let DeferredMethod { class_name, method } = deferred;
                let method_identity =
                    function_template_identity(&method.name, method.line, method.col);
                let mut analyzed = method;
                if !analyzed.is_static {
                    analyzed
                        .params
                        .insert(0, this_param(&class_name, analyzed.is_const));
                }
                let prev_class = self.current_class.take();
                self.current_class = Some(class_name.clone());
                let saved_identity = self.current_template_identity.take();
                self.current_template_identity = Some(method_identity);
                let res = self.analyze_function(&mut analyzed);
                self.current_template_identity = saved_identity;
                self.current_class = prev_class;
                if let Err(e) = res {
                    if e.kind == ErrorKind::Internal
                        || self.object_use_classes.contains(&class_name)
                    {
                        return Err(e);
                    }
                    continue;
                }
                if !analyzed.is_static && !analyzed.params.is_empty() {
                    analyzed.params.remove(0);
                }
                self.update_registered_method_return_type(&class_name, &analyzed);
                self.add_synthesized_method(&class_name, analyzed);
            }
        }
        if self.deferred_methods.is_empty() {
            Ok(())
        } else {
            Err(crate::internal_error!(
                1,
                1,
                "deferred method analysis did not converge"
            ))
        }
    }

    pub fn instantiate_pending_method(&mut self, mangled_name: &str) -> Result<(), CompileError> {
        if let Some((class_name, mut method_decl)) =
            self.pending_template_methods.remove(mangled_name)
        {
            let prev_class = self.current_class.take();
            self.current_class = Some(class_name.clone());
            let mut func_decl = method_decl.clone();
            if !method_decl.is_static {
                func_decl
                    .params
                    .insert(0, this_param(&class_name, method_decl.is_const));
            }
            let saved_identity = self.current_template_identity.take();
            self.current_template_identity = Some(function_template_identity(
                &method_decl.name,
                method_decl.line,
                method_decl.col,
            ));
            let res = self.analyze_function(&mut func_decl);
            self.current_template_identity = saved_identity;
            self.current_class = prev_class;
            res?;
            method_decl.body = func_decl.body;
            method_decl.ctor_initializers = func_decl.ctor_initializers;
            let mut found = false;
            for decl in &mut self.synthesized_decls {
                if let Decl::StructDecl { name, methods, .. } = decl
                    && name == &class_name
                {
                    if let Some(existing) = methods
                        .iter_mut()
                        .find(|m| m.name == method_decl.name && m.params == method_decl.params)
                    {
                        *existing = method_decl.clone();
                    } else {
                        methods.push(method_decl.clone());
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                self.synthesized_decls.push(Decl::StructDecl {
                    name: class_name,
                    is_class: false,
                    is_final: false,
                    base_classes: vec![],
                    base_access: vec![],
                    fields: vec![],
                    methods: vec![method_decl],
                    static_fields: vec![],
                    private_members: vec![],
                    nested_structs: vec![],
                    alignment: None,
                    line: 0,
                    col: 0,
                });
            }
        }
        Ok(())
    }
}
