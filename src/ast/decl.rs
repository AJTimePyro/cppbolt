use super::expr::Expr;
use super::stmt::Stmt;
use super::template::{TemplateDecl, TemplateParam};
use super::types::{FunctionQualifiers, Type};
use std::collections::HashMap;

pub type FieldInit = Option<(Expr, usize, usize)>;
pub type StructFieldDecl = (String, Type, FieldInit);

/// A copy constructor takes an lvalue reference to its own class.
pub fn is_copy_ctor(name: &str, params: &[Param]) -> bool {
    let Some(first) = params.first() else {
        return false;
    };
    first.ty.is_lvalue_reference()
        && matches!(first.ty.clean_type(), Type::Struct(ty)
            if crate::name::QualifiedName::new(ty).class_short_name() == name)
        && params.iter().skip(1).all(|p| p.default_val.is_some())
}

/// A move constructor takes an rvalue reference to its own class.
pub fn is_move_ctor(name: &str, params: &[Param]) -> bool {
    let Some(first) = params.first() else {
        return false;
    };
    first.ty.is_rvalue_reference()
        && matches!(first.ty.clean_type(), Type::Struct(ty)
            if crate::name::QualifiedName::new(ty).class_short_name() == name)
        && params.iter().skip(1).all(|p| p.default_val.is_some())
}

/// Visibility keys registered for a declared constructor so exception
/// validation can tell an unavailable copy/move ctor from an unrelated one.
pub fn ctor_access_keys(
    name: &str,
    params: &[Param],
    is_private: bool,
    is_explicit: bool,
) -> Vec<String> {
    let mut keys = Vec::new();
    if is_private {
        if is_copy_ctor(name, params) {
            keys.push(crate::name::copy_ctor_key(name));
        } else if is_move_ctor(name, params) {
            keys.push(crate::name::move_ctor_key(name));
        } else {
            keys.push(name.to_string());
        }
    }
    if is_explicit {
        if is_copy_ctor(name, params) {
            keys.push(crate::name::explicit_copy_ctor_key(name));
        } else if is_move_ctor(name, params) {
            keys.push(crate::name::explicit_move_ctor_key(name));
        }
    }
    keys
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub default_val: Option<Expr>,
    pub is_pack: bool,
}
/// Access specifier of a base-class declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BaseAccess {
    #[default]
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RefQualifier {
    #[default]
    None,
    LValue,
    RValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub return_type: Type,
    pub params: Vec<Param>,
    pub body: Option<Stmt>,
    pub is_virtual: bool,
    pub is_pure_virtual: bool,
    pub is_const: bool,
    pub is_explicit: bool,
    pub ref_qualifier: RefQualifier,
    pub is_static: bool,
    pub ctor_initializers: Vec<(String, Vec<Expr>)>,
    pub template_params: Vec<TemplateParam>,
    pub qualifiers: FunctionQualifiers,
    pub noexcept_expr: Option<Expr>,
    pub line: usize,
    pub col: usize,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Function(FunctionDecl),
    /// A friend function defined in a class. Registered for ADL through the
    /// class, never through ordinary unqualified lookup.
    FriendFunction {
        class_name: String,
        func: FunctionDecl,
    },
    GlobalVar {
        name: String,
        ty: Type,
        init: Option<Expr>,
        line: usize,
        col: usize,
    },
    StructDecl {
        name: String,
        is_class: bool,
        is_final: bool,
        base_classes: Vec<String>,
        base_access: Vec<BaseAccess>,
        fields: Vec<StructFieldDecl>,
        methods: Vec<FunctionDecl>,
        static_fields: Vec<(String, Type, Option<Expr>)>,
        private_members: Vec<String>,
        nested_structs: Vec<Decl>,
        alignment: Option<Type>,
        line: usize,
        col: usize,
    },
    EnumDecl {
        name: String,
        is_scoped: bool,
        variants: Vec<(String, i64)>,
    },
    StaticAssert {
        condition: Expr,
        message: Option<String>,
        line: usize,
        col: usize,
    },
    UsingNamespace {
        name: String,
        line: usize,
        col: usize,
    },
    UsingDecl {
        name: String,
        line: usize,
        col: usize,
    },
    NamespaceAlias {
        alias: String,
        target: String,
        line: usize,
        col: usize,
    },
    Template(TemplateDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub declarations: Vec<Decl>,
    /// Source names of functions declared `= delete`. Used to reject throwing
    /// or catching class types whose copy or move constructor is deleted.
    pub deleted_functions: Vec<String>,
    pub type_aliases: HashMap<String, Type>,
    /// Names of type aliases declared at namespace scope, as opposed to member
    /// aliases whose names are qualified by their owning class.
    pub namespace_type_aliases: HashMap<String, (usize, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_alias_decl_ast() {
        let decl = Decl::NamespaceAlias {
            alias: "fs".to_string(),
            target: "std::filesystem".to_string(),
            line: 1,
            col: 1,
        };
        assert!(matches!(decl, Decl::NamespaceAlias { .. }));
    }
}
