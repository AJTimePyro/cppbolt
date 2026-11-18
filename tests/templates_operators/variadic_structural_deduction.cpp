#include <cstdio>

template <typename... Ts>
struct Tuple {};

template <typename... As, typename... Bs>
void f(Tuple<As...>, Tuple<Bs...>) {
    std::printf("As: %zu, Bs: %zu\n", sizeof...(As), sizeof...(Bs));
}

template <typename First, typename... Rest>
void g(Tuple<First, Rest...>) {
    std::printf("Rest: %zu\n", sizeof...(Rest));
}

int main() {
    f(Tuple<int, double, char>{}, Tuple<float, long>{});
    f(Tuple<>{}, Tuple<int>{});
    g(Tuple<int, double, char>{});
    return 0;
}
