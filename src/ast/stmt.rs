use super::expr::Expr;
use super::types::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub line: usize,
    pub col: usize,
}

impl Stmt {
    pub fn new(kind: StmtKind, line: usize, col: usize) -> Self {
        Self { kind, line, col }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Compound(Vec<Stmt>),
    UsingNamespace(String),
    UsingDecl(String),
    VarDecl {
        name: String,
        ty: Type,
        init: Option<Expr>,
        is_static: bool,
    },
    Expr(Expr),
    If {
        cond: Expr,
        then_stmt: Box<Stmt>,
        else_stmt: Option<Box<Stmt>>,
    },
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
    DoWhile {
        body: Box<Stmt>,
        cond: Expr,
    },
    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        step: Option<Expr>,
        body: Box<Stmt>,
    },
    Switch {
        expr: Expr,
        body: Box<Stmt>,
    },
    Case(i64),
    Default,
    Break,
    Continue,
    Return(Option<Expr>),
    Empty,
}
