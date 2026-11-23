use super::template_deduce::{
    deduce_template_arguments, parse_types_list, split_template_call_name,
};
use super::template_partial_order::{self, FunctionTemplateCand};
use super::template_subst_func::{
    mark_dependent_nested, substitute_function_body_with_sema, substitute_function_signature,
};
use super::*;
use crate::ast::decl::RefQualifier;
use crate::diagnostics::TemplateFailure;
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
                // Packs need arity in their identity. Flattening their elements
                // made an empty pack collide with a single `int` argument and
                // let a later `f<int>` call bind to `f<>`'s cached signature.
                let inner = types
                    .iter()
                    .map(|t| t.to_c_type())
                    .collect::<Vec<_>>()
                    .join(", ");
                pieces.push(format!("pack[{inner}]"));
            }
            Some(TemplateArg::NonTypePack(vals)) => {
                let inner = vals
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                pieces.push(format!("npack[{inner}]"));
            }
            None if p.is_pack => pieces.push("pack[]".to_string()),
            None => pieces.push("int".to_string()),
        }
    }
    pieces.join(", ")
}

struct CandidateInfo<'a> {
    decl: &'a FunctionDecl,
    template_params: &'a [TemplateParam],
    mapping: HashMap<String, TemplateArg>,
    prepared: FunctionDecl,
    active_mapping: HashMap<String, TemplateArg>,
}

struct ViableCandidate {
    decl: FunctionDecl,
    template_params: Vec<TemplateParam>,
    mapping: HashMap<String, TemplateArg>,
    prepared: FunctionDecl,
    active_mapping: HashMap<String, TemplateArg>,
    score: usize,
}

