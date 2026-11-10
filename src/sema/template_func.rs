use super::template_deduce::{
    deduce_template_arguments, parse_types_list, split_template_call_name,
};
use super::template_subst::substitute_function;
use super::*;
use crate::ast::decl::RefQualifier;

impl Sema {
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

        if let Some(candidates) = self.function_templates.get(tmpl_name).cloned() {
            let mut last_err = None;
            for (template_decl, template_params) in &candidates {
                let min_args = template_decl
                    .params
                    .iter()
                    .filter(|p| p.default_val.is_none())
                    .count();
                let max_args = template_decl.params.len();
                if args.len() < min_args || args.len() > max_args {
                    continue;
                }
                match deduce_template_arguments(
                    template_params,
                    template_decl,
                    &explicit_args,
                    (arg_types, args),
                    pos,
                ) {
                    Ok(mapping) => {
                        let sig = self.instantiate_function_template(
                            &mapping,
                            template_params,
                            template_decl,
                            pos,
                        )?;
                        if let Err(failure) =
                            self.score_arguments(&sig.param_types, arg_types, Some(args))
                        {
                            last_err = Some(self.no_matching_conversion(
                                &template_decl.name,
                                &failure,
                                pos,
                            ));
                            continue;
                        }
                        return Ok(Some(sig));
                    }
                    Err(e) => last_err = Some(e),
                }
            }
            if let Some(e) = last_err {
                return Err(e);
            }
        }
        Ok(None)
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
