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
    /// Several declarators of one declaration, such as `int a, b = 2;`.
    ///
    /// The declarators share one scope with their surroundings, so this is not
    /// a `Compound`: `int a, b; a = 1;` must still see `a` afterwards.
    VarDeclGroup(Vec<Stmt>),
    Expr(Expr),
    If {
        /// The `if` arm followed by every `else if` arm, in source order.
        branches: Vec<(Expr, Box<Stmt>)>,
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
    RangeFor {
        decl: Box<Stmt>,
        range: Expr,
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
