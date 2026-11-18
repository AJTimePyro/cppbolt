#include <cstdio>

template <typename Init, typename... Args>
int sub_bin_right(Init init, Args... args) {
    return (args - ... - init);
}

template <typename Init, typename... Args>
int sum_bin_right(Init init, Args... args) {
    return (args + ... + init);
}

int main() {
    printf("sub_bin_right: %d\n", sub_bin_right(1, 20, 10, 5));
    printf("sub_bin_right_empty: %d\n", sub_bin_right(42));
    printf("sum_bin_right: %d\n", sum_bin_right(10, 1, 2, 3));
    printf("sum_bin_right_empty: %d\n", sum_bin_right(7));
    return 0;
}
