#include <cstdio>

struct B {
    virtual ~B() {}
    virtual int f(int n) { return n + 1; }
    virtual int f(double x) { return (int)x + 10; }
    virtual int cv() const { return 1; }
    virtual int ref() & { return 2; }
};
struct D : B {
    int f(int n) override { return n + 100; }
    int f(double x) override { return (int)x + 200; }
    int cv() const override { return 3; }
    int ref() & override { return 4; }
};

struct Pad { long long pad; };
struct B1 { virtual ~B1() {} virtual int g() { return 1; } };
struct B2 { virtual ~B2() {} virtual int g() { return 2; } };
struct E : B1, B2 { int g() override { return 9; } };

int main() {
    D d;
    B* b = &d;
    int (B::* pi)(int) = &B::f;
    int (B::* pd)(double) = &B::f;
    std::printf("%d %d %d %d %d %d\n", b->f(1), b->f(2.0), (d.*pi)(3), (d.*pd)(4.0),
                b->cv(), b->ref());
    E e;
    B1* b1 = &e;
    B2* b2 = &e;
    std::printf("%d %d\n", b1->g(), b2->g());
    return 0;
}
