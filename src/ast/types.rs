use crate::name::QualifiedName;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Struct(String),
    Enum(String),
    TypeParam(String),
    TemplateInstance {
        name: String,
        args: Vec<Type>,
    },
    Function {
        return_type: Box<Type>,
        param_types: Vec<Type>,
    },
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
            Type::TemplateInstance { name, args } => Type::TemplateInstance {
                name: name.clone(),
                args: args.iter().map(|a| a.strip_all_const()).collect(),
            },
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
                Type::Const(inner) | Type::Reference(inner) | Type::RvalueReference(inner) => {
                    curr = inner
                }
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
            Type::TypeParam(_) | Type::NonType(_) | Type::DependentArray(_, _) => 4,
            Type::Function { .. } => 4,
            Type::Const(_) => unreachable!(),
        }
    }

    pub fn align_of(&self, struct_aligns: &HashMap<String, usize>) -> usize {
        match self.strip_const() {
            Type::Void | Type::Auto => 1,
            Type::Nullptr => 4,
            Type::Bool | Type::Char | Type::SignedChar | Type::UnsignedChar => 1,
            Type::Short | Type::UnsignedShort => 2,
            Type::Int | Type::UnsignedInt | Type::Long | Type::UnsignedLong | Type::Enum(_) => 4,
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
            Type::TypeParam(_) | Type::NonType(_) | Type::DependentArray(_, _) => 4,
            Type::Function { .. } => 4,
            Type::Const(_) => unreachable!(),
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
            Type::Array(_, _)
            | Type::Struct(_)
            | Type::TemplateInstance { .. }
            | Type::TypeParam(_)
            | Type::NonType(_)
            | Type::DependentArray(_, _)
            | Type::Function { .. } => Some(WasmType::I32),
            Type::Const(_) => unreachable!(),
        }
    }

    pub fn to_c_type(&self) -> String {
        match self {
            Type::Const(inner) => format!("const {}", inner.to_c_type()),
            Type::Void => "void".to_string(),
            Type::Nullptr => "decltype(nullptr)".to_string(),
            Type::Auto => "auto".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::SignedChar => "signed char".to_string(),
            Type::UnsignedChar => "unsigned char".to_string(),
            Type::Short => "short".to_string(),
            Type::UnsignedShort => "unsigned short".to_string(),
            Type::Int => "int".to_string(),
            Type::UnsignedInt => "unsigned int".to_string(),
            Type::Long => "long".to_string(),
            Type::UnsignedLong => "unsigned long".to_string(),
            Type::LongLong => "long long".to_string(),
            Type::UnsignedLongLong => "unsigned long long".to_string(),
            Type::Float => "float".to_string(),
            Type::Double => "double".to_string(),
            Type::Pointer(inner) => format!("{} *", inner.to_c_type()),
            Type::Reference(inner) => format!("{} &", inner.to_c_type()),
            Type::RvalueReference(inner) => format!("{} &&", inner.to_c_type()),
            Type::Array(inner, n) => format!("{}[{}]", inner.to_c_type(), n),
            Type::DependentArray(inner, param) => format!("{}[{}]", inner.to_c_type(), param),
            Type::NonType(n) => n.to_string(),
            Type::TypeParam(name) => name.clone(),
            Type::TemplateInstance { name, args } => {
                let formatted = args
                    .iter()
                    .map(|a| a.to_c_type())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, formatted)
            }
            Type::Struct(name) | Type::Enum(name) => match name.as_str() {
                "istream" | "ostream" | "ios_base" | "string" => format!("std::{}", name),
                _ => name.clone(),
            },
            Type::Function { .. } => "void (*)()".to_string(),
        }
    }
}
