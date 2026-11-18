//! Instantiated member method setup.

use crate::diagnostics::ErrorKind;
use super::analyze::this_param;
use super::*;
use crate::ast::decl::FunctionDecl;
use crate::name::QualifiedName;

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
        for mut m in all_methods {
            if m.template_params.is_empty() && (m.is_virtual || m.name.starts_with('~')) {
                let prev_class = self.current_class.take();
                self.current_class = Some(inst_name.to_string());
                let mut vmethod = m.clone();
                if !vmethod.is_static {
                    vmethod
                        .params
                        .insert(0, this_param(inst_name, vmethod.is_const));
                }
                let res = self.analyze_function(&mut vmethod);
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
                        if e.kind == ErrorKind::Internal
                            || self.object_use_classes.contains(inst_name) =>
                    {
                        return Err(e);
                    }
                    Err(_) => {
                        // Only a pointer reaches this specialization: Clang
                        // never instantiates its body, so neither do we.
                    }
                }
            }
        }
        Ok(())
    }
}
