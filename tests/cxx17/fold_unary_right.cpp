#include <cstdio>

template <typename... Args>
int sub_right(Args... args) {
    return (args - ...);
}

template <typename... Args>
int div_right(Args... args) {
    return (args / ...);
}

int main() {
    printf("sub_right: %d\n", sub_right(20, 5, 3));
    printf("div_right: %d\n", div_right(100, 20, 2));
    printf("sub_single: %d\n", sub_right(42));
    printf("div_single: %d\n", div_right(99));
    return 0;
}
