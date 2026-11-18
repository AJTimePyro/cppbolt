use super::pack_expansion::TemplateArg;
use super::template_subst::{substitute_function, substitute_type};
use super::*;
use crate::ast::decl::{Decl, FunctionDecl, StructFieldDecl};
use crate::ast::template::{TemplateClass, TemplateDecl, TemplateKind, TemplateParam};
use crate::ast::types::Type;
use crate::name::QualifiedName;
use std::collections::{HashMap, HashSet};

struct NestedStructParams<'a> {
    name: &'a str,
    nested_inst_name: &'a str,
    is_class: bool,
    base_classes: &'a [String],
    fields: &'a [StructFieldDecl],
    methods: &'a [FunctionDecl],
    private_members: &'a [String],
    mapping: &'a HashMap<String, TemplateArg>,
    pos: (usize, usize),
}

struct NestedTemplateParams<'a> {
    inner_tmpl: &'a TemplateClass,
    inner_params: &'a [TemplateParam],
    inst_name: &'a str,
    mapping: &'a HashMap<String, TemplateArg>,
}

impl Sema {
    pub(super) fn register_instantiation_nested_and_aliases(
        &mut self,
        template_class: &TemplateClass,
        inst_name: &str,
        mapping: &mut HashMap<String, TemplateArg>,
        pos: (usize, usize),
    ) -> Result<(), CompileError> {
        let (line, col) = pos;
        for nested in &template_class.nested_structs {
            let nested_name = match nested {
                Decl::StructDecl { name, .. } => Some(name.as_str()),
                Decl::Template(TemplateDecl {
                    kind: TemplateKind::Class(inner_tmpl),
                    ..
                }) => Some(inner_tmpl.name.as_str()),
                _ => None,
            };
            if let Some(name) = nested_name {
                let short = name.rsplit_once("::").map(|(_, s)| s).unwrap_or(name);
                let inst_nested = format!("{}::{}", inst_name, short);
                let inst_ty = Type::Struct(inst_nested.clone());
                mapping.insert(short.to_string(), TemplateArg::Type(inst_ty.clone()));
                mapping.insert(name.to_string(), TemplateArg::Type(inst_ty.clone()));
                let plain_tmpl = QualifiedName::new(&template_class.name).plain();
                mapping.insert(
                    format!("{}::{}", plain_tmpl, short),
                    TemplateArg::Type(inst_ty),
                );
            }
        }

        self.populate_instantiation_member_aliases(template_class, inst_name, mapping, line, col)
    }

    pub(super) fn prepare_instantiation_mapping(
        &mut self,
        template_class: &TemplateClass,
        template_params: &[TemplateParam],
        args: &[Type],
        inst_name: &str,
        pos: (usize, usize),
    ) -> Result<HashMap<String, TemplateArg>, CompileError> {
        let mut mapping = super::template_deduce::build_template_arg_map(template_params, args);
        self.register_instantiation_nested_and_aliases(
            template_class,
            inst_name,
            &mut mapping,
            pos,
        )?;
        Ok(mapping)
    }

    /// Substitutes a member alias target and resolves its dependent nested
    /// types. A bad nested type here is an error in the instantiated class,
    /// not in the caller's signature.
    fn substitute_member_alias_type(
        &mut self,
        ty: &Type,
        mapping: &HashMap<String, TemplateArg>,
        nested: &HashSet<String>,
        line: usize,
        col: usize,
    ) -> Result<Type, CompileError> {
        use super::template_subst_func::mark_dependent_nested;
        let mut subst_ty = substitute_type(ty, mapping, None)?;
        mark_dependent_nested(ty, &mut subst_ty, mapping);
        if let Type::DependentNested(name) = &subst_ty {
            let member = name.rsplit_once("::").map_or(name.as_str(), |(_, m)| m);
            // Nested struct layouts are registered later in this same
            // instantiation, so leave their names for that pass to resolve.
            if nested.contains(member) {
                return Ok(Type::Struct(name.clone()));
            }
        }
        self.resolve_and_instantiate_type(&mut subst_ty, line, col)?;
        Ok(subst_ty)
    }

    fn populate_instantiation_member_aliases(
        &mut self,
        template_class: &TemplateClass,
        inst_name: &str,
        mapping: &mut HashMap<String, TemplateArg>,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let plain_tmpl = QualifiedName::new(&template_class.name).plain();
        let plain_inst = QualifiedName::new(inst_name).plain();
        let is_std = QualifiedName::new(inst_name).in_std()
            || QualifiedName::new(&template_class.name).in_std();
        let nested: HashSet<String> = template_class
            .nested_structs
            .iter()
            .filter_map(|decl| match decl {
                Decl::StructDecl { name, .. } => Some(
                    name.rsplit_once("::")
                        .map_or(name.as_str(), |(_, short)| short)
                        .to_string(),
                ),
                _ => None,
            })
            .collect();

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
                    let subst_ty =
                        self.substitute_member_alias_type(&ty, mapping, &nested, line, col)?;
                    mapping.insert(member.to_string(), TemplateArg::Type(subst_ty.clone()));
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
        Ok(())
    }

