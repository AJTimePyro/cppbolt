//! Semantic analysis and closure synthesis for C++17 lambda expressions.

use super::*;

impl Sema {
    pub(super) fn analyze_lambda(&mut self, _expr: &mut Expr) -> Result<Type, CompileError> {
        Ok(Type::Int)
    }
}
