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

#[test]
fn test_parse_variadic_template_header_and_sizeof_pack() -> Result<(), Box<dyn std::error::Error>> {
    let code = "template <typename... Ts> void f(Ts... args) { int n = sizeof...(args); }";
    let tokens = Lexer::new(code).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    let Some(Decl::Template(tmpl)) = program.declarations.first() else {
        return Err("expected template declaration".into());
    };
    assert_eq!(tmpl.params.len(), 1);
    assert_eq!(tmpl.params[0].name, "Ts");
    assert!(tmpl.params[0].is_pack);

    let TemplateKind::Function(ref func) = tmpl.kind else {
        return Err("expected template function".into());
    };
    assert_eq!(func.params.len(), 1);
    assert_eq!(func.params[0].name, "args");
    assert!(func.params[0].is_pack);

    let Some(ref body) = func.body else {
        return Err("expected function body".into());
    };
    let StmtKind::Compound(ref stmts) = body.kind else {
        return Err("expected compound body".into());
    };
    let StmtKind::VarDecl { ref init, .. } = stmts[0].kind else {
        return Err("expected vardecl stmt".into());
    };
    let Some(Expr::SizeofPack { name, .. }) = init else {
        return Err("expected SizeofPack expr".into());
    };
    assert_eq!(name, "args");
    Ok(())
}

#[test]
fn test_parse_variadic_class_template_and_non_type_pack() -> Result<(), Box<dyn std::error::Error>> {
    let code = "template <typename T, typename... Rest> class Tuple {};";
    let tokens = Lexer::new(code).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    let Some(Decl::Template(tmpl)) = program.declarations.first() else {
        return Err("expected template decl".into());
    };
    assert_eq!(tmpl.params.len(), 2);
    assert!(!tmpl.params[0].is_pack);
    assert_eq!(tmpl.params[0].name, "T");
    assert!(tmpl.params[1].is_pack);
    assert_eq!(tmpl.params[1].name, "Rest");

    let nttp_code = "template <int... Ns> void f(int...);";
    let nttp_tokens = Lexer::new(nttp_code).tokenize()?;
    let nttp_prog = Parser::new(nttp_tokens).parse_program()?;
    let Some(Decl::Template(tmpl2)) = nttp_prog.declarations.first() else {
        return Err("expected template decl 2".into());
    };
    assert!(tmpl2.params[0].is_pack);
    assert_eq!(tmpl2.params[0].name, "Ns");
    let TemplateKind::Function(ref func) = tmpl2.kind else {
        return Err("expected func".into());
    };
    assert!(func.params[0].is_pack);
    assert_eq!(func.params[0].name, "");
    Ok(())
}

#[test]
fn test_parse_variadic_pack_syntax_errors() -> Result<(), Box<dyn std::error::Error>> {
    let non_trailing_class = "template <typename... Ts, typename U> class Bad {};";
    let tokens = Lexer::new(non_trailing_class).tokenize()?;
    assert!(Parser::new(tokens).parse_program().is_err());

    let non_trailing_alias = "template <typename... Ts, typename U> using Bad = int;";
    let tokens = Lexer::new(non_trailing_alias).tokenize()?;
    assert!(Parser::new(tokens).parse_program().is_err());

    let tmpl_default_arg = "template <typename... Ts = int> void f();";
    let tokens = Lexer::new(tmpl_default_arg).tokenize()?;
    assert!(Parser::new(tokens).parse_program().is_err());

    let func_default_arg = "template <typename... Ts> void f(Ts... args = 0);";
    let tokens = Lexer::new(func_default_arg).tokenize()?;
    assert!(Parser::new(tokens).parse_program().is_err());
    Ok(())
}

#[test]
fn test_parse_sizeof_pack_on_type() -> Result<(), Box<dyn std::error::Error>> {
    let code = "template <typename... Ts> void f() { int n = sizeof...(Ts); }";
    let tokens = Lexer::new(code).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    let Some(Decl::Template(tmpl)) = program.declarations.first() else {
        return Err("expected template decl".into());
    };
    let TemplateKind::Function(ref func) = tmpl.kind else {
        return Err("expected func".into());
    };
    let Some(ref body) = func.body else {
        return Err("expected body".into());
    };
    let StmtKind::Compound(ref stmts) = body.kind else {
        return Err("expected compound".into());
    };
    let StmtKind::VarDecl { ref init, .. } = stmts[0].kind else {
        return Err("expected vardecl".into());
    };
    let Some(Expr::SizeofPack { name, .. }) = init else {
        return Err("expected SizeofPack".into());
    };
    assert_eq!(name, "Ts");
    Ok(())
}

