use super::expr::Expr;
use super::stmt::Stmt;
use super::types::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub default_val: Option<Expr>,
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
    pub is_static: bool,
    pub ctor_initializers: Vec<(String, Vec<Expr>)>,
    pub line: usize,
    pub col: usize,
}

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
        line: usize,
        col: usize,
    },
    EnumDecl {
        name: String,
        variants: Vec<(String, i64)>,
    },
    UsingNamespace(String),
    UsingDecl(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub declarations: Vec<Decl>,
}
