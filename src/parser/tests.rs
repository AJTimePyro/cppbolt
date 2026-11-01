use super::*;
use crate::lexer::Lexer;

pub(super) fn validates(source: &str) -> bool {
    let Ok(tokens) = Lexer::new(source).tokenize() else {
        return false;
    };
    Parser::new(tokens).validate_supported_features().is_ok()
}

#[test]
pub(super) fn ignores_unsupported_text_in_comments_and_strings() {
    assert!(validates("// operator()\nint main() { return 0; }"));
    assert!(validates(
        "int main() { const char* s = \"operator*()\"; return 0; }"
    ));
}

#[test]
pub(super) fn accepts_ternary_colons_inside_for_loops() {
    assert!(validates(
        "int main() { int x = 0; for (int i = 0; i < 2; i = i ? 1 : 0) { x = i; } return x; }"
    ));
}

#[test]
pub(super) fn multi_dimensional_array_dimensions_wrap_inside_out()
-> Result<(), Box<dyn std::error::Error>> {
    let tokens = Lexer::new("int matrix[2][3];").tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    assert!(matches!(program.declarations[0], Decl::GlobalVar { .. }));
    let Decl::GlobalVar { ty, .. } = &program.declarations[0] else {
        return Ok(());
    };
    let expected = Type::Array(Box::new(Type::Array(Box::new(Type::Int), 3)), 2);
    assert_eq!(ty, &expected);
    Ok(())
}

#[test]
pub(super) fn parses_range_based_for_loop() -> Result<(), Box<dyn std::error::Error>> {
    let tokens = Lexer::new("void f() { for (int x : arr) { x; } }").tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    let Some(Decl::Function(func)) = program.declarations.first() else {
        return Err("expected function declaration".into());
    };
    let Some(body) = &func.body else {
        return Err("expected function with body".into());
    };
    let StmtKind::Compound(stmts) = &body.kind else {
        return Err("expected compound body".into());
    };
    assert!(matches!(
        stmts.first(),
        Some(Stmt {
            kind: StmtKind::RangeFor { .. },
            ..
        })
    ));
    Ok(())
}

#[test]
pub(super) fn test_parse_namespace_alias() -> Result<(), Box<dyn std::error::Error>> {
    let code = "namespace fs = my_lib::filesystem; void test() { namespace local_ns = fs; }";
    let tokens = Lexer::new(code).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    assert!(matches!(
        &program.declarations[0],
        Decl::NamespaceAlias { alias, target, .. } if alias == "fs" && target == "my_lib::filesystem"
    ));
    let Some(Decl::Function(func)) = program.declarations.get(1) else {
        return Err("expected function".into());
    };
    let Some(body) = &func.body else {
        return Err("expected function body".into());
    };
    let StmtKind::Compound(stmts) = &body.kind else {
        return Err("expected compound statement".into());
    };
    assert!(matches!(
        &stmts[0].kind,
        StmtKind::NamespaceAlias { alias, target } if alias == "local_ns" && target == "fs"
    ));
    Ok(())
}

#[test]
pub(super) fn parses_lambda_expression() -> Result<(), Box<dyn std::error::Error>> {
    let tokens =
        Lexer::new("void f() { auto g = [x, &y](int a) mutable -> int { return x + y + a; }; }")
            .tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    let Some(Decl::Function(func)) = program.declarations.first() else {
        return Err("expected function declaration".into());
    };
    let Some(body) = &func.body else {
        return Err("expected body".into());
    };
    let StmtKind::Compound(stmts) = &body.kind else {
        return Err("expected compound".into());
    };
    let StmtKind::VarDecl {
        init:
            Some(Expr::Lambda {
                captures,
                is_mutable,
                return_type,
                ..
            }),
        ..
    } = &stmts[0].kind
    else {
        return Err("expected lambda var decl".into());
    };
    assert_eq!(captures.len(), 2);
    assert_eq!(captures[0].name, "x");
    assert!(!captures[0].is_reference);
    assert_eq!(captures[1].name, "y");
    assert!(captures[1].is_reference);
    assert!(*is_mutable);
    assert_eq!(*return_type, Some(Type::Int));
    Ok(())
}

