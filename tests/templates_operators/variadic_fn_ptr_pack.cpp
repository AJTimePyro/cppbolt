#include <cstdio>

template <typename... Args>
void invoke(void (*fn)(Args...), Args... args) {
    fn(args...);
}

void print_sum(int a, int b) {
    std::printf("sum: %d\n", a + b);
}

void print_three(int a, int b, int c) {
    std::printf("three: %d %d %d\n", a, b, c);
}

int main() {
    invoke(print_sum, 10, 20);
    invoke(print_three, 1, 2, 3);
    return 0;
}
