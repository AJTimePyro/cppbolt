#include <cstdio>

struct Base {
    virtual ~Base() {}
    virtual int value() { return 1; }
};
struct Middle { int pad; };

// `override` without repeating `virtual`, with a secondary polymorphic base.
struct Derived : Middle, Base {
    int value() override { return 2; }
};

// Implicit override: same signature, no keyword at all.
struct Implicit : Middle, Base {
    int value() { return 3; }
};

// Explicit virtual, no override keyword, secondary polymorphic base.
struct Explicit : Middle, Base {
    virtual int value() { return 4; }
};

int main() {
    Derived d{};
    Implicit i{};
    Explicit e{};
    Base* bd = &d;
    Base* bi = &i;
    Base* be = &e;
    std::printf("%d %d %d\n", bd->value(), bi->value(), be->value());
    int (Base::* pm)() = &Base::value;
    std::printf("%d %d %d\n", (d.*pm)(), (i.*pm)(), (e.*pm)());
    return 0;
}
