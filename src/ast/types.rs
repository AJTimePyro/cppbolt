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
    Array(Box<Type>, usize),
    Struct(String),
    Enum(String),
    Function {
        return_type: Box<Type>,
        param_types: Vec<Type>,
    },
}

impl Type {
    pub fn is_const(&self) -> bool {
        matches!(self, Type::Const(_))
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
            Type::Array(inner, size) => Type::Array(Box::new(inner.strip_all_const()), *size),
            other => other.clone(),
        }
    }

    pub fn strip_reference(&self) -> &Type {
        match self {
            Type::Reference(inner) => inner.strip_reference(),
            _ => self,
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
        let self_clean = self.strip_const();
        let other_clean = other.strip_const();
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
        matches!(self.strip_const(), Type::Reference(_))
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
            Type::Pointer(inner) | Type::Reference(inner) | Type::Array(inner, _) => Some(inner),
            _ => None,
        }
    }

    /// Returns the source-level size of this type on the compiler's 32-bit
    /// WebAssembly target. Record sizes come from semantic analysis.
    pub fn size_of(&self, struct_sizes: &HashMap<String, usize>) -> usize {
        match self.strip_const() {
            Type::Void | Type::Auto => 0,
            Type::Bool | Type::Char | Type::SignedChar | Type::UnsignedChar => 1,
            Type::Short | Type::UnsignedShort => 2,
            Type::Int | Type::UnsignedInt | Type::Long | Type::UnsignedLong | Type::Enum(_) => 4,
            Type::LongLong | Type::UnsignedLongLong => 8,
            Type::Float => 4,
            Type::Double => 8,
            Type::Pointer(_) | Type::Reference(_) => 4, // 32-bit WASM pointers
            Type::Array(elem, len) => elem.size_of(struct_sizes) * len,
            Type::Struct(name) => *struct_sizes
                .get(name)
                .or_else(|| struct_sizes.get(QualifiedName::new(name).plain()))
                .or_else(|| struct_sizes.get(QualifiedName::new(name).std_name().as_ref()))
                .unwrap_or(&4),
            Type::Function { .. } => 4,
            Type::Const(_) => unreachable!(),
        }
    }

    pub fn align_of(&self, struct_aligns: &HashMap<String, usize>) -> usize {
        match self.strip_const() {
            Type::Void | Type::Auto => 1,
            Type::Bool | Type::Char | Type::SignedChar | Type::UnsignedChar => 1,
            Type::Short | Type::UnsignedShort => 2,
            Type::Int | Type::UnsignedInt | Type::Long | Type::UnsignedLong | Type::Enum(_) => 4,
            Type::LongLong | Type::UnsignedLongLong => 8,
            Type::Float => 4,
            Type::Double => 8,
            Type::Pointer(_) | Type::Reference(_) => 4,
            Type::Array(elem, _) => elem.align_of(struct_aligns),
            Type::Struct(name) => *struct_aligns
                .get(name)
                .or_else(|| struct_aligns.get(QualifiedName::new(name).plain()))
                .or_else(|| struct_aligns.get(QualifiedName::new(name).std_name().as_ref()))
                .unwrap_or(&4),
            Type::Function { .. } => 4,
            Type::Const(_) => unreachable!(),
        }
    }

    pub fn to_wasm_type(&self) -> Option<WasmType> {
        match self.strip_const() {
            Type::Void | Type::Auto => None,
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
            | Type::Reference(_) => Some(WasmType::I32),
            Type::LongLong | Type::UnsignedLongLong => Some(WasmType::I64),
            Type::Float => Some(WasmType::F32),
            Type::Double => Some(WasmType::F64),
            Type::Array(_, _) | Type::Struct(_) | Type::Function { .. } => Some(WasmType::I32),
            Type::Const(_) => unreachable!(),
        }
    }

    pub fn to_c_type(&self) -> String {
        match self {
            Type::Const(inner) => format!("const {}", inner.to_c_type()),
            Type::Void => "void".to_string(),
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
            Type::Array(inner, n) => format!("{}[{}]", inner.to_c_type(), n),
            Type::Struct(name) | Type::Enum(name) => match name.as_str() {
                "istream" | "ostream" | "ios_base" | "string" => format!("std::{}", name),
                _ => name.clone(),
            },
            Type::Function { .. } => "void (*)()".to_string(),
        }
    }
}
