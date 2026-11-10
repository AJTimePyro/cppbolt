//! Template registration, lookup, instantiation caching, and lazy member instantiation.

use super::analyze::this_param;
use super::template_deduce::{parse_types_list, split_template_call_name};
use super::template_subst::{substitute_struct, substitute_type};
use super::*;
use crate::ast::decl::{Decl, FunctionDecl};
use crate::ast::expr::Expr;
use crate::ast::template::{TemplateClass, TemplateParam};
use crate::ast::types::Type;
use crate::name::QualifiedName;
use std::collections::HashMap;

fn insert_template_aliases<T: Clone>(map: &mut HashMap<String, T>, name: String, item: T) {
    if let Some(short) = name.split("::").last() {
        map.insert(short.to_string(), item.clone());
    }
    let qualified = QualifiedName::new(&name);
    if qualified.is_global() {
        map.insert(qualified.std_name().into_owned(), item.clone());
    } else if qualified.in_std() {
        map.insert(qualified.plain().to_string(), item.clone());
    }
    map.insert(name, item);
}

impl Sema {
    pub fn register_class_template(&mut self, decl: TemplateClass, params: Vec<TemplateParam>) {
        insert_template_aliases(&mut self.class_templates, decl.name.clone(), (decl, params));
    }

    pub fn register_function_template(&mut self, decl: FunctionDecl, params: Vec<TemplateParam>) {
        let short = decl.name.rsplit("::").next().unwrap_or(&decl.name).to_string();
        let q = QualifiedName::new(&decl.name);
        let keys = [decl.name.clone(), short, q.plain().to_string(), q.std_name().into_owned()];
        let item = (decl, params);
        for key in keys {
            let list = self.function_templates.entry(key).or_default();
            if let Some((existing_decl, _)) = list.iter_mut().find(|(d, _)| {
                d.params.len() == item.0.params.len()
                    && d.params
                        .iter()
                        .zip(&item.0.params)
                        .all(|(p1, p2)| p1.ty == p2.ty)
            }) {
                if existing_decl.body.is_none() && item.0.body.is_some() {
                    existing_decl.body = item.0.body.clone();
                }
            } else {
                list.push(item.clone());
            }
        }
    }

    pub fn attach_out_of_class_template_method(
        &mut self,
        func: FunctionDecl,
        params: &[TemplateParam],
    ) {
        let Some((class_part, method_name)) = func.name.rsplit_once("::") else {
            self.register_function_template(func, params.to_vec());
            return;
        };
        let class_name = class_part.split('<').next().unwrap_or(class_part);
        let plain_class = QualifiedName::new(class_name).plain();
        let short_method = method_name.to_string();

        let mut matched = false;
        for (key, (template_class, _)) in self.class_templates.iter_mut() {
            if QualifiedName::new(key).plain() == plain_class {
                matched = true;
                if let Some(existing) = template_class.methods.iter_mut().find(|m| {
                    m.name == short_method
                        && m.params.len() == func.params.len()
                        && m.is_const == func.is_const
                        && m.body.is_none()
                        && m.params
                            .iter()
                            .zip(&func.params)
                            .all(|(p1, p2)| p1.ty == p2.ty)
                }) {
                    existing.body = func.body.clone();
                    if !func.ctor_initializers.is_empty() {
                        existing.ctor_initializers = func.ctor_initializers.clone();
                    }
                } else {
                    let mut m = func.clone();
                    m.name = short_method.clone();
                    template_class.methods.push(m);
                }
            }
        }
        if !matched {
            self.register_function_template(func, params.to_vec());
        }
    }

