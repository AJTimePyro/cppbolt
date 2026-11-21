#include <cstdio>

namespace N {
struct S {
    friend int operator+(S, S) { return 7; }
};
}

int main() {
    N::S a, b;
    std::printf("%d\n", a + b);
    return 0;
}
