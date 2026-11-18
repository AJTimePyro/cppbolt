#include <cstdio>

struct S { int x; int f() { return x; } };
struct Holder {
    int S::* data;
    int (S::*cb)();
};

int (S::*get_callback())() { return &S::f; }
int invoke(S& s, int (S::*p)()) { return (s.*p)(); }
using PF = int (S::*)();

int main() {
    S s{8};
    Holder h{&S::x, &S::f};
    auto p = get_callback();
    PF q = get_callback();
    std::printf("%d %d %d %d\n", s.*h.data, (s.*h.cb)(), (s.*p)(), invoke(s, q));
    h.data = &S::x;
    h.cb = &S::f;
    std::printf("%d %d\n", s.*h.data, (s.*h.cb)());
    return 0;
}
