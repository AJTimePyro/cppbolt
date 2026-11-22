#include <cstdio>

struct B1 {
    int a;
};

struct B2 {
    virtual ~B2() {}
    int b;
};

struct D : B1, B2 {
    D() {
        a = 11;
        b = 22;
    }
};

int main() {
    B2* p = new D;
    B1* q = new D;
    std::printf("b=%d a=%d\n", p->b, q->a);
    delete p;
    return 0;
}
