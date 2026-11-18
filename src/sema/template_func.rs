use super::template_deduce::{
    deduce_template_arguments, parse_types_list, split_template_call_name,
};
use super::template_partial_order::{self, FunctionTemplateCand};
use super::template_subst::substitute_function;
use super::template_subst_func::substitute_function_signature;
use super::*;
use crate::ast::decl::RefQualifier;
use crate::diagnostics::{SubstitutionFailure, TemplateFailure};
use crate::sema::pack_expansion::TemplateArg;

pub(crate) fn format_canonical_args(
    template_params: &[TemplateParam],
    mapping: &HashMap<String, TemplateArg>,
) -> String {
    let mut pieces = Vec::new();
    for p in template_params {
        match mapping.get(&p.name) {
            Some(TemplateArg::Type(t)) => pieces.push(t.to_c_type()),
            Some(TemplateArg::NonType(v)) => pieces.push(v.to_string()),
            Some(TemplateArg::NonTypeAddress(addr)) => pieces.push(format!("&{addr}")),
            Some(TemplateArg::NonTypeNull(_)) => pieces.push("nullptr".to_string()),
            Some(TemplateArg::TypePack(types)) => {
                for t in types {
                    pieces.push(t.to_c_type());
                }
            }
            Some(TemplateArg::NonTypePack(vals)) => {
                for v in vals {
                    pieces.push(v.to_string());
                }
            }
            None => pieces.push("int".to_string()),
        }
    }
    pieces.join(", ")
}

type ViableCandidate = (
    FunctionDecl,
    Vec<TemplateParam>,
    HashMap<String, TemplateArg>,
    FunctionSig,
    usize,
);

