use super::*;
use crate::ast::types::FunctionQualifiers;
use crate::lexer::Lexer;
use crate::parser::Parser;

#[test]
fn test_sema_namespace_alias_resolution() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        namespace Original { int value = 42; int get() { return value; } }
        namespace Alias = Original;
        int main() {
            return Alias::get() + Alias::value;
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    sema.analyze_program(&mut program)?;
    Ok(())
}

#[test]
fn test_reinterpret_cast_rejects_float_to_int() -> Result<(), Box<dyn std::error::Error>> {
    let code = "int x = reinterpret_cast<int>(3.14f);";
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    let res = sema.analyze_program(&mut program);
    assert!(
        res.is_err(),
        "reinterpret_cast between float and int must be rejected"
    );
    Ok(())
}

#[test]
fn test_reinterpret_cast_bypasses_conversion_operator() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        struct Foo { operator int*() const { return nullptr; } };
        void test() { Foo f; int* p = reinterpret_cast<int*>(f); }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    let res = sema.analyze_program(&mut program);
    assert!(
        res.is_err(),
        "reinterpret_cast cannot invoke user conversion operator on class"
    );
    Ok(())
}

#[test]
fn test_reinterpret_cast_reference_rejects_rvalue() -> Result<(), Box<dyn std::error::Error>> {
    let code = "void test() { int& r = reinterpret_cast<int&>(42); }";
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    let res = sema.analyze_program(&mut program);
    assert!(
        res.is_err(),
        "reinterpret_cast to reference must reject rvalue"
    );
    Ok(())
}

#[test]
fn test_reinterpret_cast_reference_accepts_float() -> Result<(), Box<dyn std::error::Error>> {
    let code = "void test() { float f = 1.0f; int& r = reinterpret_cast<int&>(f); }";
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    let res = sema.analyze_program(&mut program);
    assert!(
        res.is_ok(),
        "reinterpret_cast to reference must accept float lvalues: {:?}",
        res.err()
    );
    Ok(())
}

#[test]
fn test_nested_namespace_alias() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        namespace Outer {
            namespace Real { int val = 100; }
            namespace Inner = Real;
        }
        int test1() {
            return Outer::Inner::val;
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    sema.analyze_program(&mut program)?;
    Ok(())
}

#[test]
fn test_block_scope_conflicting_alias_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        namespace N1 { int a = 1; }
        namespace N2 { int a = 2; }
        void test() {
            namespace Local = N1;
            namespace Local = N2;
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    let res = sema.analyze_program(&mut program);
    assert!(
        res.is_err(),
        "conflicting alias in block scope must be rejected"
    );
    Ok(())
}

#[test]
fn test_block_scope_namespace_alias_does_not_leak() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        namespace N { int a = 1; }
        void test() {
            {
                namespace Local = N;
            }
            int x = Local::a;
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    let res = sema.analyze_program(&mut program);
    assert!(
        res.is_err(),
        "block-scoped alias must not leak outside its scope"
    );
    Ok(())
}

#[test]
fn test_member_template_default_argument_resolution() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        struct Worker {
            template<typename T>
            int execute(T val, int bonus = 10) {
                return (int)val + bonus;
            }
        };
        int test() {
            Worker w;
            return w.execute(5) + w.execute(5, 20);
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    sema.analyze_program(&mut program)?;
    Ok(())
}

#[test]
fn test_function_template_same_arity_overloads() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        template<typename T>
        int inspect(T val);

        template<typename T>
        int inspect(T val) {
            return 1;
        }

        template<typename T>
        int inspect(T* ptr) {
            return 2;
        }

        int test() {
            int x = 42;
            return inspect(x) + inspect(&x);
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    sema.analyze_program(&mut program)?;
    Ok(())
}

#[test]
fn test_out_of_class_template_method_matching_params() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        template<typename T>
        struct Boxed {
            void set(T val);
            void set(T* ptr);
            T stored;
        };

        template<typename T>
        void Boxed<T>::set(T val) {
            this->stored = val;
        }

        template<typename T>
        void Boxed<T>::set(T* ptr) {
            this->stored = *ptr;
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    sema.analyze_program(&mut program)?;
    Ok(())
}

#[test]
fn test_synthesized_copy_operations_include_base_classes() -> Result<(), Box<dyn std::error::Error>>
{
    let code = r#"
        struct Base {
            int b;
            ~Base() {}
        };
        struct Derived : public Base {
            int d;
        };
        int test() {
            Derived d1;
            d1.b = 10;
            d1.d = 20;
            Derived d2 = d1;
            d2 = d1;
            return d2.b + d2.d;
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    sema.analyze_program(&mut program)?;
    Ok(())
}

#[test]
fn test_instantiated_template_class_synthesizes_copy_operations()
-> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        struct Resource {
            int id;
            ~Resource() {}
        };
        template<typename T>
        struct Container {
            T item;
        };
        int test() {
            Container<Resource> c1;
            Container<Resource> c2 = c1;
            c2 = c1;
            return 0;
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    sema.analyze_program(&mut program)?;
    Ok(())
}

#[test]
fn test_out_of_class_namespace_qualified_constructor() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        namespace Outer {
            namespace Inner {
                struct Widget {
                    int val;
                    Widget();
                    ~Widget();
                };
            }
        }
        Outer::Inner::Widget::Widget() {
            val = 42;
        }
        Outer::Inner::Widget::~Widget() {}
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    sema.analyze_program(&mut program)?;
    Ok(())
}