    pub(super) fn instantiate_nested_structs(
        &mut self,
        template_class: &TemplateClass,
        inst_name: &str,
        mapping: &HashMap<String, TemplateArg>,
        pos: (usize, usize),
    ) -> Result<(), CompileError> {
        let (line, col) = pos;
        for nested in &template_class.nested_structs {
            match nested {
                Decl::StructDecl {
                    name,
                    is_class,
                    base_classes,
                    fields,
                    methods,
                    private_members,
                    ..
                } => {
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
                Decl::Template(TemplateDecl {
                    params: inner_params,
                    kind: TemplateKind::Class(inner_tmpl),
                    ..
                }) => {
                    self.instantiate_single_nested_template(NestedTemplateParams {
                        inner_tmpl,
                        inner_params,
                        inst_name,
                        mapping,
                    })?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn instantiate_single_nested_template(
        &mut self,
        p: NestedTemplateParams<'_>,
    ) -> Result<(), CompileError> {
        let short_name = p
            .inner_tmpl
            .name
            .rsplit_once("::")
            .map(|(_, s)| s)
            .unwrap_or(&p.inner_tmpl.name);
        let nested_tmpl_name = format!("{}::{}", p.inst_name, short_name);
        let target_ty = Type::Struct(nested_tmpl_name.clone());
        let injected = Some((p.inner_tmpl.name.as_str(), &target_ty));

        let mut subst_fields = Vec::new();
        for (fname, fty, finit) in &p.inner_tmpl.fields {
            let mut init = finit.clone();
            if let Some((init_expr, _, _)) = &mut init {
                template_subst::substitute_expr(init_expr, p.mapping, injected)?;
            }
            let subst_fty = substitute_type(fty, p.mapping, injected)?;
            subst_fields.push((fname.clone(), subst_fty, init));
        }

        let mut subst_methods = Vec::new();
        for m in &p.inner_tmpl.methods {
            let sm = substitute_function(m, p.mapping, injected, None)?;
            subst_methods.push(sm);
        }

        let mut subst_static_fields = Vec::new();
        for (sname, sty, sinit) in &p.inner_tmpl.static_fields {
            let mut init = sinit.clone();
            if let Some(init_expr) = &mut init {
                template_subst::substitute_expr(init_expr, p.mapping, injected)?;
            }
            let subst_sty = substitute_type(sty, p.mapping, injected)?;
            subst_static_fields.push((sname.clone(), subst_sty, init));
        }

        let subst_inner_tmpl = TemplateClass {
            name: nested_tmpl_name.clone(),
            is_class: p.inner_tmpl.is_class,
            is_final: p.inner_tmpl.is_final,
            base_classes: p.inner_tmpl.base_classes.clone(),
            base_access: p.inner_tmpl.base_access.clone(),
            fields: subst_fields,
            methods: subst_methods,
            static_fields: subst_static_fields,
            private_members: p.inner_tmpl.private_members.clone(),
            nested_structs: p.inner_tmpl.nested_structs.clone(),
            alignment: p.inner_tmpl.alignment.clone(),
        };

        self.class_templates.insert(
            nested_tmpl_name.clone(),
            (subst_inner_tmpl.clone(), p.inner_params.to_vec()),
        );
        let plain_nested = QualifiedName::new(&nested_tmpl_name).plain().to_string();
        if plain_nested != nested_tmpl_name {
            self.class_templates
                .insert(plain_nested, (subst_inner_tmpl, p.inner_params.to_vec()));
        }

        Ok(())
    }

    pub(super) fn resolve_class_template(
        &mut self,
        name: &str,
        line: usize,
        col: usize,
    ) -> Result<(TemplateClass, Vec<TemplateParam>), CompileError> {
        if !self.class_templates.contains_key(name)
            && let Some((owner, _)) = name.rsplit_once("::")
        {
            self.ensure_class_instantiated(owner, line, col);
        }
        if !self.class_templates.contains_key(name)
            && let Some(cls) = &self.current_class
        {
            let scoped = format!("{}::{}", cls, name);
            if let Some(res) = self.class_templates.get(&scoped).cloned() {
                return Ok(res);
            }
        }
        self.class_templates
            .get(name)
            .cloned()
            .or_else(|| {
                let plain = QualifiedName::new(name).plain();
                self.class_templates.get(plain).cloned()
            })
            .ok_or_else(|| CompileError::new(format!("Template '{}' not found", name), line, col))
    }

    fn instantiate_single_nested_struct(
        &mut self,
        p: NestedStructParams<'_>,
    ) -> Result<(), CompileError> {
        let (line, col) = p.pos;
        let target_ty = Type::Struct(p.nested_inst_name.to_string());
        let injected = Some((p.name, &target_ty));
        let arg_map = p.mapping;

        let mut subst_fields = Vec::new();
        for (fname, fty, finit) in p.fields {
            let mut ty = substitute_type(fty, arg_map, injected)?;
            self.resolve_and_instantiate_type(&mut ty, line, col)?;
            let mut init = finit.clone();
            if let Some((init_expr, _, _)) = &mut init {
                template_subst::substitute_expr(init_expr, arg_map, injected)?;
            }
            subst_fields.push((fname.clone(), ty, init));
        }

        let mut subst_methods = Vec::new();
        for m in p.methods {
            let mut sm = substitute_function(m, arg_map, injected, None)?;
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
            is_final: false,
            base_classes: p.base_classes,
            base_access: &[],
            fields: &fields_for_layout,
            methods: &methods_for_layout,
            private_members: p.private_members,
            line,
            col,
            alignment: None,
        })?;

        let all_methods = std::mem::take(&mut methods_for_layout);
        let subst_decl = Decl::StructDecl {
            name: p.nested_inst_name.to_string(),
            is_class: p.is_class,
            is_final: false,
            base_classes: p.base_classes.to_vec(),
            base_access: Vec::new(),
            fields: subst_fields,
            methods: subst_methods,
            static_fields: Vec::new(),
            private_members: p.private_members.to_vec(),
            nested_structs: Vec::new(),
            alignment: None,
            line,
            col,
        };

        self.synthesized_decls.push(subst_decl);
        self.setup_instantiated_methods(p.nested_inst_name, all_methods)?;
        Ok(())
    }
}
