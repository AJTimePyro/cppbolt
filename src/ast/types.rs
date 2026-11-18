use crate::ast::expr::Expr;
use crate::name::QualifiedName;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
}
/// cv/ref/noexcept qualifiers attached to a function type. These are part of
/// the type of a pointer to member function in C++17.
///
/// `is_override` and `is_final` describe a declaration, not the function
/// type, so equality deliberately ignores them.
#[derive(Debug, Clone, Copy, Eq, Default)]
pub struct FunctionQualifiers {
    pub is_const: bool,
    pub is_volatile: bool,
    pub is_noexcept: bool,
    pub ref_qualifier: crate::ast::RefQualifier,
    pub is_override: bool,
    pub is_final: bool,
}

impl PartialEq for FunctionQualifiers {
    fn eq(&self, other: &Self) -> bool {
        self.is_const == other.is_const
            && self.is_volatile == other.is_volatile
            && self.is_noexcept == other.is_noexcept
            && self.ref_qualifier == other.ref_qualifier
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Nullptr,
    Bool,
    Char,
    SignedChar,
    UnsignedChar,
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    Long,
    UnsignedLong,
    LongLong,
    UnsignedLongLong,
    Float,
    Double,
    Auto,
    Const(Box<Type>),
    Pointer(Box<Type>),
    Reference(Box<Type>),
    RvalueReference(Box<Type>),
    Array(Box<Type>, usize),
    DependentArray(Box<Type>, String),
    NonType(i128),
    DependentNonType(String),
    Struct(String),
    Enum(String),
    /// A dependent qualified type such as `T::type` or `Traits<T>::type`
    /// whose base was substituted. It must resolve to a concrete type; an
    /// unresolved name is a substitution failure in the immediate context.
    DependentNested(String),
    /// A `decltype(expr)` type. The operand is kept in the AST and resolved
    /// after substitution, when the concrete types of names are known.
    /// `parenthesized` records the C++17 rule for the operand form.
    Decltype {
        expr: Box<Expr>,
        parenthesized: bool,
    },
    TypeParam(String),
    TemplateInstance {
        name: String,
        args: Vec<Type>,
    },
    Function {
        return_type: Box<Type>,
        param_types: Vec<Type>,
        qualifiers: FunctionQualifiers,
    },
    MemberPointer {
        class_name: String,
        member_type: Box<Type>,
    },
    /// The result of `obj.*pmf` when the member is a function: a bound member
    /// function, which is only valid as the callee of a call expression.
    BoundMemberFunction {
        class_name: String,
        member_type: Box<Type>,
    },
    PackExpansion(Box<Type>),
    Aligned(Box<Type>, usize),
    DependentAlignas(Box<Type>, String),
}

impl Type {
    pub fn is_const(&self) -> bool {
        matches!(self, Type::Const(_))
    }

    pub fn is_scalar(&self) -> bool {
        matches!(
            self.strip_const(),
            Type::Bool
                | Type::Char
                | Type::SignedChar
                | Type::UnsignedChar
                | Type::Short
                | Type::UnsignedShort
                | Type::Int
                | Type::UnsignedInt
                | Type::Long
                | Type::UnsignedLong
                | Type::LongLong
                | Type::UnsignedLongLong
                | Type::Float
                | Type::Double
                | Type::Nullptr
                | Type::Pointer(_)
                | Type::MemberPointer { .. }
                | Type::Enum(_)
        )
    }

    pub fn from_scalar_name(name: &str) -> Option<Type> {
        match name {
            "bool" => Some(Type::Bool),
            "char" => Some(Type::Char),
            "signed char" => Some(Type::SignedChar),
            "unsigned char" => Some(Type::UnsignedChar),
            "short" => Some(Type::Short),
            "unsigned short" => Some(Type::UnsignedShort),
            "int" => Some(Type::Int),
            "unsigned int" | "unsigned" => Some(Type::UnsignedInt),
            "long" => Some(Type::Long),
            "unsigned long" => Some(Type::UnsignedLong),
            "long long" => Some(Type::LongLong),
            "unsigned long long" => Some(Type::UnsignedLongLong),
            "float" => Some(Type::Float),
            "double" => Some(Type::Double),
            "size_t" | "std::size_t" => Some(Type::UnsignedInt),
            "ptrdiff_t" | "std::ptrdiff_t" => Some(Type::Int),
            "nullptr_t" | "std::nullptr_t" => Some(Type::Nullptr),
            _ => None,
        }
    }

