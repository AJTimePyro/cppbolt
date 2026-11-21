//! Parser nesting depth guards.

use super::*;

impl Parser {
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

    pub(super) fn enter_expression(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.expressions())
    }

    pub(super) fn enter_parenthesis(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.parentheses())
    }

    pub(super) fn enter_call(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.calls())
    }

    pub(super) fn enter_statement(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.statements())
    }

    pub(super) fn enter_block(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.blocks())
    }

    pub(super) fn enter_declaration(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.declarations())
    }

    pub(super) fn enter_type(&self) -> Result<DepthGuard, CompileError> {
        self.enter_nesting_of(self.nesting.types())
    }
}
