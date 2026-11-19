use super::call_arity::check_indirect_call_arity;
use super::constructor::ConstructorCall;
use super::overload::CandidateSelection;
use super::template_deduce::split_template_call_name;
use super::*;
/// The return type of a function pointer or of a function type.
fn function_return_type(ty: &Type) -> Option<Type> {
    match ty.strip_const() {
        Type::Pointer(inner) => function_return_type(inner),
        Type::Function { return_type, .. } => Some((**return_type).clone()),
        Type::BoundMemberFunction { member_type, .. } => function_return_type(member_type),
        _ => None,
    }
}

impl Sema {
    pub(super) fn analyze_call(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::Call {
            callee,
            args,
            pos,
            dependent,
        } = expr
        else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        let mut arg_types = Vec::new();
        for a in args.iter_mut() {
            arg_types.push(self.analyze_expr_keeping_reference(a)?);
        }

        if let Some((ty, replacement)) =
            self.try_resolve_identifier_call(callee, args, &arg_types, *pos)?
        {
            if let Some(rep) = replacement {
                *expr = rep;
            }
            return Ok(ty);
        }

        if let Some(return_type) = self.analyze_member_callee(callee, args, &arg_types)? {
            return Ok(return_type);
        }

        if let Expr::Identifier(raw_name) = &mut **callee {
            let name = raw_name.strip_prefix("::").unwrap_or(raw_name).to_string();
            self.check_ambiguous_name(&name)?;
            let var_ty = self.lookup_var(&name).map(|v| v.ty.clone());
            if let Some(ty) = var_ty {
                if let Some(return_type) = function_return_type(&ty) {
                    check_indirect_call_arity(&ty, args.len(), *pos)?;
                    return Ok(return_type);
                }
                if let Type::Struct(sname) = ty.strip_const().strip_reference()
                    && let Some(ret) =
                        self.try_functor_call(callee, sname, args, &arg_types, *pos)?
                {
                    return Ok(ret);
                }
            } else if let Some(cls) = self.current_class.clone()
                && let Some(fld) = self.find_field_in_class(&cls, &name)
            {
                let fld_ty = fld.ty.clone();
                **callee = Expr::MemberAccess {
                    object: Box::new(Expr::This),
                    member: name.clone(),
                    is_arrow: true,
                    pos: *pos,
                };
                if let Type::Struct(sname) = fld_ty.strip_const().strip_reference()
                    && let Some(ret) =
                        self.try_functor_call(callee, sname, args, &arg_types, *pos)?
                {
                    return Ok(ret);
                }
            }
            let dependent = *dependent || self.expression_is_dependent(*pos);
            return self.resolve_named_call(&name, args, &arg_types, callee, *pos, dependent);
        }

        let callee_ty = self.analyze_expr(callee)?;
        check_indirect_call_arity(&callee_ty, args.len(), *pos)?;
        if let Type::Struct(sname) = callee_ty.strip_const().strip_reference()
            && let Some(ret) = self.try_functor_call(callee, sname, args, &arg_types, *pos)?
        {
            return Ok(ret);
        }
        Ok(function_return_type(&callee_ty).unwrap_or(Type::Int))
    }

    fn try_resolve_identifier_call(
        &mut self,
        callee: &mut Expr,
        args: &mut Vec<Expr>,
        arg_types: &[Type],
        pos: ExprPos,
    ) -> Result<Option<(Type, Option<Expr>)>, CompileError> {
        let Expr::Identifier(raw_name) = &mut *callee else {
            return Ok(None);
        };
        let resolved = self.resolve_alias_in_qualified_name(raw_name);
        if resolved != *raw_name {
            *raw_name = resolved;
        }
        let name = raw_name.strip_prefix("::").unwrap_or(raw_name).to_string();
        if name == "printf" || name == "std::printf" || name.ends_with("::printf") {
            return Ok(Some((Type::Int, None)));
        }
        if let Some((ty, replacement)) = self.try_scalar_call(&name, args, pos)? {
            return Ok(Some((ty, Some(replacement))));
        }
        if let Some(sname) = self.class_named(&name, pos.line, pos.col) {
            *raw_name = sname.clone();
            let elided = match self.analyze_constructor_call(&sname, args, arg_types, pos)? {
                ConstructorCall::Elided(inner) => Some(inner),
                ConstructorCall::Selected => None,
            };
            return Ok(Some((Type::Struct(sname), elided)));
        }
        self.redirect_to_method(callee, &name, pos);
        Ok(None)
    }