    pub fn strip_const(&self) -> &Type {
        match self {
            Type::Const(inner) => inner.strip_const(),
            Type::Aligned(inner, _) | Type::DependentAlignas(inner, _) => inner.strip_const(),
            _ => self,
        }
    }

    pub fn strip_all_const(&self) -> Type {
        match self {
            Type::Const(inner) => inner.strip_all_const(),
            Type::Pointer(inner) => Type::Pointer(Box::new(inner.strip_all_const())),
            Type::Reference(inner) => Type::Reference(Box::new(inner.strip_all_const())),
            Type::RvalueReference(inner) => {
                Type::RvalueReference(Box::new(inner.strip_all_const()))
            }
            Type::Array(inner, size) => Type::Array(Box::new(inner.strip_all_const()), *size),
            Type::DependentArray(inner, param) => {
                Type::DependentArray(Box::new(inner.strip_all_const()), param.clone())
            }
            Type::Aligned(inner, align) => Type::Aligned(Box::new(inner.strip_all_const()), *align),
            Type::DependentAlignas(inner, param) => {
                Type::DependentAlignas(Box::new(inner.strip_all_const()), param.clone())
            }
            Type::TemplateInstance { name, args } => Type::TemplateInstance {
                name: name.clone(),
                args: args.iter().map(|a| a.strip_all_const()).collect(),
            },
            Type::MemberPointer {
                class_name,
                member_type,
            } => Type::MemberPointer {
                class_name: class_name.clone(),
                member_type: Box::new(member_type.strip_all_const()),
            },
            Type::PackExpansion(inner) => Type::PackExpansion(Box::new(inner.strip_all_const())),
            other => other.clone(),
        }
    }

    pub fn strip_reference(&self) -> &Type {
        match self {
            Type::Reference(inner) | Type::RvalueReference(inner) => inner.strip_reference(),
            _ => self,
        }
    }

    pub fn clean_type(&self) -> &Type {
        let mut curr = self;
        loop {
            match curr {
                Type::Const(inner)
                | Type::Reference(inner)
                | Type::RvalueReference(inner)
                | Type::Aligned(inner, _)
                | Type::DependentAlignas(inner, _) => curr = inner,
                _ => return curr,
            }
        }
    }

    /// Applies the C++ integral promotions used before arithmetic.
    ///
    /// `char`, `short`, `bool`, and enum values promote to `int`; this target
    /// has 32-bit `long`, and `unsigned long` promotes to `unsigned int`.
    pub fn integer_promotion(&self) -> Type {
        match self.strip_const() {
            Type::Bool
            | Type::Char
            | Type::SignedChar
            | Type::UnsignedChar
            | Type::Short
            | Type::UnsignedShort
            | Type::Enum(_)
            | Type::Long => Type::Int,
            Type::UnsignedLong => Type::UnsignedInt,
            other => other.clone(),
        }
    }

    /// Returns the common type of a binary arithmetic expression after
    /// applying [`Type::integer_promotion`] to both operands.
    pub fn usual_arithmetic_conversion(&self, other: &Type) -> Type {
        let self_clean = self.strip_reference().strip_const();
        let other_clean = other.strip_reference().strip_const();
        if self_clean == &Type::Double || other_clean == &Type::Double {
            return Type::Double;
        }
        if self_clean == &Type::Float || other_clean == &Type::Float {
            return Type::Float;
        }
        let left = self_clean.integer_promotion();
        let right = other_clean.integer_promotion();
        if left == Type::UnsignedLongLong || right == Type::UnsignedLongLong {
            Type::UnsignedLongLong
        } else if left == Type::LongLong || right == Type::LongLong {
            Type::LongLong
        } else if !left.is_signed() || !right.is_signed() {
            Type::UnsignedInt
        } else {
            Type::Int
        }
    }

