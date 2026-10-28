use super::constructor::ConstructorCall;
use super::overload::same_parameters;
use super::*;

/// The return type of a function pointer or of a function type.
fn function_return_type(ty: &Type) -> Option<Type> {
    let stripped = ty.strip_const();
    if let Type::Pointer(inner) = stripped {
        if let Type::Function { return_type, .. } = &**inner {
            return Some((**return_type).clone());
        }
        return None;
    }
    if let Type::Function { return_type, .. } = stripped {
        return Some((**return_type).clone());
    }
    None
}

impl Sema {
    pub(super) fn analyze_call(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::Call { callee, args, pos } = expr else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        let mut arg_types = Vec::new();
        for a in args.iter_mut() {
            arg_types.push(self.analyze_expr(a)?);
        }

        if let Expr::Identifier(raw_name) = &**callee {
            let name = raw_name.strip_prefix("::").unwrap_or(raw_name).to_string();
            if name == "printf" || name == "std::printf" || name.ends_with("::printf") {
                return Ok(Type::Int);
            }
            if let Some(sname) = self.class_named(&name) {
                match self.analyze_constructor_call(&sname, args, &arg_types, *pos)? {
                    ConstructorCall::Elided(inner) => {
                        *expr = inner;
                        return Ok(Type::Struct(sname));
                    }
                    ConstructorCall::Selected => return Ok(Type::Struct(sname)),
                }
            }
            self.redirect_to_method(callee, &name, *pos);
        }

        if let Some(return_type) = self.analyze_member_callee(callee, args, &arg_types)? {
            return Ok(return_type);
        }

        if let Expr::Identifier(raw_name) = &**callee {
            let name = raw_name.strip_prefix("::").unwrap_or(raw_name).to_string();
            self.check_ambiguous_name(&name)?;
            let var_ty = self.lookup_var(&name).map(|v| v.ty.clone());
            if let Some(ty) = var_ty {
                if let Some(return_type) = function_return_type(&ty) {
                    return Ok(return_type);
                }
                if let Type::Struct(sname) = ty.strip_const().strip_reference()
                    && let Some(ret) =
                        self.try_functor_call(callee, sname, args, &arg_types, *pos)?
                {
                    return Ok(ret);
                }
            }
            return self.resolve_named_call(&name, args, &arg_types, callee, *pos);
        }

        let callee_ty = self.analyze_expr(callee)?;
        if let Type::Struct(sname) = callee_ty.strip_const().strip_reference()
            && let Some(ret) = self.try_functor_call(callee, sname, args, &arg_types, *pos)?
        {
            return Ok(ret);
        }
        Ok(function_return_type(&callee_ty).unwrap_or(Type::Int))
    }

