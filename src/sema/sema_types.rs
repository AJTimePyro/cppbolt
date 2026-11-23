//! Small data types used by semantic analysis.

use super::*;

/// The first argument of a call that does not convert to its parameter.
#[derive(Debug, Clone)]
pub(crate) struct ConversionFailure {
    /// The index of the argument, counted from zero.
    pub index: usize,
    /// The type of the argument.
    pub from: Type,
    /// The type of the parameter.
    pub to: Type,
}

#[derive(Debug, Clone)]
pub struct VarSymbol {
    pub name: String,
    pub ty: Type,
    pub is_global: bool,
    /// Offset from the local frame pointer for locals; unused for globals.
    pub stack_offset: i32,
}

#[derive(Clone)]
pub struct Scope {
    pub vars: HashMap<String, VarSymbol>,
    pub namespace_aliases: HashMap<String, String>,
    pub type_aliases: HashMap<String, Type>,
}

/// A virtual or destructor body whose early analysis hit a name that may be
/// declared later in the translation unit. Retried once registration is complete.
#[derive(Clone)]
pub(crate) struct DeferredMethod {
    pub(crate) class_name: String,
    pub(crate) method: FunctionDecl,
}