    pub fn resolve_and_instantiate_type(
        &mut self,
        ty: &mut Type,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        match ty {
            Type::TemplateInstance { name, args } => {
                for arg in args.iter_mut() {
                    self.resolve_and_instantiate_type(arg, line, col)?;
                }
                if let Some(expanded) = self.expand_alias_template(name, args, line, col)? {
                    *ty = expanded;
                    return Ok(());
                }
                let inst_name = self.instantiate_class_template(name, args, line, col)?;
                *ty = Type::Struct(inst_name);
            }
            Type::Struct(sname) => {
                let sname_clone = sname.clone();
                self.resolve_struct_type_alias(ty, &sname_clone, line, col)?;
                if let Type::Struct(resolved_name) = ty {
                    self.check_nested_type_access(&sname_clone, line, col)?;
                    if resolved_name != &sname_clone {
                        self.check_nested_type_access(resolved_name, line, col)?;
                    }
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
            _ => {}
        }
        Ok(())
    }

    fn resolve_struct_type_alias(
        &mut self,
        ty: &mut Type,
        sname: &str,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let plain = QualifiedName::new(sname).plain();
        if let Some(alias) = self
            .type_aliases
            .get(sname)
            .cloned()
            .or_else(|| self.type_aliases.get(plain).cloned())
        {
            *ty = alias;
            return Ok(());
        }
        if let Some(gt_pos) = sname.rfind('>')
            && sname[gt_pos..].starts_with(">::")
        {
            let inst_part = &sname[..gt_pos + 1];
            if let Some((tmpl_name, args_str)) = split_template_call_name(inst_part) {
                let known: Vec<String> = self
                    .class_templates
                    .keys()
                    .chain(self.alias_templates.keys())
                    .cloned()
                    .collect();
                if let Ok(mut args) = parse_types_list(args_str, &known) {
                    for arg in &mut args {
                        self.resolve_and_instantiate_type(arg, line, col)?;
                    }
                    let inst = self.instantiate_class_template(tmpl_name, &args, line, col)?;
                    let member = &sname[gt_pos + 3..];
                    let canon = format!("{inst}::{member}");
                    if let Some(alias) = self
                        .type_aliases
                        .get(&canon)
                        .cloned()
                        .or_else(|| self.type_aliases.get(sname).cloned())
                        .or_else(|| self.type_aliases.get(plain).cloned())
                    {
                        *ty = alias;
                    } else if self.struct_layouts.contains_key(&canon) {
                        *ty = Type::Struct(canon);
                    }
                }
            }
        }
        Ok(())
    }

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

        let (template_class, template_params) = self
            .class_templates
            .get(name)
            .cloned()
            .or_else(|| {
                let plain = QualifiedName::new(name).plain();
                self.class_templates.get(plain).cloned()
            })
            .ok_or_else(|| {
                CompileError::new(format!("Template '{}' not found", name), line, col)
            })?;

        let effective_args = super::template_deduce::fill_default_template_args(
            name,
            &template_params,
            args,
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

        if self.instantiated_classes.contains(&inst_name) {
            return Ok(inst_name);
        }

        self.instantiated_classes.insert(inst_name.clone());
        self.instantiation_stack.push(inst_name.clone());
        self.instantiation_depth += 1;

        let (target_class, mapping) = self.resolve_instantiation_target(
            super::template_spec::InstantiationTarget {
                name,
                inst_name: &inst_name,
                template_params: &template_params,
                effective_args: &effective_args,
                pos: (line, col),
            },
            template_class,
        )?;

        let result = self.execute_class_instantiation(
            &target_class,
            mapping,
            &inst_name,
            (line, col),
        );

        self.instantiation_stack.pop();
        self.instantiation_depth -= 1;
        result.map(|_| inst_name)
    }

    fn execute_class_instantiation(
        &mut self,
        template_class: &TemplateClass,
        mapping: HashMap<String, Type>,
        inst_name: &str,
        pos: (usize, usize),
    ) -> Result<(), CompileError> {
        let (line, col) = pos;
        self.instantiate_nested_structs(template_class, inst_name, &mapping, pos)?;

        let mut struct_decl = substitute_struct(template_class, &mapping, inst_name, line, col);
        let Decl::StructDecl {
            fields,
            methods,
            static_fields,
            base_classes,
            is_class,
            private_members,
            ..
        } = &mut struct_decl
        else {
            return Ok(());
        };

        for (_, fty, _) in fields.iter_mut() {
            self.resolve_and_instantiate_type(fty, line, col)?;
        }

        for m in methods.iter_mut() {
            if m.template_params.is_empty() {
                self.resolve_and_instantiate_type(&mut m.return_type, line, col)?;
                for p in &mut m.params {
                    self.resolve_and_instantiate_type(&mut p.ty, line, col)?;
                }
            }
        }

        self.synthesize_instantiation_lifecycle(inst_name, base_classes, fields, methods);

        self.compute_struct_layout(RecordLayout {
            name: inst_name, is_class: *is_class, base_classes,
            fields, methods, private_members, line, col,
        })?;

        self.register_instantiated_static_fields(inst_name, static_fields, base_classes, line, col);
        self.register_instantiated_member_aliases(&template_class.name, inst_name, &mapping);
        let all_methods = std::mem::take(methods);
        self.synthesized_decls.push(struct_decl);
        self.setup_instantiated_methods(inst_name, all_methods)?;
        Ok(())
    }

    fn register_instantiated_static_fields(
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
                    name: qual_name,
                    ty: sf.1.clone(),
                    is_global: true,
                    stack_offset: 0,
                },
            );
        }
        for base in base_classes {
            self.ensure_class_instantiated(base, line, col);
            self.alias_static_fields(base, inst_name);
        }
    }

    fn register_instantiated_member_aliases(
        &mut self,
        tmpl_name: &str,
        inst_name: &str,
        mapping: &HashMap<String, Type>,
    ) {
        let plain_tmpl = QualifiedName::new(tmpl_name).plain();
        let plain_inst = QualifiedName::new(inst_name).plain();
        let is_std =
            QualifiedName::new(inst_name).in_std() || QualifiedName::new(tmpl_name).in_std();
        let mut new_aliases = Vec::new();
        for (alias_name, ty) in &self.type_aliases {
            let Some((owner, member)) = alias_name.rsplit_once("::") else {
                continue;
            };
            if QualifiedName::new(owner).plain() == plain_tmpl {
                let subst_ty = substitute_type(ty, mapping, None);
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

    pub(super) fn setup_instantiated_methods(
        &mut self,
        inst_name: &str,
        all_methods: Vec<FunctionDecl>,
    ) -> Result<(), CompileError> {
        for m in &all_methods {
            if m.template_params.is_empty() {
                self.register_struct_method(inst_name, m);
            }
        }
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
                res?;
                m.body = vmethod.body;
                m.ctor_initializers = vmethod.ctor_initializers;
                for decl in &mut self.synthesized_decls {
                    if let Decl::StructDecl { name, methods, .. } = decl
                        && name == inst_name
                    {
                        methods.push(m);
                        break;
                    }
                }
            }
        }
        Ok(())
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
            let res = self.analyze_function(&mut func_decl);
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
                    name: class_name, is_class: false, base_classes: vec![],
                    fields: vec![], methods: vec![method_decl], static_fields: vec![],
                    private_members: vec![], nested_structs: vec![], line: 0, col: 0,
                });
            }
        }
        Ok(())
    }
}
