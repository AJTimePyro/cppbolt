use super::expr::BinaryOp;

/// Represents one of the 32 C++17 fold operators ([expr.prim.fold]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldOp {
    /// Binary operators: `+`, `-`, `*`, `/`, `%`, `^`, `&`, `|`, `<<`, `>>`,
    /// `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `,`.
    Binary(BinaryOp),
    /// Simple assignment: `=`.
    Assign,
    /// Compound assignment: `+=`, `-=`, `*=`, `/=`, `%=`, `^=`, `&=`, `|=`, `<<=`, `>>=`.
    Compound(BinaryOp),
    /// Pointer-to-member: `.*`.
    DotStar,
    /// Pointer-to-member: `->*`.
    ArrowStar,
}

impl FoldOp {
    /// Returns the operator string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            FoldOp::Binary(op) => match op {
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
            },
            FoldOp::Assign => "=",
            FoldOp::Compound(op) => match op {
                BinaryOp::Add => "+=",
                BinaryOp::Sub => "-=",
                BinaryOp::Mul => "*=",
                BinaryOp::Div => "/=",
                BinaryOp::Rem => "%=",
                BinaryOp::BitAnd => "&=",
                BinaryOp::BitOr => "|=",
                BinaryOp::BitXor => "^=",
                BinaryOp::Shl => "<<=",
                BinaryOp::Shr => ">>=",
                _ => unreachable!("invalid compound assignment binary op"),
            },
            FoldOp::DotStar => ".*",
            FoldOp::ArrowStar => "->*",
        }
    }
}
