//! Resolution of `decltype` types.
//!
//! The operand is analyzed on a disposable clone of `Sema` so a failed
//! expression cannot change a later candidate. Errors from analyzing the
//! expression itself belong to the immediate context; errors raised while
//! instantiating another template stay hard.

use super::*;
use crate::diagnostics::{ErrorKind, TemplateFailure};

impl Sema {
    /// Resolves the return and parameter types of a formed signature, with
    /// its parameters bound so a `decltype` operand can name them.
    pub(super) fn resolve_signature_types(
        &mut self,
        func: &mut FunctionDecl,
        line: usize,
        col: usize,
    ) -> Result<(), TemplateFailure> {
        let saved = std::mem::take(&mut self.signature_params);
        let result = (|| {
            // A parameter's own type cannot name the parameters being bound,
            // so complete those with no parameters in scope first. Otherwise
            // a `Decltype` parameter type would re-enter this resolver with
            // the same parameter list and recurse forever.
            for p in &mut func.params {
                self.resolve_signature_type(&mut p.ty, line, col)?;
            }
            self.signature_params = func.params.clone();
            self.resolve_signature_type(&mut func.return_type, line, col)?;
            Ok(())
        })();
        self.signature_params = saved;
        result
    }

    pub(super) fn resolve_decltype_type(
        &mut self,
        expr: &Expr,
        parenthesized: bool,
        line: usize,
        col: usize,
    ) -> Result<Type, CompileError> {
        let mut probe = self.clone();
        probe.speculating = true;
        probe.speculative_hard = false;
        if probe.active_alias_visibility.is_none()
            && let Some(context) = self.decltype_alias_context.get(&expr.to_source())
        {
            probe.active_alias_visibility = Some(context.clone());
        }
        probe.enter_scope();
        if let Some(scope) = probe.scopes.last_mut() {
            for param in &self.signature_params {
                scope.vars.insert(
                    param.name.clone(),
                    VarSymbol {
                        name: param.name.clone(),
                        ty: param.ty.clone(),
                        is_global: false,
                        stack_offset: 0,
                    },
                );
            }
        }
        let mut operand = expr.clone();
        let result = probe.analyze_expr_keeping_reference(&mut operand);
        match result {
            Ok(ty) => {
                if !parenthesized {
                    // C++17 names the declared type of the entity, not the
                    // type the analyzer gave the expression: a reference
                    // member stays a reference and a const receiver adds no
                    // const to the member's declared type.
                    match &operand {
                        Expr::Identifier(name) => {
                            if let Some(var) = probe.lookup_var(name) {
                                Ok(var.ty.clone())
                            } else {
                                Ok(ty)
                            }
                        }
                        Expr::MemberAccess {
                            object,
                            member,
                            is_arrow,
                            pos,
                        } => {
                            let mut obj = (**object).clone();
                            let obj_ty = probe.analyze_expr(&mut obj)?;
                            let (class, _) = probe.member_struct_name(&obj_ty, *is_arrow, *pos)?;
                            if let Some(fld) = probe.find_field_in_class(&class, member) {
                                Ok(fld.ty.clone())
                            } else {
                                Ok(ty)
                            }
                        }
                        _ => Ok(ty),
                    }
                } else if operand.is_lvalue() && !ty.is_reference() {
                    Ok(Type::Reference(Box::new(ty)))
                } else {
                    Ok(ty)
                }
            }
            Err(error) => {
                if probe.speculative_hard || error.kind == ErrorKind::Internal {
                    Err(error)
                } else {
                    Err(
                        CompileError::new(error.message, error.line.max(line), error.col.max(col))
                            .with_kind(ErrorKind::Substitution),
                    )
                }
            }
        }
    }
}
