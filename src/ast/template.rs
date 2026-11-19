use super::decl::{Decl, FunctionDecl, StructFieldDecl};
use super::expr::Expr;
use super::types::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateParam {
    pub name: String,
    pub default_type: Option<Type>,
    pub param_type: Option<Type>,
    pub is_pack: bool,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateClass {
    pub name: String,
    /// Parsed template arguments of a specialization pattern. Kept as AST
    /// because `to_c_type()` is lossy for `decltype` operands.
    pub pattern_args: Option<Vec<Type>>,
    pub is_class: bool,
    pub is_final: bool,
    pub base_classes: Vec<String>,
    pub base_access: Vec<crate::ast::BaseAccess>,
    pub fields: Vec<StructFieldDecl>,
    pub methods: Vec<FunctionDecl>,
    /// Friend functions declared in the class template. They are registered
    /// for ADL when a concrete specialization is instantiated.
    pub friends: Vec<FunctionDecl>,
    pub static_fields: Vec<(String, Type, Option<Expr>)>,
    pub private_members: Vec<String>,
    pub nested_structs: Vec<Decl>,
    pub alignment: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplateKind {
    Class(TemplateClass),
    Function(Box<FunctionDecl>),
    Alias(String, Type),
    /// A variable template such as `template <class T> constexpr bool v = ...`.
    Variable(String, Type, Option<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateDecl {
    pub params: Vec<TemplateParam>,
    pub kind: TemplateKind,
    pub line: usize,
    pub col: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_param_equality() {
        let p1 = TemplateParam {
            name: "T".to_string(),
            default_type: None,
            param_type: None,
            is_pack: false,
            line: 1,
            col: 10,
        };
        let p2 = TemplateParam {
            name: "T".to_string(),
            default_type: None,
            param_type: None,
            is_pack: false,
            line: 1,
            col: 10,
        };
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_template_param_pack() {
        let p = TemplateParam {
            name: "Ts".to_string(),
            default_type: None,
            param_type: None,
            is_pack: true,
            line: 1,
            col: 10,
        };
        assert!(p.is_pack);
    }
}
