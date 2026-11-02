use super::template_deduce::{deduce_template_arguments, parse_types_list, split_template_call_name};
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
