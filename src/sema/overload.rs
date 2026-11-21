//! Overload resolution: the ranking of the candidates of a call and the
//! diagnostics of a call that no candidate matches.

use super::*;

impl Sema {
    pub(super) fn no_matching_constructor(&self, class_name: &str, pos: ExprPos) -> CompileError {
        let message = if self.in_initializer {
            format!("no matching constructor for initialization of '{class_name}'")
        } else {
            format!("no matching constructor for call to '{class_name}'")
        };
        CompileError::new(message, pos.line, pos.col)
    }

    /// The diagnostic for a call that no overload matches better than another.
    pub(super) fn ambiguous_call(
        &self,
        name: &str,
        arg_types: &[Type],
        pos: ExprPos,
    ) -> CompileError {
        let arguments: Vec<String> = arg_types.iter().map(|ty| ty.to_c_type()).collect();
        CompileError::new(
            format!(
                "call of overloaded '{}({})' is ambiguous",
                name,
                arguments.join(", ")
            ),
            pos.line,
            pos.col,
        )
    }

    /// The diagnostic for an argument that does not convert to its parameter.
    pub(super) fn no_matching_conversion(
        &self,
        name: &str,
        failure: &ConversionFailure,
        pos: ExprPos,
    ) -> CompileError {
        CompileError::new(
            format!(
                "no matching conversion for argument {} of '{}': cannot convert '{}' to '{}'",
                failure.index + 1,
                name,
                failure.from.to_c_type(),
                failure.to.to_c_type()
            ),
            pos.line,
            pos.col,
        )
    }

    /// Scores how well a candidate matches the argument types.
    ///
    /// Exact matches add no penalty, reference bindings cost 1, and other
    /// conversions cost 2.  The error names the first argument that does not
    /// convert to its parameter.
    pub(super) fn score_call_candidate(
        &self,
        cand: &FunctionSig,
        arg_types: &[Type],
        args: Option<&[Expr]>,
    ) -> Result<usize, ConversionFailure> {
        self.score_arguments(&cand.param_types, arg_types, args)
    }

    /// Scores the arguments against the parameters of a signature.
    ///
    /// Exact matches add no penalty, reference bindings cost 1, and other
    /// conversions cost 2.  The error names the first argument that does not
    /// convert to its parameter.
    pub(super) fn score_arguments(
        &self,
        params: &[Type],
        arg_types: &[Type],
        args: Option<&[Expr]>,
    ) -> Result<usize, ConversionFailure> {
        let mut score = 0;
        for (i, arg_ty) in arg_types.iter().enumerate() {
            let Some(p) = params.get(i) else {
                return Err(ConversionFailure {
                    index: i,
                    from: arg_ty.clone(),
                    to: Type::Void,
                });
            };
            let p_decay = p.decay_array();
            let a_decay = arg_ty.decay_array();
            let p_base = p_decay.strip_reference().strip_const();
            let a_base = a_decay.strip_reference().strip_const();
            let is_null_ptr_const = args
                .and_then(|a| a.get(i))
                .is_some_and(Sema::is_null_pointer_constant);
            if p.is_reference() {
                match self.score_reference_conversion(p, arg_ty, args.and_then(|a| a.get(i))) {
                    Ok(s) => score += s,
                    Err(_) => {
                        return Err(ConversionFailure {
                            index: i,
                            from: arg_ty.clone(),
                            to: p.clone(),
                        });
                    }
                }
            } else if p_base == a_base {
                if let Type::Struct(sname) = p_base
                    && self.is_move_only_struct(sname)
                    && (args.and_then(|a| a.get(i)).is_some_and(Expr::is_lvalue)
                        || arg_ty.is_lvalue_reference())
                {
                    return Err(ConversionFailure {
                        index: i,
                        from: arg_ty.clone(),
                        to: p.clone(),
                    });
                }
                // Exact matches add no conversion penalty.
            } else if self.check_pointer_conversion(&p_decay, &a_decay) {
                score += 1;
            } else if self.is_promotion(arg_ty, p) {
                score += 2;
            } else if p_decay.is_pointer() && is_null_ptr_const {
                score += 3;
            } else if self.can_assign(p, arg_ty) {
                if self.is_user_conversion(p, arg_ty) {
                    // A converting constructor is worse than a standard
                    // conversion.
                    score += 4;
                } else {
                    score += 3;
                }
            } else if *arg_ty == Type::Void && *p != Type::Void {
                score += 4;
            } else {
                return Err(ConversionFailure {
                    index: i,
                    from: arg_ty.clone(),
                    to: p.clone(),
                });
            }
        }
        Ok(score)
    }

