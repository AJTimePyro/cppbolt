//! C++ spelling of AST types, split from `types.rs` to keep that file under the size cap.

use super::types::{FunctionQualifiers, Type, UNBOUNDED_ARRAY};

/// C++ spells array bounds outermost first (`int[2][3]` is an array of two
/// `int[3]`), while `Type::Array` nests the element type. Render the whole
/// declarator so a stringified type reparses to the same type.
fn array_type_spelling(ty: &Type) -> String {
    fn collect(ty: &Type, bounds: &mut Vec<String>) -> String {
        match ty {
            Type::Array(inner, n) => {
                let bound = if *n == UNBOUNDED_ARRAY {
                    "[]".to_string()
                } else {
                    format!("[{n}]")
                };
                let base = collect(inner, bounds);
                bounds.insert(0, bound);
                base
            }
            Type::DependentArray(inner, param) => {
                let base = collect(inner, bounds);
                bounds.insert(0, format!("[{param}]"));
                base
            }
            other => other.to_c_type(),
        }
    }
    let mut bounds = Vec::new();
    let base = collect(ty, &mut bounds);
    format!("{base}{}", bounds.concat())
}

fn function_type(
    return_type: &Type,
    params: &[Type],
    class: Option<&str>,
    quals: FunctionQualifiers,
) -> String {
    let p = params
        .iter()
        .map(|t| t.to_c_type())
        .collect::<Vec<_>>()
        .join(", ");
    let mut suffix = String::new();
    if quals.is_const {
        suffix.push_str(" const");
    }
    if quals.is_volatile {
        suffix.push_str(" volatile");
    }
    match quals.ref_qualifier {
        crate::ast::RefQualifier::LValue => suffix.push_str(" &"),
        crate::ast::RefQualifier::RValue => suffix.push_str(" &&"),
        crate::ast::RefQualifier::None => {}
    }
    if quals.is_noexcept {
        suffix.push_str(" noexcept");
    }
    match class {
        Some(class) => format!(
            "{} ({}::*)({}){}",
            return_type.to_c_type(),
            class,
            p,
            suffix
        ),
        None => format!("{} (*)({}){}", return_type.to_c_type(), p, suffix),
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
            Type::Array(_, _) | Type::DependentArray(_, _) => array_type_spelling(self),
            Type::NonType(n) => n.to_string(),
            Type::DependentNonType(s) => {
                if let Some(addr) = s.strip_prefix("@addr:") {
                    format!("&{addr}")
                } else if s == "@null:" {
                    "nullptr".to_string()
                } else {
                    s.clone()
                }
            }
            Type::TypeParam(name) => name.clone(),
            Type::TemplateInstance { name, args } => {
                let formatted = args
                    .iter()
                    .map(|a| a.to_c_type())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, formatted)
            }
            Type::DependentNested(name) => name.clone(),
            Type::Decltype { expr, .. } => format!("decltype({})", expr.to_source()),
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
                    qualifiers,
                } => function_type(return_type, param_types, Some(class_name), *qualifiers),
                other => format!("{} {}::*", other.to_c_type(), class_name),
            },
            Type::Function {
                return_type,
                param_types,
                qualifiers,
            } => function_type(return_type, param_types, None, *qualifiers),
            Type::PackExpansion(inner) => format!("{}...", inner.to_c_type()),
            Type::Aligned(inner, _) | Type::DependentAlignas(inner, _) => inner.to_c_type(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualified_member_function_pointer_spelling() {
        let ty = Type::MemberPointer {
            class_name: "S".to_string(),
            member_type: Box::new(Type::Function {
                return_type: Box::new(Type::Int),
                param_types: vec![Type::Double],
                qualifiers: FunctionQualifiers {
                    is_const: true,
                    is_volatile: true,
                    is_noexcept: true,
                    ref_qualifier: crate::ast::RefQualifier::LValue,
                    is_override: false,
                    is_final: false,
                },
            }),
        };
        assert_eq!(
            ty.to_c_type(),
            "int (S::*)(double) const volatile & noexcept"
        );
    }
}
