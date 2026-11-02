use super::overload::same_parameters;
use super::*;

impl Sema {
    pub(super) fn resolve_member_sig(
        &mut self,
        (struct_name, member): (&str, &str),
        (is_const, is_lval): (bool, bool),
        args: &mut [Expr],
        arg_types: &[Type],
        pos: ExprPos,
    ) -> Result<Option<FunctionSig>, CompileError> {
        let all_overloads = self.find_method_overloads_in_class(struct_name, member, false);
        if !all_overloads.is_empty() {
            Self::check_ref_qualifier_compatibility(&all_overloads, is_lval, pos)?;
            if let Some(sig) = self.select_member_overload(
                (struct_name, member),
                (is_const, is_lval),
                arg_types,
                Some(args),
                pos,
            )? {
                return Ok(Some(sig));
            }
        }
        self.try_instantiate_member_template(struct_name, Some(member), arg_types, args, pos)
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

    pub(super) fn select_member_overload(
        &self,
        target: (&str, &str),
        receiver: (bool, bool),
        arg_types: &[Type],
        args: Option<&[Expr]>,
        pos: ExprPos,
    ) -> Result<Option<FunctionSig>, CompileError> {
        let (class_name, member) = target;
        let (receiver_const, receiver_is_lvalue) = receiver;
        let overloads = self.find_method_overloads_in_class(class_name, member, receiver_const);
        let mut best_score = usize::MAX;
        let mut best: Option<&FunctionSig> = None;
        let mut ambiguous = false;
        for sig in &overloads {
            if let Some(Ok(score)) =
                self.score_member_sig(sig, receiver_const, receiver_is_lvalue, arg_types, args)
            {
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
        }
        if ambiguous {
            return Err(self.ambiguous_call(member, arg_types, pos));
        }
        Ok(best.cloned())
    }

    /// Checks that the arguments of a method call match an overload.
    ///
    /// The first parameter of a method is the implicit `this`, so the
    /// arguments start at the second one.
    pub(super) fn check_member_arguments(
        &self,
        target: (&str, &str),
        receiver: (bool, bool),
        arg_types: &[Type],
        args: Option<&[Expr]>,
        pos: ExprPos,
    ) -> Result<(), CompileError> {
        let (class_name, member) = target;
        let (receiver_const, receiver_is_lvalue) = receiver;
        let overloads = self.find_method_overloads_in_class(class_name, member, receiver_const);
        if overloads.is_empty() {
            return Ok(());
        }
        let mut conversion_failure: Option<ConversionFailure> = None;
        for sig in &overloads {
            if let Some(Err(failure)) =
                self.score_member_sig(sig, receiver_const, receiver_is_lvalue, arg_types, args)
            {
                conversion_failure.get_or_insert(failure);
            }
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
        args: Option<&[Expr]>,
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
        let params = sig.param_types.get(1..).unwrap_or(&[]);
        let defaults = sig.default_args.get(1..).unwrap_or(&[]);
        let min_args = defaults.iter().take_while(|d| d.is_none()).count();
        if arg_types.len() < min_args || arg_types.len() > params.len() {
            return None;
        }
        Some(self.score_arguments(params, arg_types, args).map(|score| {
            let const_bonus =
                usize::from(!receiver_const && crate::sema::sig_receiver_is_const(sig));
            let ref_bonus = match (receiver_is_lvalue, sig.ref_qualifier) {
                (true, RefQualifier::None) | (false, RefQualifier::None) => 1,
                _ => 0,
            };
            score + const_bonus + ref_bonus
        }))
    }
}
