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