#[test]
fn test_member_template_parameter_shadowing() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        template<typename T>
        struct OuterHolder {
            T outer_val;

            template<typename T>
            T convert(T inner_val) {
                return inner_val;
            }
        };
        int test() {
            OuterHolder<int> h;
            double d = h.convert(3.14);
            return (int)d;
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    sema.analyze_program(&mut program)?;
    Ok(())
}

#[test]
fn test_functor_call_dispatch_requires_call_operator() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
        struct NotFunctor {
            template<typename T>
            void push(T item) {}
        };
        int test() {
            NotFunctor nf;
            nf();
            return 0;
        }
    "#;
    let tokens = Lexer::new(code).tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut program = parser.parse_program()?;
    let mut sema = Sema::new();
    let res = sema.analyze_program(&mut program);
    assert!(
        res.is_err(),
        "Calling an object without operator() must fail even if it has member templates"
    );
    Ok(())
}

#[test]
fn test_substitute_scalar_type() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = std::collections::HashMap::new();
    mapping.insert("T".to_string(), TemplateArg::Type(Type::Int));

    let t_param = Type::TypeParam("T".to_string());
    assert_eq!(
        template_subst::substitute_type(&t_param, &mapping, None)?,
        Type::Int
    );

    let ptr = Type::Pointer(Box::new(Type::TypeParam("T".to_string())));
    assert_eq!(
        template_subst::substitute_type(&ptr, &mapping, None)?,
        Type::Pointer(Box::new(Type::Int))
    );
    Ok(())
}

#[test]
fn test_substitute_injected_class_name() -> Result<(), Box<dyn std::error::Error>> {
    let mapping = std::collections::HashMap::new();
    let target_ty = Type::Struct("Holder<int>".to_string());
    let bare = Type::Struct("Holder".to_string());

    let res = template_subst::substitute_type(&bare, &mapping, Some(("Holder", &target_ty)))?;
    assert_eq!(res, target_ty);
    Ok(())
}

fn dummy_func(name: &str, ret: Type, param: Type, is_const: bool) -> FunctionDecl {
    FunctionDecl {
        qualifiers: FunctionQualifiers::default(),
        name: name.to_string(),
        return_type: ret,
        params: vec![Param {
            name: "val".to_string(),
            ty: param,
            default_val: None,
            is_pack: false,
        }],
        body: None,
        is_virtual: false,
        is_pure_virtual: false,
        is_const,
        is_explicit: false,
        ref_qualifier: Default::default(),
        is_static: false,
        ctor_initializers: Vec::new(),
        template_params: Vec::new(),
        line: 1,
        col: 1,
    }
}

#[test]
fn test_substitute_function() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = std::collections::HashMap::new();
    mapping.insert("T".to_string(), TemplateArg::Type(Type::Int));
    let func = dummy_func(
        "get",
        Type::TypeParam("T".to_string()),
        Type::TypeParam("T".to_string()),
        true,
    );
    let subst = template_subst::substitute_function(&func, &mapping, None, None)?;
    assert_eq!(subst.return_type, Type::Int);
    assert_eq!(subst.params[0].ty, Type::Int);
    Ok(())
}

#[test]
fn test_substitute_struct() -> Result<(), Box<dyn std::error::Error>> {
    let mut mapping = std::collections::HashMap::new();
    mapping.insert("T".to_string(), TemplateArg::Type(Type::Int));
    let tclass = TemplateClass {
        name: "Holder".to_string(),
        pattern_args: None,
        is_class: true,
        is_final: false,
        base_classes: Vec::new(),
        base_access: Vec::new(),
        fields: vec![("value".to_string(), Type::TypeParam("T".to_string()), None)],
        methods: vec![dummy_func(
            "Holder",
            Type::Void,
            Type::TypeParam("T".to_string()),
            false,
        )],
        friends: Vec::new(),
        static_fields: Vec::new(),
        private_members: Vec::new(),
        nested_structs: Vec::new(),
        alignment: None,
    };

    let subst = template_subst::substitute_struct(&tclass, &mapping, "Holder<int>", (1, 1), None)?;
    let Decl::StructDecl {
        name,
        fields,
        methods,
        ..
    } = subst
    else {
        unreachable!("expected StructDecl");
    };
    assert_eq!(name, "Holder<int>");
    assert_eq!(fields[0].1, Type::Int);
    assert_eq!(methods[0].name, "Holder<int>");
    assert_eq!(methods[0].params[0].ty, Type::Int);
    Ok(())
}

#[test]
fn test_non_const_reference_rejects_temporary() {
    let sema = Sema::new();
    let ref_ty = Type::Reference(Box::new(Type::Int));
    let val_ty = Type::Int;
    let literal = Expr::IntLiteral(42, Type::Int);
    assert!(!sema.can_assign_expr(&ref_ty, &val_ty, Some(&literal)));

    let const_ref_ty = Type::Reference(Box::new(Type::Const(Box::new(Type::Int))));
    assert!(sema.can_assign_expr(&const_ref_ty, &val_ty, Some(&literal)));
}

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
    assert!(sema.resolve_decltype_type(&bad, false, 1, 1).is_err());
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
    let ty = sema.resolve_decltype_type(&good, false, 1, 1)?;
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