    fn try_functor_call(
        &mut self,
        callee: &mut Expr,
        sname: &str,
        args: &mut Vec<Expr>,
        arg_types: &[Type],
        pos: ExprPos,
    ) -> Result<Option<Type>, CompileError> {
        let plain = QualifiedName::new(sname).plain();
        let has_member_template =
            self.has_functor_template(sname) || self.has_functor_template(plain);
        if self.find_method_in_class(sname, "operator()").is_some() || has_member_template {
            *callee = Expr::MemberAccess {
                object: Box::new(callee.clone()),
                member: "operator()".to_string(),
                is_arrow: false,
                pos,
            };
            return self.analyze_member_callee(callee, args, arg_types);
        }
        Ok(None)
    }

    fn has_functor_template(&self, sname: &str) -> bool {
        self.member_templates
            .get(sname)
            .is_some_and(|templates| templates.iter().any(|m| m.name == "operator()"))
    }

    /// Rewrites a call to a method of the current class into a member access.
    fn redirect_to_method(&self, callee: &mut Expr, name: &str, pos: ExprPos) {
        let Some(cls) = &self.current_class else {
            return;
        };
        if self.find_method_in_class(cls, name).is_some() {
            *callee = Expr::MemberAccess {
                object: Box::new(Expr::This),
                member: name.to_string(),
                is_arrow: true,
                pos,
            };
        } else if let Some((target_class, method_name)) = name.rsplit_once("::")
            && self.is_subclass(cls, target_class)
            && let Some(sig) = self.find_method_in_class(target_class, method_name)
            && !sig.is_static
        {
            let obj = if cls == target_class {
                Expr::This
            } else {
                Expr::Cast {
                    kind: CastKind::Functional,
                    target_type: Type::Pointer(Box::new(Type::Struct(target_class.to_string()))),
                    expr: Box::new(Expr::This),
                }
            };
            *callee = Expr::MemberAccess {
                object: Box::new(obj),
                member: method_name.to_string(),
                is_arrow: true,
                pos,
            };
        }
    }

    /// Returns the explicit parameters and defaults of a member call.
    ///
    /// A non-static method's first parameter is the implicit `this`; a static
    /// method starts at index zero.
    fn member_call_params(sig: &FunctionSig) -> (&[Type], &[Option<Expr>]) {
        if sig.is_static {
            (&sig.param_types[..], &sig.default_args[..])
        } else {
            (
                sig.param_types.get(1..).unwrap_or(&[]),
                sig.default_args.get(1..).unwrap_or(&[]),
            )
        }
    }

    /// Fills in the default arguments of a member call.
    fn fill_member_defaults(
        &mut self,
        sig: &FunctionSig,
        args: &mut Vec<Expr>,
    ) -> Result<(), CompileError> {
        let (params, defaults) = Self::member_call_params(sig);
        if args.len() > params.len() || defaults.len() < params.len() {
            return Ok(());
        }
        let given = args.len();
        for index in given..params.len() {
            if let Some(Some(default)) = defaults.get(index) {
                args.push(default.clone());
            }
        }
        for arg in args[given..].iter_mut() {
            self.analyze_expr(arg)?;
        }
        Ok(())
    }

