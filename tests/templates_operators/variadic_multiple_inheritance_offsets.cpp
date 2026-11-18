#include <cstdio>

struct A {
    int a;
    int get_a() { return a; }
};

struct B {
    int b1;
    int b2;
    int get_b() { return b1 + b2; }
};

struct C {
    int c;
    int get_c() { return c; }
};

template <typename... Bases>
struct PackDerived : Bases... {
    int d;
    int get_d() { return d; }
};

int main() {
    PackDerived<A, B, C> obj;
    obj.a = 10;
    obj.b1 = 20;
    obj.b2 = 30;
    obj.c = 40;
    obj.d = 50;

    A* pa = &obj;
    B* pb = &obj;
    C* pc = &obj;

    std::printf("pa=%d pb=%d pc=%d d=%d\n",
                pa->get_a(), pb->get_b(), pc->get_c(), obj.get_d());

    PackDerived<A, B, C>* back_b = static_cast<PackDerived<A, B, C>*>(pb);
    PackDerived<A, B, C>* back_c = static_cast<PackDerived<A, B, C>*>(pc);

    std::printf("back_b=%d back_c=%d\n", back_b->a, back_c->a);
    return 0;
}
