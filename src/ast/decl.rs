use super::expr::Expr;
use super::stmt::Stmt;
use super::template::{TemplateDecl, TemplateParam};
use super::types::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub default_val: Option<Expr>,
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
    pub line: usize,
    pub col: usize,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Function(FunctionDecl),
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
        base_classes: Vec<String>,
        fields: Vec<(String, Type)>,
        methods: Vec<FunctionDecl>,
        static_fields: Vec<(String, Type, Option<Expr>)>,
        private_members: Vec<String>,
        nested_structs: Vec<Decl>,
        line: usize,
        col: usize,
    },
    EnumDecl {
        name: String,
        is_scoped: bool,
        variants: Vec<(String, i64)>,
    },
    UsingNamespace(String),
    UsingDecl(String),
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
    pub type_aliases: HashMap<String, Type>,
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
