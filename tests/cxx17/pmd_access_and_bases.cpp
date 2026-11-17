#include <cstdio>

class C {
    int priv;
public:
    C() : priv(4) {}
    auto self() { return &C::priv; }   // access is allowed inside the class
};

struct A { int a; int f() { return a; } };
struct B : A { int b; };

int main() {
    C c;
    auto p = c.self();
    std::printf("%d\n", c.*p);

    B obj{};
    obj.a = 7; obj.b = 8;
    int A::* pa = &A::a;
    int B::* pb = pa;                 // public, unambiguous base
    int (A::* pf)() = &A::f;
    int (B::* pbf)() = pf;
    std::printf("%d %d\n", obj.*pb, (obj.*pbf)());
    return 0;
}
