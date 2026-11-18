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
        let saved = std::mem::replace(&mut self.signature_params, func.params.clone());
        let result = (|| {
            self.resolve_signature_type(&mut func.return_type, line, col)?;
            for p in &mut func.params {
                self.resolve_signature_type(&mut p.ty, line, col)?;
            }
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
        probe.enter_scope();
        let mut params = self.signature_params.clone();
        for param in &mut params {
            probe.resolve_and_instantiate_type(&mut param.ty, line, col)?;
        }
        if let Some(scope) = probe.scopes.last_mut() {
            for param in params {
                scope.vars.insert(
                    param.name.clone(),
                    VarSymbol {
                        name: param.name.clone(),
                        ty: param.ty,
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
                if !parenthesized
                    && matches!(operand, Expr::Identifier(_) | Expr::MemberAccess { .. })
                {
                    // Unparenthesized id-expression or member access names the
                    // declared type, without a reference.
                    Ok(ty.strip_reference().clone())
                } else if parenthesized && operand.is_lvalue() && !ty.is_reference() {
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