    fn analyze_member_callee(
        &mut self,
        callee: &mut Expr,
        args: &mut Vec<Expr>,
        arg_types: &[Type],
    ) -> Result<Option<Type>, CompileError> {
        let Expr::MemberAccess {
            object,
            member,
            is_arrow,
            pos,
        } = &mut *callee
        else {
            return Ok(None);
        };
        let obj_ty = self.analyze_expr(object.as_mut())?;
        if let Some(ret) =
            self.try_pseudo_destructor_call(&obj_ty, *is_arrow, member, args.len(), *pos)?
        {
            return Ok(Some(ret));
        }
        let (struct_name, receiver_const) = self.member_struct_name(&obj_ty, *is_arrow, *pos)?;
        if member.starts_with('~') {
            return self
                .resolve_struct_destructor(&struct_name, member, *pos)
                .map(Some);
        }
        let receiver_is_lvalue = if *is_arrow {
            true
        } else {
            obj_ty.is_reference() || object.is_lvalue()
        };
        let sig = self.resolve_member_sig(
            (&struct_name, member),
            (receiver_const, receiver_is_lvalue),
            args,
            arg_types,
            *pos,
        )?;
        if let Some(sig) = sig {
            self.fill_member_defaults(&sig, args)?;
            let (params, _) = Self::member_call_params(&sig);
            self.convert_call_args(args, params, arg_types, *pos)?;
            self.instantiate_pending_method(&sig.mangled_name)?;
            return Ok(Some(sig.return_type.clone()));
        }
        if self
            .find_method_overloads_in_class(&struct_name, member, false)
            .is_empty()
        {
            let has_member_template = self
                .member_templates
                .get(&struct_name)
                .or_else(|| {
                    self.member_templates
                        .get(QualifiedName::new(&struct_name).plain())
                })
                .is_some_and(|templates| {
                    templates
                        .iter()
                        .any(|t| QualifiedName::new(&t.name).plain() == member)
                });
            if has_member_template {
                return Err(CompileError::new(
                    format!("no matching member function for call to '{member}'"),
                    pos.line,
                    pos.col,
                ));
            }
            return Err(CompileError::new(
                format!("No member named '{member}' in struct '{struct_name}'"),
                pos.line,
                pos.col,
            ));
        }
        self.check_member_arguments(
            (&struct_name, member),
            (receiver_const, receiver_is_lvalue),
            arg_types,
            Some(args),
            *pos,
        )?;
        Ok(None)
    }

    fn convert_call_args(
        &mut self,
        args: &mut [Expr],
        param_types: &[Type],
        arg_types: &[Type],
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        for ((arg, param_ty), arg_ty) in args.iter_mut().zip(param_types).zip(arg_types) {
            if let Expr::InitializerList { elements, pos: p } = arg {
                self.resolve_list_init(param_ty, elements, InitContext::CopyListInitArg, *p)?;
            } else if let Type::Struct(sname) = arg_ty.strip_reference().strip_const() {
                let param_clean = param_ty.strip_reference().strip_const();
                if !matches!(param_clean, Type::Struct(p) if p == sname) {
                    let is_const = arg_ty.strip_reference().is_const();
                    if let Some(sig) = self.find_conversion_to(sname, param_clean, is_const) {
                        Self::wrap_conversion_call(arg, &sig.name, pos);
                    }
                }
            }
        }
        Ok(())
    }

    /// Whether the best non-template candidate is an instantiated function
    /// template specialization shadowing a template of the same base name.
    fn cached_specialization_shadows(
        &self,
        name: &str,
        selection: &Option<CandidateSelection>,
    ) -> bool {
        let base = split_template_call_name(name).map_or(name, |(base, _)| base);
        self.function_templates.contains_key(base)
            && selection
                .as_ref()
                .and_then(|s| s.best.as_ref())
                .is_some_and(|(cand, _)| self.instantiated_functions.contains(&cand.mangled_name))
    }