#[test]
fn parses_global_var_with_parenthesized_initializer() -> Result<(), Box<dyn std::error::Error>> {
    let tokens = Lexer::new("class Item {}; Item global_item(1);").tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    let Some(Decl::GlobalVar { name, ty, init, .. }) = program.declarations.get(1) else {
        return Err("expected global var declaration".into());
    };
    assert_eq!(name, "global_item");
    assert_eq!(*ty, Type::Struct("Item".to_string()));
    let Some(Expr::Call { callee, args, .. }) = init else {
        return Err("expected constructor call init".into());
    };
    assert_eq!(**callee, Expr::Identifier("Item".to_string()));
    assert_eq!(args.len(), 1);
    Ok(())
}

#[test]
fn global_var_syntax_error_reports_expression_error() -> Result<(), Box<dyn std::error::Error>> {
    let tokens = Lexer::new("class Item {}; Item global_item(1 + );").tokenize()?;
    let Err(err) = Parser::new(tokens).parse_program() else {
        return Err("expected parse error".into());
    };
    assert!(!err.to_string().contains("Expected type specifier"));
    assert!(err.to_string().contains("Unexpected token in expression"));
    Ok(())
}

#[test]
fn parses_list_initialization_constructs() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        struct Point { int x; int y; };
        struct Foo {
            int m;
            Foo() : m{10} {}
        };
        Point ret() { return {1, 2}; }
        void call() {
            auto p1 = Point{10, 20};
            auto n = int{42};
            Point p2;
            p2 = {30, 40};
            call_fn({1, 2});
            Point* ptr = new Point{10, 20};
            int* arr = new int[5]{1, 2};
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    assert!(!program.declarations.is_empty());
    Ok(())
}

#[test]
fn parses_integer_literal_types_per_cxx17_table_6() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        auto d1 = 42;
        auto d2 = 2147483648;
        auto h1 = 0x80000000;
        auto u1 = 42u;
        auto ull1 = 42ull;
        auto l1 = 42l;
        auto ul1 = 42ul;
        auto hl1 = 0x80000000L;
        auto dl1 = 2147483648L;
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    let get_init_ty = |idx: usize| -> Option<Type> {
        match &program.declarations[idx] {
            Decl::GlobalVar {
                init: Some(Expr::IntLiteral(_, ty)),
                ..
            } => Some(ty.clone()),
            _ => None,
        }
    };

    assert_eq!(get_init_ty(0), Some(Type::Int));
    assert_eq!(get_init_ty(1), Some(Type::LongLong));
    assert_eq!(get_init_ty(2), Some(Type::UnsignedInt));
    assert_eq!(get_init_ty(3), Some(Type::UnsignedInt));
    assert_eq!(get_init_ty(4), Some(Type::UnsignedLongLong));
    assert_eq!(get_init_ty(5), Some(Type::Long));
    assert_eq!(get_init_ty(6), Some(Type::UnsignedLong));
    assert_eq!(get_init_ty(7), Some(Type::UnsignedLong));
    assert_eq!(get_init_ty(8), Some(Type::LongLong));
    Ok(())
}

#[test]
fn test_parse_reinterpret_cast() -> Result<(), Box<dyn std::error::Error>> {
    let code = "int* p; char* c = reinterpret_cast<char*>(p);";
    let tokens = Lexer::new(code).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    let Some(Decl::GlobalVar {
        init: Some(init_expr),
        ..
    }) = program.declarations.get(1)
    else {
        return Err("expected second global var with init".into());
    };
    assert!(matches!(
        init_expr,
        Expr::Cast {
            kind: CastKind::Reinterpret,
            target_type: Type::Pointer(_),
            ..
        }
    ));
    Ok(())
}
