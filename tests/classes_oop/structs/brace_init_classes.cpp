#include <cstdio>

struct Base {
    int base_val;
    Base(int b) : base_val{b} {}
    Base() : base_val{0} {}
};

struct Derived : Base {
    int m1;
    int m2;
    Derived(int b, int x, int y) : Base{b}, m1{x}, m2{y} {}
    Derived() : Base{}, m1{0}, m2{0} {}
};

struct Foo {
    int a;
    int b;
    Foo(int x, int y) : a{x}, b{y} {}
    Foo() : a{100}, b{200} {}
};

int main() {
    Derived d1{10, 20, 30};
    Derived d2 = {40, 50, 60};
    Derived d3{};

    std::printf("d1: %d, %d, %d\n", d1.base_val, d1.m1, d1.m2);
    std::printf("d2: %d, %d, %d\n", d2.base_val, d2.m1, d2.m2);
    std::printf("d3: %d, %d, %d\n", d3.base_val, d3.m1, d3.m2);

    Foo f1{5, 6};
    Foo f2{};
    std::printf("f1: %d, %d\n", f1.a, f1.b);
    std::printf("f2: %d, %d\n", f2.a, f2.b);

    return 0;
}
