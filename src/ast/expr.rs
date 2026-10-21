use super::types::Type;

/// The source position of an expression node.
///
/// The position is the line and column of the first token of the expression
/// in the preprocessed text.  The preprocessor maps it back to the file and
/// line of the original source, so a diagnostic inside a built-in header
/// names the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExprPos {
    pub line: usize,
    pub col: usize,
}

impl ExprPos {
    pub const fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

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
        pos: ExprPos,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        pos: ExprPos,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        pos: ExprPos,
    },
    CompoundAssign {
        op: BinaryOp,
        target: Box<Expr>,
        value: Box<Expr>,
        pos: ExprPos,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        pos: ExprPos,
    },
    ArraySubscript {
        array: Box<Expr>,
        index: Box<Expr>,
        pos: ExprPos,
    },
    MemberAccess {
        object: Box<Expr>,
        member: String,
        is_arrow: bool,
        pos: ExprPos,
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

impl Drop for Expr {
    /// Drops deeply nested expression trees iteratively so a long
    /// left-associative chain cannot overflow the Rust stack while the AST is
    /// being released.
    fn drop(&mut self) {
        let mut stack = Vec::new();
        take_expr_children(self, &mut stack);
        while let Some(mut expr) = stack.pop() {
            take_expr_children(&mut expr, &mut stack);
        }
    }
}

/// Moves every child expression out of `expr`, leaving cheap leaf
/// placeholders behind so the recursive `Box` drop cannot descend further.
fn take_expr_children(expr: &mut Expr, out: &mut Vec<Expr>) {
    fn take_box(child: &mut Box<Expr>, out: &mut Vec<Expr>) {
        out.push(std::mem::replace(&mut **child, Expr::BoolLiteral(false)));
    }
    match expr {
        Expr::Binary { left, right, .. } => {
            take_box(left, out);
            take_box(right, out);
        }
        Expr::Unary { expr, .. } | Expr::Cast { expr, .. } | Expr::SizeofExpr(expr) => {
            take_box(expr, out);
        }
        Expr::Assign { target, value, .. } => {
            take_box(target, out);
            take_box(value, out);
        }
        Expr::CompoundAssign { target, value, .. } => {
            take_box(target, out);
            take_box(value, out);
        }
        Expr::Call { callee, args, .. } => {
            take_box(callee, out);
            out.append(&mut std::mem::take(args));
        }
        Expr::ArraySubscript { array, index, .. } => {
            take_box(array, out);
            take_box(index, out);
        }
        Expr::MemberAccess { object, .. } => take_box(object, out),
        Expr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => {
            take_box(cond, out);
            take_box(then_expr, out);
            take_box(else_expr, out);
        }
        Expr::InitializerList(list) => out.append(&mut std::mem::take(list)),
        Expr::New {
            args,
            array_size,
            placement,
            ..
        } => {
            out.append(&mut std::mem::take(args));
            if let Some(size) = array_size {
                take_box(size, out);
            }
            if let Some(place) = placement {
                take_box(place, out);
            }
        }
        Expr::Delete { expr, .. } => take_box(expr, out),
        Expr::IntLiteral(..)
        | Expr::FloatLiteral(..)
        | Expr::BoolLiteral(_)
        | Expr::CharLiteral(_)
        | Expr::StringLiteral(_)
        | Expr::Identifier(_)
        | Expr::This
        | Expr::SizeofType(_) => {}
    }
}
