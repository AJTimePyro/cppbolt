#include <cstdio>

template <typename... Ts>
void f(int x) throw(Ts...) {
    std::printf("dynamic throw: %d\n", x);
}

template <typename... Ts>
void g() throw() {
    std::printf("empty throw\n");
}

int main() {
    f<int, double>(123);
    g<char>();
    return 0;
}
