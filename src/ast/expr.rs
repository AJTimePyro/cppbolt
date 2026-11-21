use super::decl::Param;
use super::fold_op::FoldOp;
use super::stmt::Stmt;
use super::types::Type;

/// The default capture mode of a lambda expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultCapture {
    ByValue,
    ByReference,
}

/// An explicitly captured variable in a lambda introducer.
#[derive(Debug, Clone, PartialEq)]
pub struct LambdaCapture {
    pub name: String,
    pub is_reference: bool,
    pub is_pack: bool,
}

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
    Comma,
    DotStar,
    ArrowStar,
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
        /// True when substitution changed the callee or an argument, meaning
        /// the call is dependent and ADL must run at instantiation.
        dependent: bool,
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
        kind: CastKind,
        target_type: Type,
        expr: Box<Expr>,
    },
    SizeofType(Type),
    AlignofType(Type),
    SizeofExpr(Box<Expr>),
    InitializerList {
        elements: Vec<Expr>,
        pos: ExprPos,
    },
    TypedInitList {
        ty: Type,
        elements: Vec<Expr>,
        pos: ExprPos,
    },
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
    Lambda {
        default_capture: Option<DefaultCapture>,
        captures: Vec<LambdaCapture>,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Box<Stmt>,
        is_mutable: bool,
        pos: ExprPos,
    },
    SizeofPack {
        name: String,
        pos: ExprPos,
    },
    PackExpansion {
        pattern: Box<Expr>,
        pos: ExprPos,
    },
    FoldExpr {
        op: FoldOp,
        left: Option<Box<Expr>>,
        right: Option<Box<Expr>>,
        pos: ExprPos,
    },
}

/// Marker emitted by `Expr::to_source` when an expression form cannot be
/// rendered as re-parsable source. Partial-specialization confirmation turns
/// it into a hard error so an unrenderable operand never silently degrades to
/// selecting the primary template.
pub const UNRENDERABLE_EXPR: &str = "@cppbolt_unrenderable@";

impl Expr {
    pub fn pos(&self) -> ExprPos {
        match self {
            Expr::Binary { pos, .. }
            | Expr::Unary { pos, .. }
            | Expr::Assign { pos, .. }
            | Expr::CompoundAssign { pos, .. }
            | Expr::Call { pos, .. }
            | Expr::ArraySubscript { pos, .. }
            | Expr::MemberAccess { pos, .. }
            | Expr::InitializerList { pos, .. }
            | Expr::TypedInitList { pos, .. }
            | Expr::Lambda { pos, .. }
            | Expr::SizeofPack { pos, .. }
            | Expr::PackExpansion { pos, .. }
            | Expr::FoldExpr { pos, .. } => *pos,
            Expr::Ternary { cond, .. } => cond.pos(),
            Expr::Cast { expr, .. } | Expr::SizeofExpr(expr) => expr.pos(),
            _ => ExprPos::default(),
        }
    }

    /// Renders the expression as re-parsable C++ source. Used when an
    /// expression is embedded in a type name (for example `decltype(T())`)
    /// that is re-parsed after template substitution. Expression forms that
    /// cannot be rendered emit `UNRENDERABLE_EXPR`, which confirmation turns
    /// into a hard error rather than silently rejecting the candidate.
    pub fn to_source(&self) -> String {
        match self {
            Expr::Identifier(id) => id.clone(),
            Expr::IntLiteral(value, _) => value.to_string(),
            Expr::FloatLiteral(value, _) => format!("{value:?}"),
            Expr::BoolLiteral(value) => value.to_string(),
            Expr::CharLiteral(value) => format!("'{}'", value.escape_default()),
            Expr::StringLiteral(value) => format!("\"{}\"", value.escape_default()),
            Expr::This => "this".to_string(),
            Expr::Call { callee, args, .. } => {
                let rendered = args
                    .iter()
                    .map(|arg| arg.to_source())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", callee.to_source(), rendered)
            }
            Expr::MemberAccess {
                object,
                member,
                is_arrow,
                ..
            } => {
                let sep = if *is_arrow { "->" } else { "." };
                format!("{}{}{}", object.to_source(), sep, member)
            }
            Expr::ArraySubscript { array, index, .. } => {
                format!("({})[{}]", array.to_source(), index.to_source())
            }
            Expr::Ternary {
                cond,
                then_expr,
                else_expr,
            } => format!(
                "({} ? {} : {})",
                cond.to_source(),
                then_expr.to_source(),
                else_expr.to_source()
            ),
            Expr::Cast {
                kind,
                target_type,
                expr,
            } => {
                let target = target_type.to_c_type();
                match kind {
                    CastKind::Static => format!("static_cast<{target}>({})", expr.to_source()),
                    CastKind::Reinterpret => {
                        format!("reinterpret_cast<{target}>({})", expr.to_source())
                    }
                    CastKind::CStyle => format!("({target}){}", expr.to_source()),
                    CastKind::Functional => format!("{target}({})", expr.to_source()),
                }
            }
            Expr::SizeofType(ty) => format!("sizeof({})", ty.to_c_type()),
            Expr::AlignofType(ty) => format!("alignof({})", ty.to_c_type()),
            Expr::SizeofExpr(expr) => format!("sizeof({})", expr.to_source()),
            Expr::Unary { .. } => self.render_unary(),
            Expr::Binary { .. } => self.render_binary(),
            _ => UNRENDERABLE_EXPR.to_string(),
        }
    }

