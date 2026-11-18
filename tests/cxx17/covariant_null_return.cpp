#include <cstdio>

struct Pad {
    long long p;
};

struct A {
    virtual ~A() {}
    virtual A* self() { return this; }
};

struct B : Pad, A {
    bool null_result;
    B() : null_result(false) {}
    B* self() override { return null_result ? nullptr : this; }
};

int main() {
    B b;
    A* a = static_cast<A*>(&b);
    A* n = a->self();
    std::printf("nonnull=%d\n", n == a);

    b.null_result = true;
    A* z = a->self();
    std::printf("null=%d\n", z == nullptr);

    A* (A::*pm)() = &A::self;
    A* z2 = (a->*pm)();
    std::printf("pmf=%d\n", z2 == nullptr);
    return 0;
}
