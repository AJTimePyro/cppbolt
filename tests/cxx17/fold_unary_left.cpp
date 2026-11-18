#include <cstdio>

template <typename... Args>
int sub_left(Args... args) {
    return (... - args);
}

template <typename... Args>
int sum_left(Args... args) {
    return (... + args);
}

int main() {
    printf("sub_left: %d\n", sub_left(20, 5, 3));
    printf("sum_left: %d\n", sum_left(1, 2, 3, 4));
    printf("sub_single: %d\n", sub_left(42));
    printf("sum_single: %d\n", sum_left(99));
    return 0;
}
