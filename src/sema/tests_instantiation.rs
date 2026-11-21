use super::*;
use crate::lexer::Lexer;
use crate::parser::Parser;

#[test]
fn failed_class_instantiation_restores_state_and_allows_retry()
-> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        struct Bad {};
        struct Good { using type = int; };
        template <typename T> struct Traits { using type = typename T::type; };
        template <typename T> struct Box { using X = typename Traits<T>::type; int v; };
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    sema.analyze_program(&mut program)?;

    assert!(
        sema.instantiate_class_template("Box", &[Type::Struct("Bad".into())], 1, 1)
            .is_err_and(|e| e.message == "no type named 'type' in 'Bad'")
    );
    assert_eq!(sema.instantiation_depth, 0);
    assert!(sema.instantiation_stack.is_empty());
    assert!(!sema.instantiated_classes.contains("Box<Bad>"));
    assert!(!sema.instantiated_classes.contains("Traits<Bad>"));
    assert!(!sema.struct_layouts.contains_key("Box<Bad>"));
    assert!(!sema.type_aliases.contains_key("Box<Bad>::X"));

    let ok = sema.instantiate_class_template("Box", &[Type::Struct("Good".into())], 1, 1)?;
    assert_eq!(ok, "Box<Good>");
    assert!(sema.instantiated_classes.contains("Box<Good>"));
    assert!(sema.struct_layouts.contains_key("Box<Good>"));
    Ok(())
}

#[test]
fn failed_function_instantiation_does_not_cache_specialization()
-> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        template <typename T> int f(T x) { return missing_function(x); }
        int main() { return f(1); }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    assert!(sema.analyze_program(&mut program).is_err());
    assert_eq!(sema.instantiation_depth, 0);
    assert!(sema.instantiation_stack.is_empty());
    assert!(sema.instantiated_functions.is_empty());
    assert!(sema.synthesized_decls.is_empty());
    Ok(())
}

#[test]
fn late_class_instantiation_failure_restores_published_state()
-> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        template <typename T> struct V {
            virtual int f() { return this->nope; }
            int v;
        };
        V<int> x;
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    assert!(sema.analyze_program(&mut program).is_err());
    assert!(!sema.struct_layouts.contains_key("V<int>"));
    assert!(!sema.struct_sizes.contains_key("V<int>"));
    assert!(
        !sema
            .synthesized_decls
            .iter()
            .any(|d| matches!(d, Decl::StructDecl { name, .. } if name == "V<int>"))
    );
    assert!(!sema.instantiated_classes.contains("V<int>"));
    assert_eq!(sema.instantiation_depth, 0);
    assert!(sema.instantiation_stack.is_empty());
    Ok(())
}

#[test]
fn speculative_decltype_check_leaves_state_unchanged() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        struct S { int v; };
        int g;
        int f(S s) { return s.v; }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    sema.analyze_program(&mut program)?;
    let before = sema.clone();

    let bad = Expr::Identifier("missing_decltype_name".to_string());
    assert!(sema.resolve_decltype_type(&bad, false, None, 1, 1).is_err());
    assert_eq!(
        sema.struct_layouts
            .keys()
            .collect::<std::collections::HashSet<_>>(),
        before
            .struct_layouts
            .keys()
            .collect::<std::collections::HashSet<_>>()
    );
    assert_eq!(
        sema.global_vars
            .keys()
            .collect::<std::collections::HashSet<_>>(),
        before
            .global_vars
            .keys()
            .collect::<std::collections::HashSet<_>>()
    );
    assert_eq!(
        sema.functions
            .keys()
            .collect::<std::collections::HashSet<_>>(),
        before
            .functions
            .keys()
            .collect::<std::collections::HashSet<_>>()
    );
    assert_eq!(sema.synthesized_decls.len(), before.synthesized_decls.len());
    assert_eq!(sema.instantiated_classes, before.instantiated_classes);
    assert_eq!(sema.instantiated_functions, before.instantiated_functions);
    assert_eq!(sema.instantiation_depth, before.instantiation_depth);
    assert!(sema.instantiation_stack.is_empty());

    let good = Expr::Identifier("g".to_string());
    let ty = sema.resolve_decltype_type(&good, false, None, 1, 1)?;
    assert_eq!(ty, Type::Int);
    assert_eq!(
        sema.struct_layouts
            .keys()
            .collect::<std::collections::HashSet<_>>(),
        before
            .struct_layouts
            .keys()
            .collect::<std::collections::HashSet<_>>()
    );
    assert_eq!(
        sema.functions
            .keys()
            .collect::<std::collections::HashSet<_>>(),
        before
            .functions
            .keys()
            .collect::<std::collections::HashSet<_>>()
    );
    assert_eq!(sema.instantiated_classes, before.instantiated_classes);
    Ok(())
}

#[test]
fn variable_template_specialization_is_registered_once() -> Result<(), Box<dyn std::error::Error>> {
    let mut sema = Sema::new();
    let param = crate::ast::template::TemplateParam {
        name: "T".to_string(),
        default_type: None,
        param_type: None,
        is_pack: false,
        line: 1,
        col: 1,
    };
    sema.variable_templates.insert(
        "v".to_string(),
        (vec![param], Type::Bool, Some(Expr::BoolLiteral(true))),
    );

    let first = sema.instantiate_variable_template("v<int>", 1, 1)?;
    let second = sema.instantiate_variable_template("v<int>", 1, 1)?;
    assert_eq!(first, second);
    assert_eq!(sema.instantiated_variable_templates.len(), 1);
    assert_eq!(sema.synthesized_decls.len(), 1);
    assert_eq!(sema.static_const_values.get("v<int>"), Some(&1));
    Ok(())
}
