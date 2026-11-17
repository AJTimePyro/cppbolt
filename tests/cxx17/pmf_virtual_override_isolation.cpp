#include <cstdio>

// Two unrelated bases declare distinct virtuals with the same name. The
// derived class does not override them.
struct B1 { virtual ~B1() {} virtual int h() { return 1; } };
struct B2 { virtual ~B2() {} virtual int h() { return 2; } };
struct Neither : B1, B2 {};

// Partial override in a non-virtual diamond: B overrides A::f, C does not.
struct A { virtual ~A() {} virtual int f() { return 1; } };
struct B : A { int f() override { return 2; } };
struct C : A {};
struct Diamond : B, C {};

int main() {
    Neither n{};
    B1* b1 = &n;
    B2* b2 = &n;
    int (B1::* p1)() = &B1::h;
    int (B2::* p2)() = &B2::h;
    std::printf("%d %d\n", b1->h(), b2->h());
    std::printf("%d %d\n", (n.*p1)(), (n.*p2)());

    Diamond d{};
    B* b = &d;
    C* c = &d;
    int (A::* pf)() = &A::f;
    std::printf("%d %d\n", b->f(), c->f());
    std::printf("%d %d\n", (b->*pf)(), (c->*pf)());
    return 0;
}
