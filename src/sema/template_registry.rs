//! Template registration, lookup, instantiation caching, and lazy member instantiation.

use super::analyze::this_param;
use super::template_deduce::{
    deduce_template_arguments, parse_types_list, split_template_call_name,
};
use super::template_subst::{substitute_function, substitute_struct};
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
        insert_template_aliases(
            &mut self.function_templates,
            decl.name.clone(),
            (decl, params),
        );
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
                if let Some(existing) = template_class
                    .methods
                    .iter_mut()
                    .find(|m| m.name == short_method && m.params.len() == func.params.len())
                {
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
                let inst_name = self.instantiate_class_template(name, args, line, col)?;
                *ty = Type::Struct(inst_name);
            }
            Type::Pointer(inner)
            | Type::Reference(inner)
            | Type::Const(inner)
            | Type::Array(inner, _) => {
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
                        .push(m);
                }
                continue;
            }
            self.register_struct_method(inst_name, &m);
            if m.is_virtual || m.name.starts_with('~') {
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

    pub fn try_resolve_function_template_call(
        &mut self,
        name: &str,
        arg_types: &[Type],
        args: &[Expr],
        pos: ExprPos,
    ) -> Result<Option<FunctionSig>, CompileError> {
        let (tmpl_name, explicit_args) =
            if let Some((base, args_str)) = split_template_call_name(name) {
                let known: Vec<String> = self.class_templates.keys().cloned().collect();
                let mut explicit_args = parse_types_list(args_str, &known)?;
                for arg in explicit_args.iter_mut() {
                    self.resolve_and_instantiate_type(arg, pos.line, pos.col)?;
                }
                (base, explicit_args)
            } else {
                (name, Vec::new())
            };

        if let Some((template_decl, template_params)) =
            self.function_templates.get(tmpl_name).cloned()
        {
            let mapping = deduce_template_arguments(
                &template_params,
                &template_decl,
                &explicit_args,
                (arg_types, args),
                pos,
            )?;
            let sig = self.instantiate_function_template(
                &mapping,
                &template_params,
                &template_decl,
                pos,
            )?;
            return Ok(Some(sig));
        }
        Ok(None)
    }

    pub fn instantiate_function_template(
        &mut self,
        mapping: &HashMap<String, Type>,
        template_params: &[TemplateParam],
        template_decl: &FunctionDecl,
        pos: ExprPos,
    ) -> Result<FunctionSig, CompileError> {
        if self.instantiation_depth >= 500 {
            return Err(CompileError::new(
                "template instantiation depth exceeded (limit: 500)",
                pos.line,
                pos.col,
            ));
        }

        let canonical_args = template_params
            .iter()
            .map(|p| {
                mapping
                    .get(&p.name)
                    .map(|t| t.to_c_type())
                    .unwrap_or_else(|| "int".to_string())
            })
            .collect::<Vec<_>>()
            .join(", ");
        let inst_name = format!("{}<{}>", template_decl.name, canonical_args);
        let mut func_decl =
            substitute_function(template_decl, mapping, None, Some(inst_name.clone()));

        self.resolve_and_instantiate_type(&mut func_decl.return_type, pos.line, pos.col)?;
        for p in &mut func_decl.params {
            self.resolve_and_instantiate_type(&mut p.ty, pos.line, pos.col)?;
        }

        let mangled = self.mangle_name(&func_decl.name, &func_decl.params);
        let sig = FunctionSig {
            name: func_decl.name.clone(),
            mangled_name: mangled.clone(),
            return_type: func_decl.return_type.clone(),
            param_types: func_decl.params.iter().map(|p| p.ty.clone()).collect(),
            default_args: func_decl
                .params
                .iter()
                .map(|p| p.default_val.clone())
                .collect(),
            is_static: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
        };

        if !self.instantiated_functions.contains(&mangled) {
            self.instantiated_functions.insert(mangled.clone());
            self.instantiation_depth += 1;
            let res = self.analyze_function(&mut func_decl);
            self.instantiation_depth -= 1;
            res?;
            self.synthesized_decls.push(Decl::Function(func_decl));
            self.register_function(sig.clone());
            self.functions
                .entry(inst_name)
                .or_default()
                .push(sig.clone());
            self.functions
                .entry(template_decl.name.clone())
                .or_default()
                .push(sig.clone());
        }

        Ok(sig)
    }
}
