//! Semantic analysis for return statements.

use super::*;

impl Sema {
    /// Analyzes a `return` statement against the current function return type.
    pub(super) fn analyze_return(
        &mut self,
        expr_opt: &mut Option<Expr>,
        line: usize,
        col: usize,
    ) -> Result<(), CompileError> {
        let Some(expr) = expr_opt else { return Ok(()) };
        let mut ret_ty = self.analyze_return_expr(expr, line, col)?;
        if let Some(expected) = &self.current_func_return_type {
            if !matches!(
                expr,
                Expr::InitializerList { .. } | Expr::TypedInitList { .. }
            ) {
                if let Type::Struct(sname) = ret_ty.strip_reference().strip_const() {
                    let is_const = ret_ty.strip_reference().is_const();
                    if let Some(sig) = self.find_conversion_to(sname, expected, is_const) {
                        Self::wrap_conversion_call(expr, &sig.name, ExprPos::new(line, col));
                        ret_ty = expected.clone();
                    }
                }
                if !self.can_return_value(expected, &ret_ty, expr) {
                    return Err(CompileError::new(
                        format!(
                            "Return type mismatch: expected {}, found {}",
                            expected.to_c_type(),
                            ret_ty.to_c_type()
                        ),
                        line,
                        col,
                    ));
                }
            }
        } else {
            self.current_func_return_type = Some(ret_ty.strip_reference().strip_const().clone());
        }
        Ok(())
    }

    fn analyze_return_expr(
        &mut self,
        expr: &mut Expr,
        line: usize,
        col: usize,
    ) -> Result<Type, CompileError> {
        match expr {
            Expr::InitializerList { elements, pos } => {
                let Some(expected) = self.current_func_return_type.clone() else {
                    return Err(CompileError::new(
                        "cannot deduce return type from initializer list",
                        line,
                        col,
                    ));
                };
                self.resolve_list_init(&expected, elements, InitContext::CopyListInitReturn, *pos)
            }
            Expr::TypedInitList {
                ty: explicit_ty,
                elements,
                pos,
            } => self.resolve_list_init(explicit_ty, elements, InitContext::DirectList, *pos),
            _ => self.analyze_expr_keeping_reference(expr),
        }
    }

    fn can_return_value(&self, expected: &Type, ret_ty: &Type, expr: &Expr) -> bool {
        let is_same_struct = match (
            expected.strip_reference().strip_const(),
            ret_ty.strip_reference().strip_const(),
        ) {
            (Type::Struct(s1), Type::Struct(s2)) => {
                QualifiedName::new(s1).matches(QualifiedName::new(s2))
            }
            _ => false,
        };
        let is_local_var = if let Expr::Identifier(name) = expr {
            self.scopes.iter().rev().any(|s| s.vars.contains_key(name))
        } else {
            false
        };
        if is_local_var
            && is_same_struct
            && !expected.is_reference()
            && !ret_ty.is_reference()
            && !ret_ty.strip_reference().is_const()
        {
            true
        } else {
            self.can_assign_expr(expected, ret_ty, Some(expr))
        }
    }
}
