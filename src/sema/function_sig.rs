//! Function signature type.

use super::*;

/// One overload after parameter adjustment.
///
/// For non-static methods, `param_types[0]` is the implicit `this` pointer.
#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: String,
    pub mangled_name: String,
    pub return_type: Type,
    pub param_types: Vec<Type>,
    pub default_args: Vec<Option<Expr>>,
    pub is_static: bool,
    pub is_explicit: bool,
    pub ref_qualifier: RefQualifier,
    pub qualifiers: FunctionQualifiers,
}
