use super::template_subst::{substitute_function, substitute_type};
use super::*;
use crate::ast::decl::{Decl, FunctionDecl};
use crate::ast::template::{TemplateClass, TemplateParam};
use crate::ast::types::Type;
use crate::name::QualifiedName;
use std::collections::HashMap;

struct NestedStructParams<'a> {
    name: &'a str,
    nested_inst_name: &'a str,
    is_class: bool,
    base_classes: &'a [String],
    fields: &'a [StructFieldDecl],
    methods: &'a [FunctionDecl],
    private_members: &'a [String],
    mapping: &'a HashMap<String, Type>,
    pos: (usize, usize),
}

impl Sema {
    pub(super) fn register_instantiation_nested_and_aliases(
        &mut self,
        template_class: &TemplateClass,
        inst_name: &str,
        mapping: &mut HashMap<String, Type>,
        pos: (usize, usize),
    ) {
        let (line, col) = pos;
        for nested in &template_class.nested_structs {
            if let Decl::StructDecl { name, .. } = nested {
                let short = name.rsplit_once("::").map(|(_, s)| s).unwrap_or(name);
                let inst_nested = format!("{}::{}", inst_name, short);
                let inst_ty = Type::Struct(inst_nested.clone());
                mapping.insert(short.to_string(), inst_ty.clone());
                mapping.insert(name.clone(), inst_ty.clone());
                let plain_tmpl = QualifiedName::new(&template_class.name).plain();
                mapping.insert(format!("{}::{}", plain_tmpl, short), inst_ty);
            }
        }

        self.populate_instantiation_member_aliases(
            template_class,
            inst_name,
            mapping,
            line,
            col,
        );
    }

    pub(super) fn prepare_instantiation_mapping(
        &mut self,
        template_class: &TemplateClass,
        template_params: &[TemplateParam],
        args: &[Type],
        inst_name: &str,
        pos: (usize, usize),
    ) -> HashMap<String, Type> {
        let mut mapping = HashMap::new();
        for (param, arg) in template_params.iter().zip(args.iter()) {
            mapping.insert(param.name.clone(), arg.clone());
        }
        self.register_instantiation_nested_and_aliases(
            template_class,
            inst_name,
            &mut mapping,
            pos,
        );
        mapping
    }

    fn populate_instantiation_member_aliases(
        &mut self,
        template_class: &TemplateClass,
        inst_name: &str,
        mapping: &mut HashMap<String, Type>,
        line: usize,
        col: usize,
    ) {
        let plain_tmpl = QualifiedName::new(&template_class.name).plain();
        let plain_inst = QualifiedName::new(inst_name).plain();
        let is_std = QualifiedName::new(inst_name).in_std()
            || QualifiedName::new(&template_class.name).in_std();

        let mut changed = true;
        let mut iterations = 0;
        while changed && iterations < 16 {
            changed = false;
            iterations += 1;
            let existing: Vec<(String, Type)> = self
                .type_aliases
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            for (alias_name, ty) in existing {
                let Some((owner, member)) = alias_name.rsplit_once("::") else {
                    continue;
                };
                if QualifiedName::new(owner).plain() == plain_tmpl && !mapping.contains_key(member)
                {
                    let mut subst_ty = substitute_type(&ty, mapping, None);
                    let _ = self.resolve_and_instantiate_type(&mut subst_ty, line, col);
                    mapping.insert(member.to_string(), subst_ty.clone());
                    self.type_aliases
                        .insert(format!("{inst_name}::{member}"), subst_ty.clone());
                    if is_std {
                        self.type_aliases
                            .insert(format!("std::{plain_inst}::{member}"), subst_ty.clone());
                    }
                    if plain_inst != inst_name {
                        self.type_aliases
                            .insert(format!("{plain_inst}::{member}"), subst_ty);
                    }
                    changed = true;
                }
            }
        }
    }

    pub(super) fn instantiate_nested_structs(
        &mut self,
        template_class: &TemplateClass,
        inst_name: &str,
        mapping: &HashMap<String, Type>,
        pos: (usize, usize),
    ) -> Result<(), CompileError> {
        let (line, col) = pos;
        for nested in &template_class.nested_structs {
            let Decl::StructDecl {
                name,
                is_class,
                base_classes,
                fields,
                methods,
                private_members,
                ..
            } = nested
            else {
                continue;
            };

            let short_name = name.rsplit_once("::").map(|(_, s)| s).unwrap_or(name);
            let nested_inst_name = format!("{}::{}", inst_name, short_name);
            if self.struct_layouts.contains_key(&nested_inst_name) {
                continue;
            }

            self.instantiate_single_nested_struct(NestedStructParams {
                name,
                nested_inst_name: &nested_inst_name,
                is_class: *is_class,
                base_classes,
                fields,
                methods,
                private_members,
                mapping,
                pos: (line, col),
            })?;
        }
        Ok(())
    }

    fn instantiate_single_nested_struct(
        &mut self,
        p: NestedStructParams<'_>,
    ) -> Result<(), CompileError> {
        let (line, col) = p.pos;
        let target_ty = Type::Struct(p.nested_inst_name.to_string());
        let injected = Some((p.name, &target_ty));

        let mut subst_fields = Vec::new();
        for (fname, fty, finit) in p.fields {
            let mut ty = substitute_type(fty, p.mapping, injected);
            self.resolve_and_instantiate_type(&mut ty, line, col)?;
            let mut init = finit.clone();
            if let Some((init_expr, _, _)) = &mut init {
                template_subst::substitute_expr(init_expr, p.mapping, injected);
            }
            subst_fields.push((fname.clone(), ty, init));
        }

        let mut subst_methods = Vec::new();
        for m in p.methods {
            let mut sm = substitute_function(m, p.mapping, injected, None);
            if sm.template_params.is_empty() {
                self.resolve_and_instantiate_type(&mut sm.return_type, line, col)?;
                for param in &mut sm.params {
                    self.resolve_and_instantiate_type(&mut param.ty, line, col)?;
                }
            }
            subst_methods.push(sm);
        }

        let fields_for_layout = subst_fields.clone();
        let mut methods_for_layout = subst_methods.clone();
        self.synthesize_instantiation_lifecycle(
            p.nested_inst_name,
            p.base_classes,
            &fields_for_layout,
            &mut methods_for_layout,
        );

        self.compute_struct_layout(RecordLayout {
            name: p.nested_inst_name,
            is_class: p.is_class,
            base_classes: p.base_classes,
            fields: &fields_for_layout,
            methods: &methods_for_layout,
            private_members: p.private_members,
            line,
            col,
        })?;

        let all_methods = std::mem::take(&mut methods_for_layout);
        let subst_decl = Decl::StructDecl {
            name: p.nested_inst_name.to_string(),
            is_class: p.is_class,
            base_classes: p.base_classes.to_vec(),
            fields: subst_fields,
            methods: subst_methods,
            static_fields: Vec::new(),
            private_members: p.private_members.to_vec(),
            nested_structs: Vec::new(),
            line,
            col,
        };

        self.synthesized_decls.push(subst_decl);
        self.setup_instantiated_methods(p.nested_inst_name, all_methods)?;
        Ok(())
    }
}
