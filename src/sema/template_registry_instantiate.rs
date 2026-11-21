//! Class template instantiation.

use super::template_registry::function_template_identity;
use super::template_subst::{SubstContext, substitute_struct};
use super::*;
use crate::ast::template::{TemplateClass, TemplateParam};
use std::collections::HashMap;

impl Sema {
    pub fn instantiate_class_template(
        &mut self,
        name: &str,
        args: &[Type],
        line: usize,
        col: usize,
    ) -> Result<String, CompileError> {
        if let Some(expanded) = self.expand_alias_template(name, args, line, col)? {
            if let Type::Struct(sname) = expanded {
                return Ok(sname);
            }
            return Ok(expanded.to_c_type());
        }

        let (template_class, template_params) = self.resolve_class_template(name, line, col)?;

        // A non-type argument spelled as a static const member of another
        // class template (`Trait<T>::value`) must be folded to its constant
        // value before argument-kind validation.
        let mut resolved_args = args.to_vec();
        self.resolve_nontype_template_args(&template_params, &mut resolved_args, (line, col))?;

        let effective_args = super::template_deduce::fill_default_template_args(
            name,
            &template_params,
            &resolved_args,
            (line, col),
        )?;

        if self.instantiation_depth >= 500 {
            return Err(CompileError::new(
                "template instantiation depth exceeded (limit: 500)",
                line,
                col,
            ));
        }

        let canonical_args = effective_args
            .iter()
            .map(|a| a.to_c_type())
            .collect::<Vec<_>>()
            .join(", ");
        let plain_name = QualifiedName::new(name).plain();
        let inst_name = format!("{}<{}>", plain_name, canonical_args);

        // A completed specialization is cached. An in-progress one is still
        // on the stack and must satisfy recursive references without being
        // treated as finished.
        if self.instantiated_classes.contains(&inst_name)
            || self.instantiation_stack.iter().any(|s| s == &inst_name)
        {
            return Ok(inst_name);
        }

        self.instantiation_stack.push(inst_name.clone());
        self.instantiation_depth += 1;
        let snapshot = super::template_state::InstantiationSnapshot::capture(self);

        let result = self
            .resolve_instantiation_target(
                super::template_spec::InstantiationTarget {
                    name,
                    inst_name: &inst_name,
                    template_params: &template_params,
                    effective_args: &effective_args,
                    pos: (line, col),
                },
                template_class,
            )
            .and_then(|(target_class, mapping)| {
                self.execute_class_instantiation(&target_class, mapping, &inst_name, (line, col))
            });

        self.finish_class_instantiation(result, &inst_name, snapshot)
    }

    fn resolve_nontype_template_args(
        &mut self,
        params: &[TemplateParam],
        args: &mut [Type],
        pos: (usize, usize),
    ) -> Result<(), CompileError> {
        let (line, col) = pos;
        for (param, arg) in params.iter().zip(args.iter_mut()) {
            if param.param_type.is_some() {
                self.try_resolve_nontype_arg(arg, line, col)?;
            }
        }
        Ok(())
    }

    fn finish_class_instantiation(
        &mut self,
        result: Result<(), CompileError>,
        inst_name: &str,
        snapshot: super::template_state::InstantiationSnapshot,
    ) -> Result<String, CompileError> {
        self.instantiation_stack.pop();
        self.instantiation_depth -= 1;
        match result {
            Ok(()) => {
                self.instantiated_classes.insert(inst_name.to_string());
                Ok(inst_name.to_string())
            }
            Err(error) => {
                if self.speculating {
                    self.speculative_hard = true;
                }
                snapshot.restore(self);
                Err(error)
            }
        }
    }

