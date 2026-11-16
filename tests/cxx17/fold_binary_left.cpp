#include <cstdio>

template <typename Init, typename... Args>
int sub_bin_left(Init init, Args... args) {
    return (init - ... - args);
}

template <typename Init, typename... Args>
int sum_bin_left(Init init, Args... args) {
    return (init + ... + args);
}

int main() {
    printf("sub_bin_left: %d\n", sub_bin_left(100, 20, 10, 5));
    printf("sub_bin_left_empty: %d\n", sub_bin_left(100));
    printf("sum_bin_left: %d\n", sum_bin_left(1, 2, 3, 4));
    printf("sum_bin_left_empty: %d\n", sum_bin_left(0));
    return 0;
}
