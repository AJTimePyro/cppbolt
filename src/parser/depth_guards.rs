//! Parser nesting depth guards.

use super::*;

impl Parser {
    /// Reserves one level of the counter of one nesting construct.
    pub(super) fn enter_nesting_of(
        &self,
        counter: &crate::depth::DepthCounter,
    ) -> Result<DepthGuard, CompileError> {
        match counter.enter(self.current().line, self.current().col) {
            Ok(guard) => Ok(guard),
            Err(error) => {
                self.nesting_exceeded.set(true);
                Err(error)
            }
        }
    }

    /// Reserves one level of the expression counter.
    pub(super) fn enter_expression(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.expressions())
    }

    /// Reserves one level of the parenthesis counter.
    pub(super) fn enter_parenthesis(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.parentheses())
    }

    /// Reserves one level of the call counter.
    pub(super) fn enter_call(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.calls())
    }

    /// Reserves one level of the statement counter.
    pub(super) fn enter_statement(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.statements())
    }

    /// Reserves one level of the block counter.
    pub(super) fn enter_block(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.blocks())
    }

    /// Reserves one level of the declaration counter.
    pub(super) fn enter_declaration(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.declarations())
    }

    /// Reserves one level of the type counter.
    pub(super) fn enter_type(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.types())
    }
}
