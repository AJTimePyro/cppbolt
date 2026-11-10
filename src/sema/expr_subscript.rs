use super::*;
use crate::ast::expr::{Expr, UnaryOp};
use crate::ast::types::Type;

impl Sema {
    pub(super) fn analyze_subscript(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::ArraySubscript {
            array,
            index,
            pos,
        } = expr
        else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        let arr_ty = self.analyze_expr(array)?;
        let _idx_ty = self.analyze_expr(index)?;
        let receiver_const = receiver_is_const_type(&arr_ty);
        let clean_ty = arr_ty.strip_const();
        if let Type::Struct(sname) = clean_ty
            && let Some(sig) =
                self.find_method_in_class_for_receiver(sname, "operator[]", receiver_const)
        {
            self.instantiate_pending_method(&sig.mangled_name)?;
            return Ok(sig.return_type.strip_reference().clone());
        }
        if let Some(inner) = clean_ty.deref_type() {
            Ok(inner.strip_reference().clone())
        } else {
            let ty_name = arr_ty.to_c_type();
            Err(CompileError::new(
                format!("Subscripted value is not an array or pointer: {ty_name}"),
                pos.line,
                pos.col,
            ))
        }
    }

    pub(super) fn analyze_ternary(&mut self, expr: &mut Expr) -> Result<Type, CompileError> {
        let Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } = expr
        else {
            return Err(crate::internal_error!(
                self.current_line,
                self.current_col,
                "unexpected expression kind"
            ));
        };

        self.analyze_expr(cond)?;
        let t_ty = self.analyze_expr(then_expr)?;
        let e_ty = self.analyze_expr(else_expr)?;
        Ok(t_ty.conditional_result_type(&e_ty))
    }

    /// Returns true when an expression names a storage location that an
    /// assignment may modify.
    pub(super) fn is_assignable_target(&mut self, expr: &Expr, ty: &Type) -> bool {
        if ty.is_reference() {
            return true;
        }
        match expr {
            Expr::Identifier(_) | Expr::MemberAccess { .. } | Expr::ArraySubscript { .. } => true,
            Expr::Unary {
                op: UnaryOp::Deref,
                expr: inner,
                ..
            } => {
                let mut e = (**inner).clone();
                !matches!(self.analyze_expr(&mut e), Ok(t) if matches!(t.strip_reference().strip_const(), Type::Struct(_)))
            }
            Expr::CompoundAssign { target, .. } => self.is_assignable_target(target, ty),
            _ => false,
        }
    }
}