    /// Renders a unary expression, handling postfix operators separately.
    fn render_unary(&self) -> String {
        let Expr::Unary { op, expr, .. } = self else {
            return UNRENDERABLE_EXPR.to_string();
        };
        match op {
            UnaryOp::PostInc => return format!("{}++", expr.to_source()),
            UnaryOp::PostDec => return format!("{}--", expr.to_source()),
            _ => {}
        }
        let sym = match op {
            UnaryOp::Neg => "-",
            UnaryOp::Plus => "+",
            UnaryOp::Not => "!",
            UnaryOp::BitNot => "~",
            UnaryOp::Deref => "*",
            UnaryOp::AddrOf => "&",
            UnaryOp::PreInc => "++",
            UnaryOp::PreDec => "--",
            UnaryOp::PostInc | UnaryOp::PostDec => unreachable!(),
        };
        format!("{}{}", sym, expr.to_source())
    }

    /// Renders a binary expression with its operator symbol.
    fn render_binary(&self) -> String {
        let Expr::Binary {
            op, left, right, ..
        } = self
        else {
            return UNRENDERABLE_EXPR.to_string();
        };
        let sym = match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Rem => "%",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::LogicalAnd => "&&",
            BinaryOp::LogicalOr => "||",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::Shl => "<<",
            BinaryOp::Shr => ">>",
            BinaryOp::Comma => ",",
            BinaryOp::DotStar => ".*",
            BinaryOp::ArrowStar => "->*",
        };
        format!("({} {} {})", left.to_source(), sym, right.to_source())
    }

    pub fn is_lvalue(&self) -> bool {
        match self {
            Expr::Identifier(_) | Expr::ArraySubscript { .. } => true,
            Expr::MemberAccess {
                object, is_arrow, ..
            } => *is_arrow || object.is_lvalue(),
            Expr::Binary { op, left, .. } => match op {
                BinaryOp::ArrowStar => true,
                BinaryOp::DotStar => left.is_lvalue(),
                _ => false,
            },
            Expr::Unary {
                op: UnaryOp::Deref, ..
            } => true,
            Expr::Assign { .. } | Expr::CompoundAssign { .. } => true,
            Expr::Cast { target_type, .. } => target_type.is_lvalue_reference(),
            _ => false,
        }
    }
}

pub fn zero_literal_for_type(ty: &Type) -> Expr {
    match ty.strip_const() {
        Type::Bool => Expr::BoolLiteral(false),
        Type::Float | Type::Double => Expr::FloatLiteral(0.0, ty.clone()),
        Type::Char | Type::SignedChar | Type::UnsignedChar => Expr::CharLiteral('\0'),
        _ => Expr::IntLiteral(0, ty.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_typed_init_list_drops_without_overflow() {
        let mut cur = Expr::IntLiteral(1, Type::Int);
        for _ in 0..10_000 {
            cur = Expr::TypedInitList {
                ty: Type::Int,
                elements: vec![cur],
                pos: ExprPos::new(1, 1),
            };
        }
        drop(cur);
    }

    #[test]
    fn deep_initializer_list_drops_without_overflow() {
        let mut cur = Expr::IntLiteral(1, Type::Int);
        for _ in 0..10_000 {
            cur = Expr::InitializerList {
                elements: vec![cur],
                pos: ExprPos::new(1, 1),
            };
        }
        drop(cur);
    }

    #[test]
    fn deep_pack_expansion_drops_without_overflow() {
        let mut cur = Expr::IntLiteral(1, Type::Int);
        for _ in 0..10_000 {
            cur = Expr::PackExpansion {
                pattern: Box::new(cur),
                pos: ExprPos::new(1, 1),
            };
        }
        drop(cur);
    }

    #[test]
    fn deep_fold_expr_drops_without_overflow() {
        let mut cur = Expr::IntLiteral(1, Type::Int);
        for _ in 0..10_000 {
            cur = Expr::FoldExpr {
                op: FoldOp::Binary(BinaryOp::Add),
                left: Some(Box::new(cur)),
                right: None,
                pos: ExprPos::new(1, 1),
            };
        }
        drop(cur);
    }

    #[test]
    fn test_sizeof_pack_expr() {
        let expr = Expr::SizeofPack {
            name: "Args".to_string(),
            pos: ExprPos::new(2, 5),
        };
        assert_eq!(expr.pos(), ExprPos::new(2, 5));
    }
}