    pub(super) fn score_reference_conversion(
        &self,
        p: &Type,
        arg_ty: &Type,
        arg_expr: Option<&Expr>,
    ) -> Result<usize, ()> {
        let is_lvalue = arg_expr.is_some_and(Expr::is_lvalue) || arg_ty.is_lvalue_reference();
        if let Type::RvalueReference(inner) = p {
            let matches_type = self.reference_types_compatible(
                inner.strip_const(),
                arg_ty.strip_reference().strip_const(),
            );
            if is_lvalue
                || (arg_ty.strip_reference().is_const() && !inner.is_const())
                || !matches_type
            {
                return Err(());
            }
            Ok(1)
        } else if let Type::Reference(inner) = p {
            let matches_type = self.reference_types_compatible(
                inner.strip_const(),
                arg_ty.strip_reference().strip_const(),
            );
            if !inner.is_const() {
                if !is_lvalue || arg_ty.strip_reference().is_const() || !matches_type {
                    return Err(());
                }
                Ok(1)
            } else if matches_type {
                let penalty = if is_lvalue {
                    usize::from(!arg_ty.strip_reference().is_const())
                } else {
                    1
                };
                Ok(1 + penalty)
            } else if self.can_assign(inner.strip_const(), arg_ty) {
                Ok(3)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    /// Whether an expression at `pos` was dependent in the template definition
    /// whose body is currently being analyzed. Positions are preserved by
    /// substitution, so this also holds for rewritten and nested expressions.
    pub(super) fn expression_is_dependent(&self, pos: ExprPos) -> bool {
        self.current_template_identity
            .as_ref()
            .is_some_and(|identity| {
                self.dependent_calls
                    .get(identity)
                    .is_some_and(|positions| positions.contains(&(pos.line, pos.col)))
            })
    }

    pub(super) fn find_call_candidates(
        &mut self,
        name: &str,
        arg_types: &[Type],
        dependent: bool,
    ) -> Option<Vec<FunctionSig>> {
        // Namespace aliases validate late so specialization confirmation sees
        // registered methods, but ordinary lookup must stay at the alias's
        // declaration position. Use its snapshot when one is active.
        let alias_vis = self.active_alias_visibility.as_ref();
        let template_id = self.current_template_identity.as_ref();
        let visible_functions = self.active_visible_functions();
        let visible_usings = alias_vis
            .map(|vis| &vis.usings)
            .or_else(|| template_id.and_then(|id| self.template_definition_usings.get(id)));
        let opened = alias_vis
            .map(|vis| vis.namespaces.as_slice())
            .or_else(|| {
                template_id
                    .and_then(|id| self.template_definition_namespaces.get(id))
                    .map(|v| v.as_slice())
            })
            .unwrap_or(self.open_namespaces.as_slice());
        let mut candidates =
            self.ordinary_lookup_candidates(name, alias_vis, visible_usings, opened);
        // While re-analyzing a substituted template body, ordinary lookup is
        // limited to the overloads that existed at the template's definition.
        // ADL below is intentionally not filtered: it runs at instantiation.
        // A mangled callee is already a resolved binding from an earlier
        // analysis pass, so it must not be filtered by the definition-time set.
        // It still goes through ADL below, which finds nothing for a mangled
        // name but stays correct for builtin names such as `free`.
        let already_bound = self
            .functions
            .get(name)
            .is_some_and(|sigs| sigs.iter().any(|sig| sig.mangled_name == name));
        if !already_bound && let Some(visible) = &visible_functions {
            candidates = candidates.map(|sigs| {
                sigs.into_iter()
                    .filter(|sig| visible.contains(&sig.mangled_name))
                    .collect()
            });
            if candidates.as_ref().is_some_and(|sigs| sigs.is_empty()) {
                candidates = None;
            }
        }
        // ADL is for unqualified calls only, and its candidates are merged
        // with ordinary lookup rather than used as a fallback. Copy the filter
        // only when ADL needs it; the lookup phase above borrows it.
        if !name.contains("::") {
            let visible_functions = visible_functions.cloned();
            candidates = self.merge_adl_candidates(
                candidates,
                name,
                arg_types,
                dependent,
                visible_functions.as_ref(),
            );
        }
        if candidates.is_none()
            && let Some((class_name, method_name)) = name.rsplit_once("::")
        {
            if class_name.contains('<') {
                let _ =
                    self.ensure_class_instantiated(class_name, self.current_line, self.current_col);
            }
            let static_methods = self.find_static_methods_in_class(class_name, method_name);
            if !static_methods.is_empty() {
                candidates = Some(static_methods);
            }
        }
        candidates
    }

    /// Ordinary (non-ADL) lookup at the active declaration position.
    fn ordinary_lookup_candidates(
        &self,
        name: &str,
        alias_vis: Option<&crate::sema::AliasVisibility>,
        visible_usings: Option<&std::collections::HashSet<String>>,
        opened: &[String],
    ) -> Option<Vec<FunctionSig>> {
        let q1 = name.to_string();
        let q2 = QualifiedName::new(name).alternate().into_owned();
        self.functions
            .get(&q1)
            .or_else(|| self.functions.get(&q2))
            .or_else(|| {
                if visible_usings.is_some_and(|visible| !visible.contains(name)) {
                    return None;
                }
                let target = alias_vis
                    .and_then(|vis| vis.using_targets.as_ref())
                    .and_then(|targets| targets.get(name))
                    .or_else(|| self.using_decls.get(name));
                target.and_then(|full| self.functions.get(full))
            })
            .or_else(|| {
                opened
                    .iter()
                    .find_map(|ns| self.functions.get(&format!("{}::{}", ns, name)))
            })
            .cloned()
    }

    /// Merges ADL candidates with ordinary lookup, de-duplicating by mangled name.
    fn merge_adl_candidates(
        &mut self,
        candidates: Option<Vec<FunctionSig>>,
        name: &str,
        arg_types: &[Type],
        dependent: bool,
        visible_functions: Option<&std::collections::HashSet<String>>,
    ) -> Option<Vec<FunctionSig>> {
        let adl_filter = if dependent {
            None
        } else {
            visible_functions.cloned()
        };
        let mut adl = self.adl_candidates(name, arg_types);
        if let Some(visible) = &adl_filter {
            adl.retain(|sig| visible.contains(&sig.mangled_name));
        }
        if adl.is_empty() {
            return candidates;
        }
        let mut merged: Vec<FunctionSig> = Vec::new();
        for sig in candidates.into_iter().flatten().chain(adl) {
            if !merged
                .iter()
                .any(|existing| existing.mangled_name == sig.mangled_name)
            {
                merged.push(sig);
            }
        }
        Some(merged)
    }
}

/// Returns true when two overloads of a method take the same arguments.
///
/// The implicit `this` parameter is not part of the comparison, so a const
/// overload and a non-const overload are different functions.
pub(super) fn same_parameters(first: &FunctionSig, second: &FunctionSig) -> bool {
    let first_params = if first.param_types.is_empty() {
        &[][..]
    } else {
        &first.param_types[1..]
    };
    let second_params = if second.param_types.is_empty() {
        &[][..]
    } else {
        &second.param_types[1..]
    };
    first_params == second_params
        && crate::sema::sig_receiver_is_const(first) == crate::sema::sig_receiver_is_const(second)
}

impl Sema {
    /// Returns true when the conversion from `from` to `to` is a promotion.
    ///
    /// A small integer or an unscoped enum promotes to `int`, and a `float`
    /// promotes to `double`. Scoped enums (`enum class`) never promote to `int`.
    pub(super) fn is_promotion(&self, from: &Type, to: &Type) -> bool {
        let from = from.strip_const();
        let to = to.strip_const();
        if let Type::Enum(ename) = from
            && self.scoped_enums.contains(ename)
        {
            return false;
        }
        match from {
            Type::Bool
            | Type::Char
            | Type::SignedChar
            | Type::UnsignedChar
            | Type::Short
            | Type::UnsignedShort
            | Type::Enum(_) => matches!(to, Type::Int | Type::UnsignedInt),
            Type::Float => matches!(to, Type::Double),
            _ => false,
        }
    }

    /// The outcome of the overload resolution of a call.
    pub(super) fn select_candidate<'a>(
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
            let score = match self.score_call_candidate(cand, arg_types, Some(args)) {
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
                    break;
                }
            } else if score == selection.best_score
                && let Some((best, _)) = selection.best
                && best.param_types != cand.param_types
            {
                selection.ambiguous = true;
            }
        }
        selection
    }

    /// The arguments of a call, with the default arguments filled in.
    pub(super) fn full_arguments(&self, cand: &FunctionSig, args: &[Expr]) -> Vec<Expr> {
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
pub(super) struct CandidateSelection<'a> {
    pub(super) best: Option<(&'a FunctionSig, Vec<Expr>)>,
    pub(super) best_score: usize,
    pub(super) conversion_failure: Option<ConversionFailure>,
    pub(super) ambiguous: bool,
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