fn select_best_template_candidate(viable: &[ViableCandidate], num_args: usize) -> Option<usize> {
    let mut best_idx = None;
    for (i, v_i) in viable.iter().enumerate() {
        let cand_i = FunctionTemplateCand::new(&v_i.0, &v_i.1);
        let strictly_better = viable.iter().enumerate().all(|(j, v_j)| {
            if i == j {
                return true;
            }
            let cand_j = FunctionTemplateCand::new(&v_j.0, &v_j.1);
            template_partial_order::compare_function_templates(&cand_i, &cand_j, num_args)
                == Some(std::cmp::Ordering::Less)
        });
        if strictly_better {
            if best_idx.is_some() {
                return None;
            }
            best_idx = Some(i);
        }
    }
    best_idx
}

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
            let mut viable = self.collect_viable_template_candidates(
                &candidates,
                &explicit_args,
                (arg_types, args),
                pos,
            )?;
            if viable.is_empty() {
                return Ok(None);
            }

            let num_args = if args.is_empty() {
                arg_types.len()
            } else {
                args.len()
            };

            let mut min_score = usize::MAX;
            for v in &viable {
                if v.4 < min_score {
                    min_score = v.4;
                }
            }
            viable.retain(|v| v.4 == min_score);

            if viable.len() == 1 {
                let v = viable.remove(0);
                let sig = self.instantiate_function_template(&v.2, &v.1, &v.0, pos)?;
                return Ok(Some(sig));
            }

            if let Some(idx) = select_best_template_candidate(&viable, num_args) {
                let v = viable.remove(idx);
                let sig = self.instantiate_function_template(&v.2, &v.1, &v.0, pos)?;
                return Ok(Some(sig));
            }
            return Err(self.ambiguous_call(tmpl_name, arg_types, pos));
        }
        Ok(None)
    }

    fn collect_viable_template_candidates(
        &mut self,
        candidates: &[(FunctionDecl, Vec<TemplateParam>)],
        explicit_args: &[Type],
        call_args: (&[Type], &[Expr]),
        pos: ExprPos,
    ) -> Result<Vec<ViableCandidate>, CompileError> {
        let (arg_types, args) = call_args;
        let mut last_err = None;
        let mut last_substitution: Option<SubstitutionFailure> = None;
        let mut rejections: Vec<String> = Vec::new();
        let mut viable = Vec::new();
        for (template_decl, template_params) in candidates {
            let has_pack = template_decl.params.last().is_some_and(|p| p.is_pack)
                || template_params.last().is_some_and(|p| p.is_pack);
            let min_args = template_decl
                .params
                .iter()
                .filter(|p| !p.is_pack && p.default_val.is_none())
                .count();
            let max_args = template_decl.params.len();
            let num_args = if args.is_empty() {
                arg_types.len()
            } else {
                args.len()
            };
            if num_args < min_args || (!has_pack && num_args > max_args) {
                continue;
            }
            match deduce_template_arguments(
                template_params,
                template_decl,
                explicit_args,
                (arg_types, args),
                pos,
            ) {
                Ok(mapping) => {
                    let sig = match self.substitute_template_function_sig(
                        &mapping,
                        template_params,
                        template_decl,
                        pos,
                    ) {
                        Ok(sig) => sig,
                        Err(TemplateFailure::Substitution(failure)) => {
                            let failure = failure.at_call(pos.line, pos.col);
                            rejections.push(format!("substitution failure: {}", failure.message));
                            last_substitution = Some(failure);
                            continue;
                        }
                        Err(TemplateFailure::Hard(error)) => return Err(error),
                    };
                    match self.score_arguments(&sig.param_types, arg_types, Some(args)) {
                        Ok(score) => {
                            viable.push((
                                template_decl.clone(),
                                template_params.clone(),
                                mapping,
                                sig,
                                score,
                            ));
                        }
                        Err(failure) => {
                            let error =
                                self.no_matching_conversion(&template_decl.name, &failure, pos);
                            rejections.push(error.message.clone());
                            last_err = Some(error);
                        }
                    }
                }
                Err(e) => {
                    rejections.push(e.message.clone());
                    last_err = Some(e);
                }
            }
        }
        if viable.is_empty() {
            if rejections.len() > 1 {
                let name = candidates
                    .first()
                    .map(|(decl, _)| decl.name.as_str())
                    .unwrap_or("template function");
                return Err(CompileError::new(
                    format!(
                        "no matching function for call to '{name}': {} candidates rejected: {}",
                        rejections.len(),
                        rejections.join("; ")
                    ),
                    pos.line,
                    pos.col,
                ));
            }
            if let Some(e) = last_err {
                return Err(e);
            }
            if let Some(failure) = last_substitution {
                let name = candidates
                    .first()
                    .map(|(decl, _)| decl.name.as_str())
                    .unwrap_or("template function");
                return Err(CompileError::new(
                    format!(
                        "no matching function for call to '{name}': substitution failure: {}",
                        failure.message
                    ),
                    failure.line,
                    failure.col,
                ));
            }
        }
        Ok(viable)
    }

    pub fn substitute_template_function_sig(
        &mut self,
        mapping: &HashMap<String, TemplateArg>,
        template_params: &[TemplateParam],
        template_decl: &FunctionDecl,
        pos: ExprPos,
    ) -> Result<FunctionSig, TemplateFailure> {
        let canonical_args = format_canonical_args(template_params, mapping);
        let inst_name = format!("{}<{}>", template_decl.name, canonical_args);
        let (mut func_decl, _) =
            substitute_function_signature(template_decl, mapping, Some(inst_name))?;

        // Resolving a nested type can instantiate another template. Errors from
        // that side effect are outside the immediate context and stay hard.
        self.resolve_and_instantiate_type(&mut func_decl.return_type, pos.line, pos.col)?;
        for p in &mut func_decl.params {
            self.resolve_and_instantiate_type(&mut p.ty, pos.line, pos.col)?;
        }

        let mangled = self.mangle_name(&func_decl.name, &func_decl.params);
        Ok(FunctionSig {
            qualifiers: FunctionQualifiers::default(),
            name: func_decl.name,
            mangled_name: mangled,
            return_type: func_decl.return_type,
            param_types: func_decl.params.iter().map(|p| p.ty.clone()).collect(),
            default_args: func_decl
                .params
                .iter()
                .map(|p| p.default_val.clone())
                .collect(),
            is_static: false,
            is_explicit: false,
            ref_qualifier: RefQualifier::None,
        })
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
        mapping: &HashMap<String, TemplateArg>,
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

        let canonical_args = format_canonical_args(template_params, mapping);
        let inst_name = format!("{}<{}>", template_decl.name, canonical_args);
        let mut func_decl =
            substitute_function(template_decl, mapping, None, Some(inst_name.clone()))?;

        self.resolve_and_instantiate_type(&mut func_decl.return_type, pos.line, pos.col)?;
        for p in &mut func_decl.params {
            self.resolve_and_instantiate_type(&mut p.ty, pos.line, pos.col)?;
        }

        let mangled = self.mangle_name(&func_decl.name, &func_decl.params);
        let sig = FunctionSig {
            qualifiers: FunctionQualifiers::default(),
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
            if let Some(alias) = QualifiedName::new(&template_decl.name).implicit_alias() {
                self.functions
                    .entry(alias.into_owned())
                    .or_default()
                    .push(sig.clone());
            }
        }

        Ok(sig)
    }
}
