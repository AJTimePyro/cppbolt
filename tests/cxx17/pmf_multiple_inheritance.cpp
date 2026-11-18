#include <cstdio>

struct B1 {
    virtual ~B1() {}
    virtual int f() { return 1; }
    int b1;
};
struct B2 {
    virtual ~B2() {}
    virtual int g() { return 2; }
    int b2;
};
struct D : B1, B2 {
    int d;
    int f() override { return 11; }
    int g() override { return 22; }
};

// Two unrelated virtual functions with the same name in both bases.
struct C1 { virtual ~C1() {} virtual int h() { return 1; } };
struct C2 { virtual ~C2() {} virtual int h() { return 2; } };
struct E : C1, C2 { int h() override { return 33; } };

int main() {
    D obj{};
    obj.b1 = 5; obj.b2 = 6; obj.d = 7;

    // Ordinary virtual calls through both base pointers reach the final
    // overrider with a correctly adjusted `this`.
    B1* b1 = &obj;
    B2* b2 = &obj;
    std::printf("%d %d\n", b1->f(), b2->g());
    std::printf("%d %d\n", obj.f(), obj.g());

    // Member-function pointers through each base and after conversion.
    int (B1::* pf)() = &B1::f;
    int (B2::* pg)() = &B2::g;
    int (D::* pdf)() = pf;
    int (D::* pdg)() = pg;
    std::printf("%d %d %d %d\n", (obj.*pf)(), (obj.*pg)(), (obj.*pdf)(), (obj.*pdg)());
    std::printf("%d %d\n", (b1->*pf)(), (b2->*pg)());
    std::printf("%d %d\n", (obj.*pdg)(), (b2->*pg)());

    // Taking a derived member pointer to an overridden base virtual.
    int (D::* pderived_f)() = &D::f;
    int (D::* pderived_g)() = &D::g;
    std::printf("%d %d\n", (obj.*pderived_f)(), (obj.*pderived_g)());

    E e{};
    int (C1::* p1)() = &C1::h;
    int (C2::* p2)() = &C2::h;
    C1* c1 = &e;
    C2* c2 = &e;
    std::printf("%d %d %d %d\n", (e.*p1)(), (e.*p2)(), (c1->*p1)(), (c2->*p2)());
    return 0;
}
