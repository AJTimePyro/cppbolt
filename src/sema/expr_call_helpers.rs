//! Shared helpers for resolving named and member calls.

use super::overload::CandidateSelection;
use super::*;

/// The inputs to resolving a call written as a plain identifier.
pub(super) struct CallSite<'a> {
    pub(super) name: &'a str,
    pub(super) args: &'a mut Vec<Expr>,
    pub(super) arg_types: &'a [Type],
    pub(super) callee: &'a mut Expr,
    pub(super) pos: ExprPos,
    pub(super) dependent: bool,
}

impl Sema {
    pub(super) fn report_no_matching_call(
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

    pub(super) fn apply_call_candidate(
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

    pub(super) fn try_scalar_call(
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

    /// Error for a member call whose name has no matching declaration.
    pub(super) fn member_call_error(
        &self,
        struct_name: &str,
        member: &str,
        pos: ExprPos,
    ) -> Option<CompileError> {
        if !self
            .find_method_overloads_in_class(struct_name, member, false)
            .is_empty()
        {
            return None;
        }
        let has_member_template = self
            .member_templates
            .get(struct_name)
            .or_else(|| {
                self.member_templates
                    .get(QualifiedName::new(struct_name).plain())
            })
            .is_some_and(|templates| {
                templates
                    .iter()
                    .any(|t| QualifiedName::new(&t.name).plain() == member)
            });
        let msg = if has_member_template {
            format!("no matching member function for call to '{member}'")
        } else {
            format!("No member named '{member}' in struct '{struct_name}'")
        };
        Some(CompileError::new(msg, pos.line, pos.col))
    }

    /// Applies the better of a plain overload selection and a template
    /// candidate, returning `None` when no selected candidate can apply.
    pub(super) fn apply_best_candidate(
        &mut self,
        site: &mut CallSite<'_>,
        selection: &Option<CandidateSelection<'_>>,
        tmpl_res: &Result<Option<FunctionSig>, CompileError>,
        tmpl_score: Option<usize>,
        shadowed: bool,
    ) -> Option<Result<Type, CompileError>> {
        let sel = selection.as_ref()?;
        if sel.ambiguous {
            return None;
        }
        let (cand, full_args) = sel.best.as_ref()?;
        let pos = site.pos;
        if let Some(t_score) = tmpl_score
            && (t_score < sel.best_score || (t_score == sel.best_score && shadowed))
            && let Ok(Some(sig)) = tmpl_res
        {
            let full = self.full_arguments(sig, site.args.as_slice());
            return Some(self.apply_call_candidate(
                sig,
                full,
                &mut *site.args,
                site.arg_types,
                (&mut *site.callee, pos),
            ));
        }
        Some(self.apply_call_candidate(
            cand,
            full_args.clone(),
            &mut *site.args,
            site.arg_types,
            (&mut *site.callee, pos),
        ))
    }
}
