use super::*;
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
fn test_synthesized_copy_operations_include_base_classes() -> Result<(), Box<dyn std::error::Error>> {
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
fn test_instantiated_template_class_synthesizes_copy_operations() -> Result<(), Box<dyn std::error::Error>> {
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
    assert!(res.is_err(), "Calling an object without operator() must fail even if it has member templates");
    Ok(())
}