    /// Returns the common type of a conditional expression whose branches
    /// have the supplied types. Arithmetic branches use the usual arithmetic
    /// conversions; other branches retain the first branch's type.
    pub fn conditional_result_type(&self, other: &Type) -> Type {
        let left = self.strip_reference();
        let right = other.strip_reference();
        let left_arithmetic = left.is_integer() || left.is_floating();
        let right_arithmetic = right.is_integer() || right.is_floating();
        if left_arithmetic && right_arithmetic {
            left.usual_arithmetic_conversion(right)
        } else {
            left.clone()
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self.strip_const(),
            Type::Bool
                | Type::Char
                | Type::SignedChar
                | Type::UnsignedChar
                | Type::Short
                | Type::UnsignedShort
                | Type::Int
                | Type::UnsignedInt
                | Type::Long
                | Type::UnsignedLong
                | Type::LongLong
                | Type::UnsignedLongLong
                | Type::Enum(_)
        )
    }

    pub fn is_signed(&self) -> bool {
        matches!(
            self.strip_const(),
            Type::Char
                | Type::SignedChar
                | Type::Short
                | Type::Int
                | Type::Long
                | Type::LongLong
                | Type::Enum(_)
        )
    }

    pub fn is_floating(&self) -> bool {
        matches!(self.strip_const(), Type::Float | Type::Double)
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self.strip_const(), Type::Pointer(_))
    }

    pub fn is_reference(&self) -> bool {
        matches!(
            self.strip_const(),
            Type::Reference(_) | Type::RvalueReference(_)
        )
    }

    pub fn is_lvalue_reference(&self) -> bool {
        matches!(self.strip_const(), Type::Reference(_))
    }

