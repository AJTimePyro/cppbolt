//! Target-type propagation for overload-sensitive `&Class::member` addresses.

use super::*;

/// Returns true when an expression is directly the address of a qualified
/// class member (`&Class::member`). Only such expressions should consume a
/// target type supplied by an enclosing initializer/assignment/cast.
pub(crate) fn is_member_pointer_address(expr: &Expr) -> bool {
    matches!(expr, Expr::Unary { op: UnaryOp::AddrOf, expr, .. }
        if matches!(&**expr, Expr::Identifier(name) if name.contains("::")))
}

impl Sema {
    /// Runs `analysis` with `expected` installed as the current target type,
    /// restoring the previous value afterwards (including on error).
    pub(crate) fn with_expected_type<T>(
        &mut self,
        expected: Option<&Type>,
        analysis: impl FnOnce(&mut Self) -> Result<T, CompileError>,
    ) -> Result<T, CompileError> {
        let previous = self.expected_type.take();
        self.expected_type = expected.cloned();
        let result = analysis(self);
        self.expected_type = previous;
        result
    }
}

pub(crate) fn select_function_sig<'b>(
    sigs: &'b [FunctionSig],
    expected: Option<&Type>,
) -> &'b FunctionSig {
    if let Some(exp) = expected {
        let inner = match exp.strip_reference().strip_const() {
            Type::Pointer(inner) => inner.strip_const(),
            other => other,
        };
        if let Type::Function {
            return_type,
            param_types,
            qualifiers,
        } = inner
        {
            for s in sigs {
                if s.return_type == **return_type
                    && s.param_types == *param_types
                    && (!qualifiers.is_noexcept || s.qualifiers.is_noexcept)
                {
                    return s;
                }
            }
        }
    }
    &sigs[0]
}
