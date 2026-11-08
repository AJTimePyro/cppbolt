//! Overload scoring shared by function, method, and constructor lookup.

use super::*;

impl<'a> Codegen<'a> {
    /// Scores how well a parameter list matches the argument types.
    ///
    /// Lower is better: exact match (0), const/reference adjustment (1-2),
    /// arithmetic conversion (10), standard conversion (20), and user-defined
    /// conversion (1000). Returns `None` when the arguments cannot bind.
    pub(crate) fn score_overload_params(
        &self,
        params: &[Type],
        arg_types: &[Type],
    ) -> Option<usize> {
        if params.len() != arg_types.len() {
            return None;
        }
        let mut score = 0;
        for (p, a) in params.iter().zip(arg_types) {
            let p_decay = p.decay_array();
            let a_decay = a.decay_array();
            let p_is_ptr = p_decay.is_pointer();
            if p_decay.strip_const() == a_decay.strip_const() {
                if p_decay != a_decay {
                    score += 1;
                }
            } else if self.sema.check_pointer_conversion(&p_decay, &a_decay) {
                score += 1;
            } else if p_is_ptr && (a.is_integer() || *a == Type::Nullptr) {
                score += 20;
            } else if let Type::Reference(inner) = p {
                let matches_type = self.sema.reference_types_compatible(
                    inner.strip_const(),
                    a.strip_reference().strip_const(),
                );
                if !inner.is_const() {
                    if a.strip_reference().is_const() || !matches_type {
                        return None;
                    }
                    score += 1;
                } else if matches_type {
                    let const_bonus = usize::from(!a.strip_reference().is_const());
                    score += 1 + const_bonus;
                } else if self.sema.can_assign(inner.strip_const(), a) {
                    score += 3;
                } else {
                    return None;
                }
            } else if (p.is_integer() && a.is_integer()) || (p.is_floating() && a.is_floating()) {
                score += 10;
            } else if self.sema.can_assign(p, a) {
                let is_user_conv = matches!(p.strip_const().strip_reference(), Type::Struct(_))
                    && !matches!(a.strip_const().strip_reference(), Type::Struct(_));
                if is_user_conv {
                    score += 1000;
                } else {
                    score += 20;
                }
            } else if *a == Type::Void && *p != Type::Void {
                score += 40;
            } else {
                return None;
            }
        }
        Some(score)
    }
}
