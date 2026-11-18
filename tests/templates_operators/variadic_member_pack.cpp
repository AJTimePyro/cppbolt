#include <cstdio>

template <typename T>
struct S {
    template <typename... Us>
    int f(Us... us) { return 7; }
};

template <typename... Ts>
struct V {
    template <typename... Us>
    int g(Us... us) { return 9; }
};

int main() {
    S<int> s;
    V<int, double> v;
    std::printf("%d %d\n", s.f(1, 2), v.g(1, 2, 3));
    return 0;
}