#[test]
fn test_parse_pack_expansion_contexts() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        template <typename T> T forward(T t);
        template <typename... Args>
        void g(Args... args) {
            f(args...);
            f(forward<Args>(args)...);
            int a[] = { args... };
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    let Some(Decl::Template(tmpl)) = program.declarations.get(1) else {
        return Err("expected template decl".into());
    };
    let TemplateKind::Function(ref func) = tmpl.kind else {
        return Err("expected function decl".into());
    };
    let Some(ref body) = func.body else {
        return Err("expected body".into());
    };
    let StmtKind::Compound(ref stmts) = body.kind else {
        return Err("expected compound stmt".into());
    };

    let StmtKind::Expr(Expr::Call { ref args, .. }) = stmts[0].kind else {
        return Err("expected call expr 0".into());
    };
    assert_eq!(args.len(), 1);
    assert!(matches!(&args[0], Expr::PackExpansion { pattern, .. } if matches!(**pattern, Expr::Identifier(ref id) if id == "args")));

    let StmtKind::Expr(Expr::Call { ref args, .. }) = stmts[1].kind else {
        return Err("expected call expr 1".into());
    };
    assert_eq!(args.len(), 1);
    assert!(matches!(&args[0], Expr::PackExpansion { pattern, .. } if matches!(**pattern, Expr::Call { .. })));

    let StmtKind::VarDecl { ref init, .. } = stmts[2].kind else {
        return Err("expected vardecl 2".into());
    };
    let Some(Expr::InitializerList { elements, .. }) = init else {
        return Err("expected initializer list".into());
    };
    assert_eq!(elements.len(), 1);
    assert!(matches!(&elements[0], Expr::PackExpansion { pattern, .. } if matches!(**pattern, Expr::Identifier(ref id) if id == "args")));

    Ok(())
}

#[test]
fn test_parse_pack_expansion_in_base_classes_and_ctor_inits() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        template <typename... Bases>
        struct Derived : public Bases... {
            Derived(Bases... args) : Bases(args)... {}
        };
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    let Some(Decl::Template(tmpl)) = program.declarations.first() else {
        return Err("expected template decl".into());
    };
    let TemplateKind::Class(ref cls) = tmpl.kind else {
        return Err("expected class template".into());
    };
    assert_eq!(cls.base_classes, vec!["Bases...".to_string()]);
    assert_eq!(cls.methods.len(), 1);
    assert_eq!(cls.methods[0].ctor_initializers.len(), 1);
    assert_eq!(cls.methods[0].ctor_initializers[0].0, "Bases...");
    assert_eq!(cls.methods[0].ctor_initializers[0].1.len(), 1);
    Ok(())
}

#[test]
fn test_parse_pack_expansion_in_template_args() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        template <typename... Ts>
        struct Tuple {};

        template <typename... Args>
        void helper() {
            Tuple<Args...> t;
            Tuple<int, Args..., double> t2;
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    assert_eq!(program.declarations.len(), 2);
    Ok(())
}

#[test]
fn test_parse_pack_expansion_in_ordinary_function() -> Result<(), Box<dyn std::error::Error>> {
    let code = "void g() { f(1, args..., 2); int a[] = { 1, args..., 2 }; }";
    let tokens = Lexer::new(code).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    let Some(Decl::Function(func)) = program.declarations.first() else {
        return Err("expected function".into());
    };
    let Some(ref body) = func.body else {
        return Err("expected body".into());
    };
    let StmtKind::Compound(ref stmts) = body.kind else {
        return Err("expected compound".into());
    };
    let StmtKind::Expr(Expr::Call { ref args, .. }) = stmts[0].kind else {
        return Err("expected call".into());
    };
    assert_eq!(args.len(), 3);
    assert!(matches!(&args[1], Expr::PackExpansion { .. }));

    let StmtKind::VarDecl { ref init, .. } = stmts[1].kind else {
        return Err("expected vardecl".into());
    };
    let Some(Expr::InitializerList { elements, .. }) = init else {
        return Err("expected init list".into());
    };
    assert_eq!(elements.len(), 3);
    assert!(matches!(&elements[1], Expr::PackExpansion { .. }));
    Ok(())
}

#[test]
fn test_parse_fold_expressions_all_forms() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("(... + args)", true),
        ("(args + ...)", true),
        ("(0 + ... + args)", true),
        ("(args + ... + 0)", true),
        ("(args + ... - 0)", false),
    ];
    for (src, should_succeed) in cases {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let res = parser.parse_expr();
        assert_eq!(res.is_ok(), should_succeed, "Failed on: {}", src);
    }
    Ok(())
}

