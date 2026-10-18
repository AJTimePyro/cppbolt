use super::types::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    LogicalAnd,
    LogicalOr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl BinaryOp {
    pub fn op_name(&self) -> Option<&'static str> {
        match self {
            BinaryOp::Add => Some("operator+"),
            BinaryOp::Sub => Some("operator-"),
            BinaryOp::Mul => Some("operator*"),
            BinaryOp::Div => Some("operator/"),
            BinaryOp::Rem => Some("operator%"),
            BinaryOp::BitAnd => Some("operator&"),
            BinaryOp::BitOr => Some("operator|"),
            BinaryOp::BitXor => Some("operator^"),
            BinaryOp::Shl => Some("operator<<"),
            BinaryOp::Shr => Some("operator>>"),
            BinaryOp::Eq => Some("operator=="),
            BinaryOp::Ne => Some("operator!="),
            BinaryOp::Lt => Some("operator<"),
            BinaryOp::Le => Some("operator<="),
            BinaryOp::Gt => Some("operator>"),
            BinaryOp::Ge => Some("operator>="),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Plus,
    Not,
    BitNot,
    Deref,
    AddrOf,
    PreInc,
    PreDec,
    PostInc,
    PostDec,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLiteral(i64, Type),
    FloatLiteral(f64, Type),
    BoolLiteral(bool),
    CharLiteral(char),
    StringLiteral(String),
    Identifier(String),
    This,
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    CompoundAssign {
        op: BinaryOp,
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    ArraySubscript {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    MemberAccess {
        object: Box<Expr>,
        member: String,
        is_arrow: bool,
    },
    Ternary {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    Cast {
        target_type: Type,
        expr: Box<Expr>,
    },
    SizeofType(Type),
    SizeofExpr(Box<Expr>),
    InitializerList(Vec<Expr>),
    New {
        ty: Type,
        args: Vec<Expr>,
        is_array: bool,
        array_size: Option<Box<Expr>>,
        placement: Option<Box<Expr>>,
    },
    Delete {
        expr: Box<Expr>,
        is_array: bool,
    },
}
