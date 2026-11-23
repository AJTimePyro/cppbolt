//! Abstract syntax tree: types, expressions, statements, declarations.
//!
//! See the mdBook guide at `docs/book/internals/parser-and-ast.md`.

pub mod decl;
pub mod expr;
mod expr_drop;
pub mod expr_ops;
pub mod fold_op;
pub mod stmt;
pub mod template;
mod type_display;
pub mod types;
#[cfg(test)]
mod types_tests;

pub use decl::{
    BaseAccess, Decl, FieldInit, FunctionDecl, Param, Program, RefQualifier, StructFieldDecl,
};
pub use expr::{
    BinaryOp, CastKind, DefaultCapture, Expr, ExprPos, LambdaCapture, UnaryOp,
    zero_literal_for_type,
};
pub use fold_op::FoldOp;
pub use stmt::{CatchClause, Stmt, StmtKind};
pub use template::{TemplateClass, TemplateDecl, TemplateKind, TemplateParam};
pub use types::FunctionQualifiers;
