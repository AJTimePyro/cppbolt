#include <cstdio>

template <class T> void binary(T t) { t + t; }
template <class T> void unary(T t) { !t; }

namespace N {
struct B {};
int operator+(B, B) { std::printf("plus\n"); return 1; }
int operator!(B) { std::printf("not\n"); return 0; }
}

template <class T> int builtin(T t) { return t + 1; }

int main() {
    binary(N::B{});
    unary(N::B{});
    std::printf("%d\n", builtin(2));
    return 0;
}
