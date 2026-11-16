use super::*;
use crate::lexer::Lexer;
use crate::parser::Parser;

fn analyze_ok(code: &str) -> bool {
    let Ok(tokens) = Lexer::new(code).tokenize() else {
        return false;
    };
    let Ok(mut program) = Parser::new(tokens).parse_program() else {
        return false;
    };
    let mut sema = Sema::new();
    sema.analyze_program(&mut program).is_ok()
}

#[test]
fn test_sema_pointer_to_member_data_and_function() {
    assert!(analyze_ok(
        "struct Point { int x; int y; }; \
         int main() { auto pm = &Point::x; Point p; int val = p .* pm; }"
    ));
    assert!(analyze_ok(
        "struct Point { int x; int y; }; \
         int main() { auto pm = &Point::x; Point p; Point* ptr = &p; int val = ptr ->* pm; }"
    ));
    assert!(analyze_ok(
        "struct Base { int b; }; \
         struct Derived : public Base { int d; }; \
         int main() { int Base::* pmb = &Base::b; int Derived::* pmd = pmb; }"
    ));
    assert!(analyze_ok(
        "struct Widget { int calc(int a) { return a; } }; \
         int main() { auto pmf = &Widget::calc; }"
    ));
    assert!(analyze_ok(
        "struct Point { int x; }; \
         int main() { int Point::* pm = nullptr; }"
    ));
    assert!(!analyze_ok(
        "struct A { int a; }; \
         struct B { int b; }; \
         int main() { A obj; int B::* pm = &B::b; return obj .* pm; }"
    ));
    assert!(analyze_ok(
        "struct Foo { int add(int a, int b) { return a + b; } }; \
         int main() { Foo f; auto pm = &Foo::add; return (f .* pm)(1, 2); }"
    ));
    assert!(analyze_ok(
        "struct S { int x; }; \
         template <typename T, typename C> T get_val(C* obj, T C::* pm) { return obj->*pm; } \
         int main() { S s; s.x = 42; return get_val(&s, &S::x); }"
    ));
}