fn select_best_template_candidate(viable: &[ViableCandidate], num_args: usize) -> Option<usize> {
    let mut best_idx = None;
    for (i, v_i) in viable.iter().enumerate() {
        let cand_i = FunctionTemplateCand::new(&v_i.decl, &v_i.template_params);
        let strictly_better = viable.iter().enumerate().all(|(j, v_j)| {
            if i == j {
                return true;
            }
            let cand_j = FunctionTemplateCand::new(&v_j.decl, &v_j.template_params);
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

        let mut candidates = self
            .function_templates
            .get(tmpl_name)
            .cloned()
            .unwrap_or_default();
        if let Some(visible) = self.active_visible_functions() {
            candidates.retain(|(decl, _)| {
                visible.contains(&format!(
                    "T:{}",
                    super::template_registry::function_template_identity(
                        &decl.name, decl.line, decl.col,
                    )
                ))
            });
        }
        // Hidden friend templates are reached through ADL on the argument
        // types, never through the ordinary lookup set above.
        candidates.extend(self.adl_friend_templates(tmpl_name, arg_types));
        if !candidates.is_empty() {
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
                if v.score < min_score {
                    min_score = v.score;
                }
            }
            viable.retain(|v| v.score == min_score);

            if viable.len() == 1 {
                let v = viable.remove(0);
                let sig = self.instantiate_function_template(&v, pos)?;
                return Ok(Some(sig));
            }

            if let Some(idx) = select_best_template_candidate(&viable, num_args) {
                let v = viable.remove(idx);
                let sig = self.instantiate_function_template(&v, pos)?;
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
        let mut rejections = super::template_func_helpers::Rejections::default();
        let mut viable = Vec::new();
        for (template_decl, template_params) in candidates {
            let num_args = if args.is_empty() {
                arg_types.len()
            } else {
                args.len()
            };
            if !Self::candidate_arity_ok(template_decl, template_params, num_args) {
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
                    let (sig, prepared, active_mapping) = match self
                        .substitute_template_function_sig(
                            &mapping,
                            template_params,
                            template_decl,
                            pos,
                        ) {
                        Ok(formed) => formed,
                        Err(TemplateFailure::Substitution(failure)) => {
                            let failure = failure.at_call(pos.line, pos.col);
                            rejections
                                .messages
                                .push(format!("substitution failure: {}", failure.message));
                            rejections.last_substitution = Some(failure);
                            continue;
                        }
                        Err(TemplateFailure::Hard(error)) => return Err(error),
                    };
                    match self.score_arguments(&sig.param_types, arg_types, Some(args)) {
                        Ok(score) => {
                            viable.push(Self::make_candidate(
                                CandidateInfo {
                                    decl: template_decl,
                                    template_params,
                                    mapping,
                                    prepared,
                                    active_mapping,
                                },
                                score,
                            ));
                        }
                        Err(failure) => {
                            let error =
                                self.no_matching_conversion(&template_decl.name, &failure, pos);
                            rejections.messages.push(error.message.clone());
                            rejections.last_err = Some(error);
                        }
                    }
                }
                Err(e) => {
                    rejections.messages.push(e.message.clone());
                    rejections.last_err = Some(e);
                }
            }
        }
        if viable.is_empty() {
            rejections.into_error(candidates, pos)?;
        }
        Ok(viable)
    }

    fn make_candidate(info: CandidateInfo<'_>, score: usize) -> ViableCandidate {
        ViableCandidate {
            decl: info.decl.clone(),
            template_params: info.template_params.to_vec(),
            mapping: info.mapping,
            prepared: info.prepared,
            active_mapping: info.active_mapping,
            score,
        }
    }

    fn candidate_arity_ok(
        template_decl: &FunctionDecl,
        template_params: &[TemplateParam],
        num_args: usize,
    ) -> bool {
        let has_pack = template_decl.params.last().is_some_and(|p| p.is_pack)
            || template_params.last().is_some_and(|p| p.is_pack);
        let min_args = template_decl
            .params
            .iter()
            .filter(|p| !p.is_pack && p.default_val.is_none())
            .count();
        let max_args = template_decl.params.len();
        !(num_args < min_args || (!has_pack && num_args > max_args))
    }

    pub fn substitute_template_function_sig(
        &mut self,
        mapping: &HashMap<String, TemplateArg>,
        template_params: &[TemplateParam],
        template_decl: &FunctionDecl,
        pos: ExprPos,
    ) -> Result<(FunctionSig, FunctionDecl, HashMap<String, TemplateArg>), TemplateFailure> {
        let canonical_args = format_canonical_args(template_params, mapping);
        let inst_name = format!("{}<{}>", template_decl.name, canonical_args);
        let (mut func_decl, active_mapping) =
            substitute_function_signature(template_decl, mapping, Some(inst_name), Some(self))?;

        // A default template argument is part of the signature. Its dependent
        // nested types must resolve even when the argument is unused in it.
        for param in template_params {
            if let Some(def) = &param.default_type
                && let Some(TemplateArg::Type(arg)) = mapping.get(&param.name)
            {
                let mut resolved = arg.clone();
                mark_dependent_nested(def, &mut resolved, mapping);
                self.resolve_signature_type(&mut resolved, pos.line, pos.col)?;
            }
        }
        // Calls inside a signature's `decltype` also belong to the template
        // definition's lookup context.
        let identity = super::template_registry::function_template_identity(
            &template_decl.name,
            template_decl.line,
            template_decl.col,
        );
        let saved_identity = self.current_template_identity.take();
        self.current_template_identity = Some(identity);
        let sig_result = self.resolve_signature_types(&mut func_decl, pos.line, pos.col);
        self.current_template_identity = saved_identity;
        sig_result?;

        let mangled = self.mangle_name(&func_decl.name, &func_decl.params);
        let sig = FunctionSig {
            qualifiers: func_decl.qualifiers,
            name: func_decl.name.clone(),
            mangled_name: mangled,
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
        Ok((sig, func_decl, active_mapping))
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

    fn instantiate_function_template(
        &mut self,
        cand: &ViableCandidate,
        pos: ExprPos,
    ) -> Result<FunctionSig, CompileError> {
        if self.instantiation_depth >= 500 {
            return Err(CompileError::new(
                "template instantiation depth exceeded (limit: 500)",
                pos.line,
                pos.col,
            ));
        }

        let canonical_args = format_canonical_args(&cand.template_params, &cand.mapping);
        let inst_name = format!("{}<{}>", cand.decl.name, canonical_args);
        let identity = super::template_registry::function_template_identity(
            &cand.decl.name,
            cand.decl.line,
            cand.decl.col,
        );
        let mut func_decl = cand.prepared.clone();
        substitute_function_body_with_sema(&mut func_decl, &cand.active_mapping, None, Some(self))
            .map_err(|mut error| {
                if error.line == 0 && error.col == 0 {
                    error.line = pos.line;
                    error.col = pos.col;
                }
                error
            })?;

        let mangled = self.mangle_name(&func_decl.name, &func_decl.params);
        let sig = FunctionSig {
            qualifiers: func_decl.qualifiers,
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
            // The marker is visible to recursive calls while the body is
            // analyzed; it is removed again if analysis fails so a later
            // call can retry the specialization.
            self.instantiated_functions.insert(mangled.clone());
            self.instantiation_depth += 1;
            let saved_identity = self.current_template_identity.take();
            self.current_template_identity = Some(identity);
            let res = self.analyze_function(&mut func_decl);
            self.current_template_identity = saved_identity;
            self.instantiation_depth -= 1;
            if let Err(error) = res {
                if self.speculating {
                    self.speculative_hard = true;
                }
                self.instantiated_functions.remove(&mangled);
                return Err(error);
            }
            self.synthesized_decls.push(Decl::Function(func_decl));
            self.register_function(sig.clone());
            self.functions
                .entry(inst_name)
                .or_default()
                .push(sig.clone());
            self.functions
                .entry(cand.decl.name.clone())
                .or_default()
                .push(sig.clone());
            self.register_function_template_aliases(&cand.decl.name, &sig);
        }

        Ok(sig)
    }

    fn register_function_template_aliases(&mut self, name: &str, sig: &FunctionSig) {
        if let Some(alias) = QualifiedName::new(name).implicit_alias() {
            self.functions
                .entry(alias.into_owned())
                .or_default()
                .push(sig.clone());
        }
    }
}
