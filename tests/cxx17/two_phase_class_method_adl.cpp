#include <cstdio>

template <class T> struct A {
    void call(T value) { g(value); }
};

namespace N {
struct S {};
void g(S) { std::printf("N::g\n"); }
}

int main() {
    A<N::S> a;
    a.call(N::S{});
    return 0;
}
