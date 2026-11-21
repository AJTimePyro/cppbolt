#include <cstdio>

namespace N {
struct A {
    template <class U> friend void f(A, U) { std::printf("f\n"); }
};
}

int main() {
    N::A a;
    f(a, 1);
    f(a, 2.0);
    f(a, 1);
    return 0;
}
