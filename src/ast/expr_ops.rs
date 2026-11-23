//! Binary, unary, and cast operator definitions for AST expressions.

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
    Comma,
    DotStar,
    ArrowStar,
}

impl BinaryOp {
    pub const fn symbol(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Rem => "%",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::Shl => "<<",
            BinaryOp::Shr => ">>",
            BinaryOp::LogicalAnd => "&&",
            BinaryOp::LogicalOr => "||",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::Comma => ",",
            BinaryOp::DotStar => ".*",
            BinaryOp::ArrowStar => "->*",
        }
    }

    pub const fn compound_symbol(&self) -> Option<&'static str> {
        match self {
            BinaryOp::Add => Some("+="),
            BinaryOp::Sub => Some("-="),
            BinaryOp::Mul => Some("*="),
            BinaryOp::Div => Some("/="),
            BinaryOp::Rem => Some("%="),
            BinaryOp::BitAnd => Some("&="),
            BinaryOp::BitOr => Some("|="),
            BinaryOp::BitXor => Some("^="),
            BinaryOp::Shl => Some("<<="),
            BinaryOp::Shr => Some(">>="),
            _ => None,
        }
    }

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
            BinaryOp::Comma => Some("operator,"),
            BinaryOp::ArrowStar => Some("operator->*"),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastKind {
    Static,
    Reinterpret,
    CStyle,
    Functional,
}
