//! Overload resolution: the ranking of the candidates of a call and the
//! diagnostics of a call that no candidate matches.

use super::*;

impl Sema {
    /// The diagnostic for arguments that match no constructor of a class.
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
    ) -> Result<usize, ConversionFailure> {
        self.score_arguments(&cand.param_types, arg_types)
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
            let p_base = p_decay.strip_const();
            let a_base = a_decay.strip_const();
            if p_base == a_base {
                // Exact matches add no conversion penalty.
            } else if p.is_reference()
                && p.deref_type().map(|d| d.strip_const()) == Some(arg_ty.strip_const())
            {
                score += 1;
            } else if self.is_promotion(arg_ty, p) {
                score += 2;
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

    pub(super) fn find_call_candidates(
        &mut self,
        name: &str,
        arg_types: &[Type],
    ) -> Option<Vec<FunctionSig>> {
        let q1 = name.to_string();
        let q2 = QualifiedName::new(name).alternate().into_owned();
        let mut candidates = self
            .functions
            .get(&q1)
            .or_else(|| self.functions.get(&q2))
            .or_else(|| {
                self.using_decls
                    .get(name)
                    .and_then(|full| self.functions.get(full))
            })
            .or_else(|| {
                self.open_namespaces
                    .iter()
                    .find_map(|ns| self.functions.get(&format!("{}::{}", ns, name)))
            })
            .cloned();
        if candidates.is_none() {
            for aty in arg_types {
                let clean = aty.strip_const().strip_reference();
                if let Type::Struct(s) = clean
                    && let Some((ns, _)) = s.rsplit_once("::")
                    && let Some(sigs) = self.functions.get(&format!("{}::{}", ns, name))
                {
                    candidates = Some(sigs.clone());
                    break;
                }
            }
        }
        if candidates.is_none()
            && let Some((class_name, method_name)) = name.rsplit_once("::")
        {
            let static_methods = self.find_static_methods_in_class(class_name, method_name);
            if !static_methods.is_empty() {
                candidates = Some(static_methods);
            }
        }
        candidates
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
}
