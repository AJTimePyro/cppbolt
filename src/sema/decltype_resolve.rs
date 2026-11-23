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
            // Bind each parameter before resolving the next one's type so a
            // later `decltype(x)` can name an earlier parameter.
            for p in &mut func.params {
                self.resolve_signature_type(&mut p.ty, line, col)?;
                self.signature_params.push(p.clone());
            }
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
        alias: Option<&str>,
        line: usize,
        col: usize,
    ) -> Result<Type, CompileError> {
        let mut probe = self.clone();
        probe.speculating = true;
        probe.speculative_hard = false;
        if probe.active_alias_visibility.is_none()
            && let Some(name) = alias
            && let Some(context) = self.decltype_alias_context.get(name)
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
                        is_param: true,
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
                    Self::declared_type_of_operand(&mut probe, &operand, ty)
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

    /// C++17 names the declared type of the entity, not the type the analyzer
    /// gave the expression: a reference member stays a reference and a const
    /// receiver adds no const to the member's declared type.
    fn declared_type_of_operand(
        probe: &mut Sema,
        operand: &Expr,
        analyzed: Type,
    ) -> Result<Type, CompileError> {
        match operand {
            Expr::Identifier(name) => Ok(probe
                .lookup_var(name)
                .map_or_else(|| analyzed.clone(), |var| var.ty.clone())),
            Expr::MemberAccess {
                object,
                member,
                is_arrow,
                pos,
            } => {
                let mut obj = (**object).clone();
                let obj_ty = probe.analyze_expr(&mut obj)?;
                let (class, _) = probe.member_struct_name(&obj_ty, *is_arrow, *pos)?;
                Ok(probe
                    .find_field_in_class(&class, member)
                    .map_or_else(|| analyzed.clone(), |fld| fld.ty.clone()))
            }
            _ => Ok(analyzed),
        }
    }
}
