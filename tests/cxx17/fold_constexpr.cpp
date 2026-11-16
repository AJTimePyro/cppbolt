#include <cstdio>

template <typename... Args>
constexpr int cexpr_sum(Args... args) {
    return (... + args);
}

template <typename... Args>
constexpr int cexpr_sub_left(Args... args) {
    return (... - args);
}

template <typename... Args>
constexpr bool cexpr_all_positive(Args... args) {
    return ((args > 0) && ...);
}

constexpr int g_sum = cexpr_sum(10, 20, 30);
constexpr int g_left = cexpr_sub_left(100, 20, 10);
constexpr bool g_all_pos = cexpr_all_positive(1, 2, 3);
constexpr bool g_not_all_pos = cexpr_all_positive(1, -2, 3);

int main() {
    printf("cexpr_sum: %d\n", g_sum);
    printf("cexpr_left: %d\n", g_left);
    printf("cexpr_all: %d\n", g_all_pos ? 1 : 0);
    printf("cexpr_not_all: %d\n", g_not_all_pos ? 1 : 0);
    return 0;
}
