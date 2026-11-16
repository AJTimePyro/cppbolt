//! C++ spelling of AST types, kept out of `types.rs` to respect the file-size cap.

use super::types::Type;

fn function_type(return_type: &Type, params: &[Type], class: Option<&str>) -> String {
    let p = params
        .iter()
        .map(|t| t.to_c_type())
        .collect::<Vec<_>>()
        .join(", ");
    match class {
        Some(class) => format!("{} ({}::*)({})", return_type.to_c_type(), class, p),
        None => format!("{} (*)({})", return_type.to_c_type(), p),
    }
}

impl Type {
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
            Type::DependentNonType(s) => s.clone(),
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
            Type::MemberPointer {
                class_name,
                member_type,
            }
            | Type::BoundMemberFunction {
                class_name,
                member_type,
            } => match member_type.as_ref() {
                Type::Function {
                    return_type,
                    param_types,
                } => function_type(return_type, param_types, Some(class_name)),
                other => format!("{} {}::*", other.to_c_type(), class_name),
            },
            Type::Function {
                return_type,
                param_types,
            } => function_type(return_type, param_types, None),
            Type::PackExpansion(inner) => format!("{}...", inner.to_c_type()),
            Type::Aligned(inner, _) | Type::DependentAlignas(inner, _) => inner.to_c_type(),
        }
    }
}
