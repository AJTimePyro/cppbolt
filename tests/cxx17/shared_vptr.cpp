#include <cstdio>

struct A {
    virtual ~A() {}
    virtual int f() { return 1; }
};

struct Pad {
    long long x;
};

struct B : Pad, A {
    virtual int g() { return 2; }
};

struct C {
    virtual ~C() {}
    virtual int h() { return 3; }
};

struct D : B, C {
    int g() override { return 22; }
    int h() override { return 33; }
};

int main() {
    D d;
    B* b = &d;
    C* c = &d;
    A* a = &d;
    std::printf("g=%d h=%d f=%d\n", b->g(), c->h(), a->f());
    return 0;
}
