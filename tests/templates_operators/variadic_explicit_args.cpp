#include <cstdio>

template <typename... Ts>
void foo(Ts... args) {
    std::printf("explicit pack arity: %zu\n", sizeof...(Ts));
}

template <typename First, typename... Rest>
void bar(First f, Rest... rest) {
    std::printf("bar: %d %zu\n", f, sizeof...(rest));
}

int main() {
    foo<int, int>(1, 2);
    foo<int, double, char>(10, 2.5, 'Z');
    bar<int, int, int>(100, 200, 300);
    return 0;
}