    fn resolve_named_call(
        &mut self,
        name: &str,
        args: &mut Vec<Expr>,
        arg_types: &[Type],
        callee: &mut Expr,
        pos: ExprPos,
        dependent: bool,
    ) -> Result<Type, CompileError> {
        let candidates = self.find_call_candidates(name, arg_types, dependent);
        let selection = candidates
            .as_ref()
            .map(|c| self.select_candidate(c, args, arg_types));

        // An instantiated specialization is registered under the template's
        // base name for operator lookup, so a plain named call must re-run
        // deduction instead of binding to an earlier specialization that
        // happens to share the substituted parameter types.
        let shadowed = self.cached_specialization_shadows(name, &selection);

        if !shadowed
            && let Some(sel) = &selection
            && !sel.ambiguous
            && let Some((cand, full_args)) = &sel.best
            && cand.param_types == arg_types
        {
            return self.apply_call_candidate(
                cand,
                full_args.clone(),
                args,
                arg_types,
                (callee, pos),
            );
        }

        let tmpl_res = self.try_resolve_function_template_call(name, arg_types, args, pos);
        let tmpl_score = match &tmpl_res {
            Ok(Some(sig)) => self.score_call_candidate(sig, arg_types, Some(args)).ok(),
            _ => None,
        };

        if let Some(sel) = &selection
            && !sel.ambiguous
            && let Some((cand, full_args)) = &sel.best
        {
            if let Some(t_score) = tmpl_score
                && (t_score < sel.best_score || (t_score == sel.best_score && shadowed))
                && let Ok(Some(sig)) = &tmpl_res
            {
                let full = self.full_arguments(sig, args);
                return self.apply_call_candidate(sig, full, args, arg_types, (callee, pos));
            }
            return self.apply_call_candidate(
                cand,
                full_args.clone(),
                args,
                arg_types,
                (callee, pos),
            );
        }

        if let Ok(Some(sig)) = tmpl_res {
            let full_args = self.full_arguments(&sig, args);
            return self.apply_call_candidate(&sig, full_args, args, arg_types, (callee, pos));
        }

        if let Err(e) = tmpl_res
            && selection.as_ref().and_then(|s| s.best.as_ref()).is_none()
        {
            return Err(e);
        }

        Err(self.report_no_matching_call(
            name,
            arg_types,
            args.len(),
            (candidates.as_ref(), selection),
            pos,
        ))
    }

    fn report_no_matching_call(
        &self,
        name: &str,
        arg_types: &[Type],
        args_len: usize,
        sel_cand: (Option<&Vec<FunctionSig>>, Option<CandidateSelection>),
        pos: ExprPos,
    ) -> CompileError {
        let (Some(candidates), Some(selection)) = sel_cand else {
            return CompileError::unresolved_name(
                format!("Undefined function '{name}'"),
                pos.line,
                pos.col,
            );
        };
        if selection.ambiguous {
            return self.ambiguous_call(name, arg_types, pos);
        }
        if let Some(failure) = selection.conversion_failure {
            return self.no_matching_conversion(name, &failure, pos);
        }
        let cand = &candidates[0];
        CompileError::new(
            format!(
                "No matching function call for '{name}': expected {} arguments, found {}",
                cand.param_types.len(),
                args_len
            ),
            pos.line,
            pos.col,
        )
    }

    fn apply_call_candidate(
        &mut self,
        cand: &FunctionSig,
        mut full_args: Vec<Expr>,
        args: &mut Vec<Expr>,
        arg_types: &[Type],
        target: (&mut Expr, ExprPos),
    ) -> Result<Type, CompileError> {
        let (callee, pos) = target;
        self.convert_call_args(&mut full_args, &cand.param_types, arg_types, pos)?;
        let mangled = cand.mangled_name.clone();
        let ret_type = cand.return_type.clone();
        self.instantiate_pending_method(&mangled)?;
        *callee = Expr::Identifier(mangled);
        *args = full_args;
        Ok(ret_type)
    }

    fn try_scalar_call(
        &mut self,
        name: &str,
        args: &mut [Expr],
        pos: ExprPos,
    ) -> Result<Option<(Type, Expr)>, CompileError> {
        let Some(ty) = self
            .type_aliases
            .get(name)
            .cloned()
            .or_else(|| Type::from_scalar_name(name))
        else {
            return Ok(None);
        };
        if !ty.is_scalar() {
            return Ok(None);
        }
        if args.len() > 1 {
            return Err(CompileError::new(
                "scalar functional cast requires 0 or 1 arguments",
                pos.line,
                pos.col,
            ));
        }
        if args.is_empty() {
            return Ok(Some((ty.clone(), zero_literal_for_type(&ty))));
        }
        let mut arg = args[0].clone();
        let arg_ty = self.analyze_expr(&mut arg)?;
        if !self.can_assign_expr(&ty, &arg_ty, Some(&arg)) {
            return Err(CompileError::new(
                format!(
                    "cannot convert '{}' to '{}' in functional cast",
                    arg_ty.to_c_type(),
                    ty.to_c_type()
                ),
                pos.line,
                pos.col,
            ));
        }
        let cast = Expr::Cast {
            kind: CastKind::Functional,
            target_type: ty.clone(),
            expr: Box::new(arg),
        };
        Ok(Some((ty, cast)))
    }
}
