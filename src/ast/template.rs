use super::decl::{Decl, FunctionDecl, StructFieldDecl};
use super::expr::Expr;
use super::types::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateParam {
    pub name: String,
    pub default_type: Option<Type>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateClass {
    pub name: String,
    pub is_class: bool,
    pub base_classes: Vec<String>,
    pub fields: Vec<StructFieldDecl>,
    pub methods: Vec<FunctionDecl>,
    pub static_fields: Vec<(String, Type, Option<Expr>)>,
    pub private_members: Vec<String>,
    pub nested_structs: Vec<Decl>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplateKind {
    Class(TemplateClass),
    Function(Box<FunctionDecl>),
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
            line: 1,
            col: 10,
        };
        let p2 = TemplateParam {
            name: "T".to_string(),
            default_type: None,
            line: 1,
            col: 10,
        };
        assert_eq!(p1, p2);
    }
}
