#include <cstdio>

struct A {
    template <class T> auto call(T value) -> decltype(g(value)) { return g(value); }
};

namespace N {
struct S {};
int g(S) { return 7; }
}

int main() {
    A a;
    std::printf("%d\n", a.call(N::S{}));
    return 0;
}
