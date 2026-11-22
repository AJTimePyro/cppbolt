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
                BinaryOp::DotStar | BinaryOp::ArrowStar => unreachable!("not a fold-binary op"),
                _ => op.symbol(),
            },
            FoldOp::Assign => "=",
            FoldOp::Compound(op) => op
                .compound_symbol()
                .unwrap_or_else(|| unreachable!("invalid compound assignment binary op")),
            FoldOp::DotStar => ".*",
            FoldOp::ArrowStar => "->*",
        }
    }
}