    fn execute_class_instantiation(
        &mut self,
        template_class: &TemplateClass,
        mapping: HashMap<String, crate::sema::pack_expansion::TemplateArg>,
        inst_name: &str,
        pos: (usize, usize),
    ) -> Result<(), CompileError> {
        let (line, col) = pos;
        self.instantiate_nested_structs(template_class, inst_name, &mapping, pos)?;

        let struct_aligns = self.struct_aligns.clone();
        let mut struct_decl = {
            let mut ctx = SubstContext {
                mapping: &mapping,
                injected_class: None,
                pos,
                struct_aligns: Some(&struct_aligns),
                sema: Some(self),
            };
            substitute_struct(template_class, inst_name, &mut ctx)?
        };
        let Decl::StructDecl {
            fields,
            methods,
            static_fields,
            base_classes,
            base_access,
            is_class,
            private_members,
            alignment,
            ..
        } = &mut struct_decl
        else {
            return Ok(());
        };

        for (_, fty, _) in fields.iter_mut() {
            self.resolve_and_instantiate_type(fty, line, col)?;
        }

        for base in base_classes.iter() {
            let _ = self.ensure_class_instantiated(base, line, col);
        }

        self.resolve_instantiated_method_signatures(methods, line, col)?;

        self.synthesize_instantiation_lifecycle(inst_name, base_classes, fields, methods);

        self.compute_struct_layout(RecordLayout {
            name: inst_name,
            is_class: *is_class,
            is_final: false,
            base_classes,
            base_access,
            fields,
            methods,
            private_members,
            line,
            col,
            alignment: alignment
                .as_ref()
                .map(|ty| ty.align_of(&self.struct_aligns)),
        })?;

        self.register_instantiated_static_fields(inst_name, static_fields, base_classes, line, col);
        self.register_instantiated_member_aliases(
            &template_class.name,
            inst_name,
            &mapping,
            line,
            col,
        );
        let inst_ty = Type::Struct(inst_name.to_string());
        self.instantiate_friend_functions(template_class, &mapping, inst_name, &inst_ty)?;
        let all_methods = std::mem::take(methods);
        self.synthesized_decls.push(struct_decl);
        self.setup_instantiated_methods(inst_name, all_methods)?;
        Ok(())
    }

    fn resolve_instantiated_method_signatures(
        &mut self,
        methods: &mut [FunctionDecl],
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        for m in methods.iter_mut() {
            if m.template_params.is_empty() {
                let saved_identity = self.current_template_identity.take();
                self.current_template_identity =
                    Some(function_template_identity(&m.name, m.line, m.col));
                let result = (|| {
                    self.resolve_and_instantiate_type(&mut m.return_type, line, col)?;
                    for p in &mut m.params {
                        self.resolve_and_instantiate_type(&mut p.ty, line, col)?;
                    }
                    Ok(())
                })();
                self.current_template_identity = saved_identity;
                result?;
            }
        }
        Ok(())
    }

    fn instantiate_friend_functions(
        &mut self,
        template_class: &TemplateClass,
        mapping: &HashMap<String, crate::sema::pack_expansion::TemplateArg>,
        inst_name: &str,
        inst_ty: &Type,
    ) -> Result<(), CompileError> {
        for friend in &template_class.friends {
            let mut instantiated = super::template_subst_func::substitute_function(
                friend,
                mapping,
                Some((template_class.name.as_str(), inst_ty)),
                None,
            )?;
            if !friend.template_params.is_empty() {
                // The enclosing class parameters are already substituted; the
                // friend keeps its own parameters until a call deduces them.
                self.register_hidden_friend_template(
                    inst_name,
                    instantiated.clone(),
                    friend.template_params.clone(),
                );
                continue;
            }
            self.register_friend_function(inst_name, instantiated.clone())?;
            let prev_class = self.current_class.take();
            self.current_class = Some(inst_name.to_string());
            let analyzed = self.analyze_function(&mut instantiated);
            self.current_class = prev_class;
            analyzed?;
            self.synthesized_decls
                .push(crate::ast::Decl::Function(instantiated));
        }
        Ok(())
    }
}
