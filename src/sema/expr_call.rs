use super::overload::same_parameters;
use super::*;

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
            let name = raw_name.strip_prefix("::").unwrap_or(raw_name);
            if name == "printf" || name == "std::printf" || name.ends_with("::printf") {
                return Ok(Type::Int);
            }

            let alias = QualifiedName::new(name).alternate();
            let sname_match = if self.struct_layouts.contains_key(name) {
                Some(name.to_string())
            } else if self.struct_layouts.contains_key(alias.as_ref()) {
                Some(alias.into_owned())
            } else {
                None
            };
            if let Some(sname) = sname_match {
                // C++17 guarantees that a prvalue of the class initializes
                // the temporary directly, so `T x(T(1))` and `T x = T(1)`
                // construct the object instead of copying it.
                if let [only] = args.as_slice()
                    && matches!(arg_types[0].strip_const(), Type::Struct(other) if *other == sname)
                    && matches!(only, Expr::Call { .. })
                {
                    let inner = std::mem::replace(&mut args[0], Expr::BoolLiteral(false));
                    *expr = inner;
                    return Ok(Type::Struct(sname));
                }
                // The arguments must match a constructor of the class.
                match self.find_constructor_for(&sname, &arg_types) {
                    Some(sig) => {
                        // A constructor call may leave out arguments that have
                        // default values.
                        self.fill_constructor_defaults(&sig, args)?;
                    }
                    None => {
                        if !self.is_copy_initialization(&sname, &arg_types) {
                            return Err(self.no_matching_constructor(&sname, *pos));
                        }
                    }
                }
                return Ok(Type::Struct(sname));
            }

            if let Some(cls) = &self.current_class
                && self.find_method_in_class(cls, name).is_some()
            {
                **callee = Expr::MemberAccess {
                    object: Box::new(Expr::This),
                    member: name.to_string(),
                    is_arrow: true,
                    pos: *pos,
                };
            }
        }

        if let Some(return_type) = self.analyze_member_callee(callee, args, &arg_types)? {
            return Ok(return_type);
        }

        if let Expr::Identifier(raw_name) = &**callee {
            let name = raw_name.strip_prefix("::").unwrap_or(raw_name).to_string();
            self.check_ambiguous_name(&name)?;
            if let Some(var) = self.lookup_var(&name) {
                let stripped = var.ty.strip_const();
                if let Type::Pointer(inner) = stripped {
                    if let Type::Function { return_type, .. } = &**inner {
                        return Ok((**return_type).clone());
                    }
                } else if let Type::Function { return_type, .. } = stripped {
                    return Ok((**return_type).clone());
                }
            }
            return self.resolve_named_call(&name, args, &arg_types, callee, *pos);
        }

        // No named function matched: treat the callee as a function pointer.
        let callee_ty = self.analyze_expr(callee)?;
        if let Type::Pointer(inner) = callee_ty.strip_const() {
            if let Type::Function { return_type, .. } = &**inner {
                return Ok((**return_type).clone());
            }
        } else if let Type::Function { return_type, .. } = callee_ty.strip_const() {
            return Ok((**return_type).clone());
        }
        Ok(Type::Int)
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

    /// Finds the constructor of a class that accepts the given arguments.
    ///
    /// The constructor overload with the fewest defaulted arguments wins, and
    /// an exact match beats a reference binding, which beats an arithmetic
    /// conversion.
    fn find_constructor_for(&self, class_name: &str, arg_types: &[Type]) -> Option<FunctionSig> {
        let short = class_name.split("::").last().unwrap_or(class_name);
        let alternate = QualifiedName::new(class_name).alternate();
        let mut candidates: Vec<FunctionSig> = Vec::new();
        for name in [
            format!("{class_name}::{short}"),
            format!("{alternate}::{short}"),
        ] {
            if let Some(sigs) = self.functions.get(&name) {
                candidates.extend(sigs.iter().cloned());
            }
        }
        if candidates.is_empty() {
            return None;
        }
        let mut best: Option<FunctionSig> = None;
        let mut best_score = usize::MAX;
        for cand in &candidates {
            let params = if cand.param_types.is_empty() {
                &[][..]
            } else {
                &cand.param_types[1..]
            };
            let defaults = if cand.default_args.is_empty() {
                &[][..]
            } else {
                &cand.default_args[1..]
            };
            if arg_types.len() > params.len() || defaults.len() < params.len() {
                continue;
            }
            // Every omitted argument needs a default value.
            if !defaults[arg_types.len()..].iter().all(|d| d.is_some()) {
                continue;
            }
            let mut score = 0;
            let mut fits = true;
            for (index, arg_ty) in arg_types.iter().enumerate() {
                let param = &params[index];
                if param.strip_all_const() == arg_ty.strip_all_const() {
                    continue;
                }
                if param.is_reference()
                    && param.deref_type().map(|d| d.strip_all_const())
                        == Some(arg_ty.strip_all_const())
                {
                    score += 1;
                    continue;
                }
                if self.can_assign(param, arg_ty) {
                    score += 2;
                    continue;
                }
                fits = false;
                break;
            }
            if !fits {
                continue;
            }
            // Prefer the overload with the fewest defaulted arguments.
            score += (params.len() - arg_types.len()) * 100;
            if score < best_score {
                best_score = score;
                best = Some(cand.clone());
            }
        }
        best
    }

    /// Returns true when the arguments describe a copy initialization.
    ///
    /// A single argument of the class itself is initialized with the copy
    /// constructor, whether or not the class declares one.
    fn is_copy_initialization(&self, class_name: &str, arg_types: &[Type]) -> bool {
        arg_types.len() == 1
            && matches!(arg_types[0].strip_const(), Type::Struct(other) if *other == class_name)
    }

    /// Fills in the default arguments of a constructor call.
    ///
    /// The default expressions are analyzed like ordinary arguments.
    fn fill_constructor_defaults(
        &mut self,
        sig: &FunctionSig,
        args: &mut Vec<Expr>,
    ) -> Result<(), CompileError> {
        let given = args.len();
        let last = sig.param_types.len().saturating_sub(1);
        for index in given..last {
            if let Some(Some(default)) = sig.default_args.get(index + 1) {
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
            let struct_name = if *is_arrow {
                let stripped = obj_ty.strip_const();
                if let Type::Pointer(inner) | Type::Reference(inner) = stripped {
                    match inner.strip_const() {
                        Type::Struct(s) => s.clone(),
                        _ => {
                            return Err(CompileError::new(
                                "Arrow member access on non-struct pointer",
                                pos.line,
                                pos.col,
                            ));
                        }
                    }
                } else {
                    return Err(CompileError::new(
                        "Arrow member access on non-pointer",
                        pos.line,
                        pos.col,
                    ));
                }
            } else {
                let stripped = obj_ty.strip_const();
                let target_ty = if let Type::Reference(inner) = stripped {
                    inner.strip_const()
                } else {
                    stripped
                };
                match target_ty {
                    Type::Struct(s) => s.clone(),
                    _ => {
                        return Err(CompileError::new(
                            "Dot member access on non-struct",
                            pos.line,
                            pos.col,
                        ));
                    }
                }
            };

            if member.starts_with('~') {
                return Ok(Some(Type::Void));
            }
            let receiver_const = crate::sema::receiver_is_const_type(&obj_ty);
            if let Some(sig) =
                self.find_method_in_class_for_receiver(&struct_name, member, receiver_const)
            {
                // The arguments must match one of the overloads of the method.
                self.check_member_arguments(&struct_name, member, receiver_const, arg_types, *pos)?;
                // A member call may leave out arguments that have default
                // values.
                self.fill_member_defaults(&sig, args)?;
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

    /// Checks that the arguments of a method call match an overload.
    ///
    /// The first parameter of a method is the implicit `this`, so the
    /// arguments start at the second one.
    fn check_member_arguments(
        &self,
        class_name: &str,
        member: &str,
        receiver_const: bool,
        arg_types: &[Type],
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        let overloads = self.find_method_overloads_in_class(class_name, member, receiver_const);
        if overloads.is_empty() {
            return Ok(());
        }
        let mut conversion_failure: Option<ConversionFailure> = None;
        let mut expected: Option<usize> = None;
        let mut best_score = usize::MAX;
        let mut best: Option<&FunctionSig> = None;
        let mut ambiguous = false;
        for sig in &overloads {
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
                expected = Some(params.len());
                continue;
            }
            match self.score_arguments(params, arg_types) {
                Ok(score) => {
                    // A non-const receiver prefers the non-const overload.
                    let score = score
                        + usize::from(!receiver_const && crate::sema::sig_receiver_is_const(sig));
                    if score < best_score {
                        best_score = score;
                        best = Some(sig);
                        ambiguous = false;
                    } else if score == best_score
                        && let Some(best) = best
                        && !same_parameters(best, sig)
                    {
                        // Two overloads convert the arguments equally well.
                        ambiguous = true;
                    }
                }
                Err(failure) => {
                    if conversion_failure.is_none() {
                        conversion_failure = Some(failure);
                    }
                }
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
        Err(CompileError::new(
            format!(
                "No matching function call for '{}': expected {} arguments, found {}",
                member,
                expected.unwrap_or(0),
                arg_types.len()
            ),
            pos.line,
            pos.col,
        ))
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
        let candidates = self.find_call_candidates(name, arg_types);
        if let Some(candidates) = candidates {
            let mut best_candidate: Option<(&FunctionSig, Vec<Expr>)> = None;
            let mut best_score = usize::MAX;
            let mut conversion_failure: Option<ConversionFailure> = None;
            let mut ambiguous = false;

            for cand in &candidates {
                let min_args = cand.default_args.iter().take_while(|d| d.is_none()).count();
                let max_args = cand.param_types.len();

                if args.len() >= min_args && args.len() <= max_args {
                    let score = match self.score_call_candidate(cand, arg_types) {
                        Ok(score) => score,
                        Err(failure) => {
                            if conversion_failure.is_none() {
                                conversion_failure = Some(failure);
                            }
                            continue;
                        }
                    };

                    if score < best_score {
                        best_score = score;
                        let mut full_args = args.clone();
                        for i in args.len()..max_args {
                            if let Some(def_expr) = &cand.default_args[i] {
                                full_args.push(def_expr.clone());
                            }
                        }
                        best_candidate = Some((cand, full_args));
                        ambiguous = false;
                        if score == 0 {
                            break; // No candidate can score lower.
                        }
                    } else if score == best_score
                        && let Some((best, _)) = best_candidate
                        && best.param_types != cand.param_types
                    {
                        // Two overloads convert the arguments equally well.
                        ambiguous = true;
                    }
                }
            }

            if ambiguous {
                return Err(self.ambiguous_call(name, arg_types, pos));
            }
            if let Some((cand, full_args)) = best_candidate {
                let mangled = cand.mangled_name.clone();
                let ret_type = cand.return_type.clone();
                *callee = Expr::Identifier(mangled);
                *args = full_args;
                return Ok(ret_type);
            }
            // The arguments fit an overload, but one of them does not
            // convert to its parameter.
            if let Some(failure) = conversion_failure {
                return Err(self.no_matching_conversion(name, &failure, pos));
            }
            let cand = &candidates[0];
            return Err(CompileError::new(
                format!(
                    "No matching function call for '{}': expected {} arguments, found {}",
                    name,
                    cand.param_types.len(),
                    args.len()
                ),
                pos.line,
                pos.col,
            ));
        }
        Err(CompileError::new(
            format!("Undefined function '{}'", name),
            pos.line,
            pos.col,
        ))
    }
}