    pub fn is_rvalue_reference(&self) -> bool {
        matches!(self.strip_const(), Type::RvalueReference(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self.strip_const(), Type::Array(_, _))
    }

    pub fn decay_array(&self) -> Type {
        match self.strip_const() {
            Type::Array(inner, _) => Type::Pointer(inner.clone()),
            _ => self.clone(),
        }
    }

    pub fn deref_type(&self) -> Option<&Type> {
        match self.strip_const() {
            Type::Pointer(inner)
            | Type::Reference(inner)
            | Type::RvalueReference(inner)
            | Type::Array(inner, _) => Some(inner),
            _ => None,
        }
    }

    /// Returns the source-level size of this type on the compiler's 32-bit
    /// WebAssembly target. Record sizes come from semantic analysis.
    pub fn size_of(&self, struct_sizes: &HashMap<String, usize>) -> usize {
        match self.strip_const() {
            Type::Void | Type::Auto => 0,
            Type::Nullptr => 4,
            Type::Bool | Type::Char | Type::SignedChar | Type::UnsignedChar => 1,
            Type::Short | Type::UnsignedShort => 2,
            Type::Int | Type::UnsignedInt | Type::Long | Type::UnsignedLong | Type::Enum(_) => 4,
            Type::LongLong | Type::UnsignedLongLong => 8,
            Type::Float => 4,
            Type::Double => 8,
            Type::Pointer(_) | Type::Reference(_) | Type::RvalueReference(_) => 4, // 32-bit WASM pointers
            Type::Array(elem, len) => elem.size_of(struct_sizes).saturating_mul(*len),
            Type::Struct(name) => *struct_sizes
                .get(name)
                .or_else(|| struct_sizes.get(QualifiedName::new(name).plain()))
                .or_else(|| struct_sizes.get(QualifiedName::new(name).std_name().as_ref()))
                .unwrap_or(&4),
            Type::TemplateInstance { name, .. } => *struct_sizes
                .get(name)
                .or_else(|| struct_sizes.get(QualifiedName::new(name).plain()))
                .unwrap_or(&4),
            Type::TypeParam(_)
            | Type::DependentNested(_)
            | Type::Decltype { .. }
            | Type::NonType(_)
            | Type::DependentNonType(_)
            | Type::DependentArray(_, _) => 4,
            Type::MemberPointer { member_type, .. } => {
                if matches!(member_type.strip_const(), Type::Function { .. }) {
                    8
                } else {
                    4
                }
            }
            Type::Function { .. } | Type::BoundMemberFunction { .. } => 4,
            Type::PackExpansion(_) => 0,
            Type::Const(_) => unreachable!(),
            Type::Aligned(inner, _) | Type::DependentAlignas(inner, _) => {
                inner.size_of(struct_sizes)
            }
        }
    }

    pub fn align_of(&self, struct_aligns: &HashMap<String, usize>) -> usize {
        match self {
            Type::Aligned(inner, align) => (*align).max(inner.align_of(struct_aligns)),
            Type::DependentAlignas(inner, _) => inner.align_of(struct_aligns),
            Type::Const(inner) => inner.align_of(struct_aligns),
            _ => match self.strip_const() {
                Type::Void | Type::Auto => 1,
                Type::Nullptr => 4,
                Type::Bool | Type::Char | Type::SignedChar | Type::UnsignedChar => 1,
                Type::Short | Type::UnsignedShort => 2,
                Type::Int | Type::UnsignedInt | Type::Long | Type::UnsignedLong | Type::Enum(_) => {
                    4
                }
                Type::LongLong | Type::UnsignedLongLong => 8,
                Type::Float => 4,
                Type::Double => 8,
                Type::Pointer(_) | Type::Reference(_) | Type::RvalueReference(_) => 4,
                Type::Array(elem, _) => elem.align_of(struct_aligns),
                Type::Struct(name) => *struct_aligns
                    .get(name)
                    .or_else(|| struct_aligns.get(QualifiedName::new(name).plain()))
                    .or_else(|| struct_aligns.get(QualifiedName::new(name).std_name().as_ref()))
                    .unwrap_or(&4),
                Type::TemplateInstance { name, .. } => *struct_aligns
                    .get(name)
                    .or_else(|| struct_aligns.get(QualifiedName::new(name).plain()))
                    .unwrap_or(&4),
                Type::TypeParam(_)
                | Type::DependentNested(_)
                | Type::Decltype { .. }
                | Type::NonType(_)
                | Type::DependentNonType(_)
                | Type::DependentArray(_, _) => 4,
                Type::MemberPointer { member_type, .. } => {
                    if matches!(member_type.strip_const(), Type::Function { .. }) {
                        8
                    } else {
                        4
                    }
                }
                Type::Function { .. } | Type::BoundMemberFunction { .. } => 4,
                Type::PackExpansion(_) => 1,
                Type::Const(_) => unreachable!(),
                Type::Aligned(..) | Type::DependentAlignas(..) => unreachable!(),
            },
        }
    }

    pub fn to_wasm_type(&self) -> Option<WasmType> {
        match self.strip_const() {
            Type::Void | Type::Auto => None,
            Type::Nullptr => Some(WasmType::I32),
            Type::Bool
            | Type::Char
            | Type::SignedChar
            | Type::UnsignedChar
            | Type::Short
            | Type::UnsignedShort
            | Type::Int
            | Type::UnsignedInt
            | Type::Long
            | Type::UnsignedLong
            | Type::Enum(_)
            | Type::Pointer(_)
            | Type::Reference(_)
            | Type::RvalueReference(_) => Some(WasmType::I32),
            Type::LongLong | Type::UnsignedLongLong => Some(WasmType::I64),
            Type::Float => Some(WasmType::F32),
            Type::Double => Some(WasmType::F64),
            Type::MemberPointer { member_type, .. } => {
                if matches!(member_type.strip_const(), Type::Function { .. }) {
                    Some(WasmType::I64)
                } else {
                    Some(WasmType::I32)
                }
            }
            Type::Array(_, _)
            | Type::Struct(_)
            | Type::TemplateInstance { .. }
            | Type::TypeParam(_)
            | Type::DependentNested(_)
            | Type::Decltype { .. }
            | Type::NonType(_)
            | Type::DependentNonType(_)
            | Type::DependentArray(_, _)
            | Type::Function { .. }
            | Type::BoundMemberFunction { .. } => Some(WasmType::I32),
            Type::PackExpansion(_) => None,
            Type::Const(_) => unreachable!(),
            Type::Aligned(inner, _) | Type::DependentAlignas(inner, _) => inner.to_wasm_type(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_expansion_type() {
        let ty = Type::PackExpansion(Box::new(Type::TypeParam("T".to_string())));
        assert_eq!(ty.to_c_type(), "T...");
        assert_eq!(ty.strip_all_const(), ty);
    }
}
