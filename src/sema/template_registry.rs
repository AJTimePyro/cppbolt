//! Template registration, lookup, instantiation caching, and lazy member instantiation.

use super::analyze::this_param;
use super::template_subst::substitute_struct;
use super::*;
use crate::ast::decl::{Decl, FunctionDecl};
use crate::ast::template::{TemplateClass, TemplateParam};
use crate::ast::types::Type;
use crate::name::QualifiedName;
use std::collections::HashMap;

impl Sema {
    pub fn register_class_template(&mut self, decl: TemplateClass, params: Vec<TemplateParam>) {
        let name = decl.name.clone();
        self.class_templates
            .insert(name.clone(), (decl.clone(), params.clone()));
        if let Some(short) = name.split("::").last() {
            self.class_templates
                .insert(short.to_string(), (decl.clone(), params.clone()));
        }
        let qualified = QualifiedName::new(&name);
        if qualified.is_global() {
            self.class_templates.insert(
                qualified.std_name().into_owned(),
                (decl.clone(), params.clone()),
            );
        } else if qualified.in_std() {
            self.class_templates
                .insert(qualified.plain().to_string(), (decl, params));
        }
    }

    pub fn register_function_template(&mut self, decl: FunctionDecl, params: Vec<TemplateParam>) {
        let name = decl.name.clone();
        self.function_templates
            .insert(name.clone(), (decl.clone(), params.clone()));
        if let Some(short) = name.split("::").last() {
            self.function_templates
                .insert(short.to_string(), (decl.clone(), params.clone()));
        }
        let qualified = QualifiedName::new(&name);
        if qualified.is_global() {
            self.function_templates.insert(
                qualified.std_name().into_owned(),
                (decl.clone(), params.clone()),
            );
        } else if qualified.in_std() {
            self.function_templates
                .insert(qualified.plain().to_string(), (decl, params));
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
                let canonical_args = args
                    .iter()
                    .map(|a| a.to_c_type())
                    .collect::<Vec<_>>()
                    .join(", ");
                let plain_name = QualifiedName::new(name).plain();
                let inst_name = format!("{}<{}>", plain_name, canonical_args);
                if !self.instantiated_classes.contains(&inst_name) {
                    self.instantiate_class_template(name, args, line, col)?;
                }
                *ty = Type::Struct(inst_name);
            }
            Type::Const(inner) => {
                self.resolve_and_instantiate_type(inner, line, col)?;
            }
            Type::Pointer(inner) | Type::Reference(inner) => {
                if let Type::TemplateInstance { name, args } = &mut **inner {
                    for arg in args.iter_mut() {
                        self.resolve_and_instantiate_type(arg, line, col)?;
                    }
                    let canonical_args = args
                        .iter()
                        .map(|a| a.to_c_type())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let plain_name = QualifiedName::new(name).plain();
                    **inner = Type::Struct(format!("{}<{}>", plain_name, canonical_args));
                }
            }
            Type::Array(inner, _) => {
                self.resolve_and_instantiate_type(inner, line, col)?;
            }
            _ => {}
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

        if args.len() != template_params.len() {
            return Err(CompileError::new(
                format!(
                    "wrong number of template arguments for '{}': expected {}, found {}",
                    name,
                    template_params.len(),
                    args.len()
                ),
                line,
                col,
            ));
        }

        if self.instantiation_depth >= 500 {
            return Err(CompileError::new(
                "template instantiation depth exceeded (limit: 500)",
                line,
                col,
            ));
        }

        let canonical_args = args
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

        let result = self.execute_class_instantiation(
            &template_class,
            &template_params,
            args,
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
        template_params: &[TemplateParam],
        args: &[Type],
        inst_name: &str,
        pos: (usize, usize),
    ) -> Result<(), CompileError> {
        let (line, col) = pos;
        let mut mapping = HashMap::new();
        for (param, arg) in template_params.iter().zip(args.iter()) {
            mapping.insert(param.name.clone(), arg.clone());
        }

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

        for (_, fty) in fields.iter_mut() {
            self.resolve_and_instantiate_type(fty, line, col)?;
        }

        self.compute_struct_layout(RecordLayout {
            name: inst_name,
            is_class: *is_class,
            base_classes,
            fields,
            methods,
            private_members,
            line,
            col,
        })?;

        self.register_instantiated_static_fields(inst_name, static_fields);
        let all_methods = std::mem::take(methods);
        self.setup_instantiated_methods(inst_name, all_methods, methods)?;

        self.synthesized_decls.push(struct_decl);
        Ok(())
    }

    fn register_instantiated_static_fields(
        &mut self,
        inst_name: &str,
        static_fields: &[(String, Type, Option<Expr>)],
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
    }

    fn setup_instantiated_methods(
        &mut self,
        inst_name: &str,
        all_methods: Vec<FunctionDecl>,
        methods: &mut Vec<FunctionDecl>,
    ) -> Result<(), CompileError> {
        for mut m in all_methods {
            self.register_struct_method(inst_name, &m);
            if m.is_virtual {
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
                methods.push(m);
            } else {
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
                    .insert(mangled, (inst_name.to_string(), m));
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
            for decl in &mut self.synthesized_decls {
                if let Decl::StructDecl { name, methods, .. } = decl
                    && name == &class_name
                {
                    methods.push(method_decl);
                    break;
                }
            }
        }
        Ok(())
    }
}