    fn try_functor_call(
        &mut self,
        callee: &mut Expr,
        sname: &str,
        args: &mut Vec<Expr>,
        arg_types: &[Type],
        pos: ExprPos,
    ) -> Result<Option<Type>, CompileError> {
        if self.find_method_in_class(sname, "operator()").is_some() {
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

    /// Rewrites a call to a method of the current class into a member access.
    fn redirect_to_method(&self, callee: &mut Expr, name: &str, pos: ExprPos) {
        if let Some(cls) = &self.current_class
            && self.find_method_in_class(cls, name).is_some()
        {
            *callee = Expr::MemberAccess {
                object: Box::new(Expr::This),
                member: name.to_string(),
                is_arrow: true,
                pos,
            };
        }
    }

    /// Fills in the default arguments of a member call.
    ///
    /// The first parameter of a method is the implicit `this`, so the
    /// arguments start at the second one.
    fn fill_member_defaults(
        &mut self,
        sig: &FunctionSig,
        args: &mut Vec<Expr>,
    ) -> Result<(), CompileError> {
        if sig.param_types.len() < 2 {
            return Ok(());
        }
        let params = &sig.param_types[1..];
        let defaults = &sig.default_args[1..];
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
        if let Expr::MemberAccess {
            object,
            member,
            is_arrow,
            pos,
        } = &mut *callee
        {
            let obj_ty = self.analyze_expr(object.as_mut())?;
            let (struct_name, receiver_const) =
                self.member_struct_name(&obj_ty, *is_arrow, *pos)?;

            if member.starts_with('~') {
                return Ok(Some(Type::Void));
            }
            let receiver_is_lvalue = if *is_arrow {
                true
            } else {
                obj_ty.is_reference() || object.is_lvalue()
            };
            let all_overloads = self.find_method_overloads_in_class(&struct_name, member, false);
            if all_overloads.is_empty() {
                return Err(CompileError::new(
                    format!("No member named '{}' in struct '{}'", member, struct_name),
                    pos.line,
                    pos.col,
                ));
            }
            Self::check_ref_qualifier_compatibility(&all_overloads, receiver_is_lvalue, *pos)?;
            if let Some(sig) = self.find_method_in_class_for_receiver_category(
                &struct_name,
                member,
                receiver_const,
                receiver_is_lvalue,
            ) {
                // The arguments must match one of the overloads of the method.
                self.check_member_arguments(
                    &struct_name,
                    member,
                    (receiver_const, receiver_is_lvalue),
                    arg_types,
                    *pos,
                )?;
                // A member call may leave out arguments that have default
                // values.
                self.fill_member_defaults(&sig, args)?;
                self.convert_call_args(args, &sig.param_types[1..], arg_types, *pos);
                return Ok(Some(sig.return_type.clone()));
            } else {
                return Err(CompileError::new(
                    format!("No member named '{}' in struct '{}'", member, struct_name),
                    pos.line,
                    pos.col,
                ));
            }
        }
        Ok(None)
    }

    fn check_ref_qualifier_compatibility(
        all_overloads: &[FunctionSig],
        receiver_is_lvalue: bool,
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        let has_matching = all_overloads.iter().any(|sig| match sig.ref_qualifier {
            RefQualifier::None => true,
            RefQualifier::LValue => receiver_is_lvalue || crate::sema::sig_receiver_is_const(sig),
            RefQualifier::RValue => !receiver_is_lvalue,
        });
        if !has_matching {
            let msg = if receiver_is_lvalue {
                "cannot call rvalue ref-qualified method on an lvalue"
            } else {
                "cannot call lvalue ref-qualified method on an rvalue"
            };
            return Err(CompileError::new(msg, pos.line, pos.col));
        }
        Ok(())
    }

    /// Checks that the arguments of a method call match an overload.
    ///
    /// The first parameter of a method is the implicit `this`, so the
    /// arguments start at the second one.
    fn check_member_arguments(
        &self,
        class_name: &str,
        member: &str,
        receiver: (bool, bool),
        arg_types: &[Type],
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        let (receiver_const, receiver_is_lvalue) = receiver;
        let overloads = self.find_method_overloads_in_class(class_name, member, receiver_const);
        if overloads.is_empty() {
            return Ok(());
        }
        let mut conversion_failure: Option<ConversionFailure> = None;
        let mut best_score = usize::MAX;
        let mut best: Option<&FunctionSig> = None;
        let mut ambiguous = false;
        for sig in &overloads {
            match self.score_member_sig(sig, receiver_const, receiver_is_lvalue, arg_types) {
                Some(Ok(score)) => {
                    if score < best_score {
                        best_score = score;
                        best = Some(sig);
                        ambiguous = false;
                    } else if score == best_score
                        && let Some(best) = best
                        && !same_parameters(best, sig)
                    {
                        ambiguous = true;
                    }
                }
                Some(Err(failure)) => {
                    conversion_failure.get_or_insert(failure);
                }
                None => {}
            }
        }
        if ambiguous {
            return Err(self.ambiguous_call(member, arg_types, pos));
        }
        if best.is_some() {
            return Ok(());
        }
        if let Some(failure) = conversion_failure {
            return Err(self.no_matching_conversion(member, &failure, pos));
        }
        let expected = overloads
            .first()
            .map(|s| s.param_types.len().saturating_sub(1))
            .unwrap_or(0);
        Err(CompileError::new(
            format!(
                "No matching function call for '{}': expected {} arguments, found {}",
                member,
                expected,
                arg_types.len()
            ),
            pos.line,
            pos.col,
        ))
    }

    fn score_member_sig(
        &self,
        sig: &FunctionSig,
        receiver_const: bool,
        receiver_is_lvalue: bool,
        arg_types: &[Type],
    ) -> Option<Result<usize, ConversionFailure>> {
        if receiver_is_lvalue && sig.ref_qualifier == RefQualifier::RValue {
            return None;
        }
        if !receiver_is_lvalue
            && sig.ref_qualifier == RefQualifier::LValue
            && !crate::sema::sig_receiver_is_const(sig)
        {
            return None;
        }
        let params = if sig.param_types.is_empty() {
            &[][..]
        } else {
            &sig.param_types[1..]
        };
        let defaults = if sig.default_args.is_empty() {
            &[][..]
        } else {
            &sig.default_args[1..]
        };
        let min_args = defaults.iter().take_while(|d| d.is_none()).count();
        if arg_types.len() < min_args || arg_types.len() > params.len() {
            return None;
        }
        Some(self.score_arguments(params, arg_types).map(|score| {
            let const_bonus =
                usize::from(!receiver_const && crate::sema::sig_receiver_is_const(sig));
            let ref_bonus = match (receiver_is_lvalue, sig.ref_qualifier) {
                (true, RefQualifier::None) | (false, RefQualifier::None) => 1,
                _ => 0,
            };
            score + const_bonus + ref_bonus
        }))
    }

    fn convert_call_args(
        &self,
        args: &mut [Expr],
        param_types: &[Type],
        arg_types: &[Type],
        pos: ExprPos,
    ) {
        for ((arg, param_ty), arg_ty) in args.iter_mut().zip(param_types).zip(arg_types) {
            if let Type::Struct(sname) = arg_ty.strip_reference().strip_const() {
                let param_clean = param_ty.strip_reference().strip_const();
                if !matches!(param_clean, Type::Struct(p) if p == sname) {
                    let is_const = arg_ty.strip_reference().is_const();
                    if let Some(sig) = self.find_conversion_to(sname, param_clean, is_const) {
                        Self::wrap_conversion_call(arg, &sig.name, pos);
                    }
                }
            }
        }
    }

    fn resolve_named_call(
        &mut self,
        name: &str,
        args: &mut Vec<Expr>,
        arg_types: &[Type],
        callee: &mut Expr,
        pos: ExprPos,
    ) -> Result<Type, CompileError> {
        // Overload resolution: exact matches score best, followed by
        // reference binding, arithmetic conversions, and user conversions.
        let Some(candidates) = self.find_call_candidates(name, arg_types) else {
            return Err(CompileError::new(
                format!("Undefined function '{}'", name),
                pos.line,
                pos.col,
            ));
        };
        let selection = self.select_candidate(&candidates, args, arg_types);
        if selection.ambiguous {
            return Err(self.ambiguous_call(name, arg_types, pos));
        }
        if let Some((cand, mut full_args)) = selection.best {
            self.convert_call_args(&mut full_args, &cand.param_types, arg_types, pos);
            let mangled = cand.mangled_name.clone();
            let ret_type = cand.return_type.clone();
            *callee = Expr::Identifier(mangled);
            *args = full_args;
            return Ok(ret_type);
        }
        // The arguments fit an overload, but one of them does not convert to
        // its parameter.
        if let Some(failure) = selection.conversion_failure {
            return Err(self.no_matching_conversion(name, &failure, pos));
        }
        let cand = &candidates[0];
        Err(CompileError::new(
            format!(
                "No matching function call for '{}': expected {} arguments, found {}",
                name,
                cand.param_types.len(),
                args.len()
            ),
            pos.line,
            pos.col,
        ))
    }

    /// The outcome of the overload resolution of a call.
    fn select_candidate<'a>(
        &self,
        candidates: &'a [FunctionSig],
        args: &[Expr],
        arg_types: &[Type],
    ) -> CandidateSelection<'a> {
        let mut selection = CandidateSelection::default();
        for cand in candidates {
            let min_args = cand.default_args.iter().take_while(|d| d.is_none()).count();
            let max_args = cand.param_types.len();
            if args.len() < min_args || args.len() > max_args {
                continue;
            }
            let score = match self.score_call_candidate(cand, arg_types) {
                Ok(score) => score,
                Err(failure) => {
                    if selection.conversion_failure.is_none() {
                        selection.conversion_failure = Some(failure);
                    }
                    continue;
                }
            };
            if score < selection.best_score {
                selection.best_score = score;
                selection.best = Some((cand, self.full_arguments(cand, args)));
                selection.ambiguous = false;
                if score == 0 {
                    break; // No candidate can score lower.
                }
            } else if score == selection.best_score
                && let Some((best, _)) = selection.best
                && best.param_types != cand.param_types
            {
                // Two overloads convert the arguments equally well.
                selection.ambiguous = true;
            }
        }
        selection
    }

    /// The arguments of a call, with the default arguments filled in.
    fn full_arguments(&self, cand: &FunctionSig, args: &[Expr]) -> Vec<Expr> {
        let mut full = args.to_vec();
        for index in args.len()..cand.param_types.len() {
            if let Some(Some(default)) = cand.default_args.get(index) {
                full.push(default.clone());
            }
        }
        full
    }
}

/// The outcome of the overload resolution of a call.
struct CandidateSelection<'a> {
    /// The best candidate and its arguments.
    best: Option<(&'a FunctionSig, Vec<Expr>)>,
    /// The conversion cost of the best candidate.
    best_score: usize,
    /// The first argument that does not convert to its parameter.
    conversion_failure: Option<ConversionFailure>,
    /// Whether two candidates convert the arguments equally well.
    ambiguous: bool,
}

impl Default for CandidateSelection<'_> {
    fn default() -> Self {
        Self {
            best: None,
            best_score: usize::MAX,
            conversion_failure: None,
            ambiguous: false,
        }
    }
}
