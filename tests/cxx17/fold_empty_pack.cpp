#include <cstdio>

template <typename... Args>
bool fold_and(Args... args) {
    return (... && args);
}

template <typename... Args>
bool fold_or(Args... args) {
    return (... || args);
}

template <typename... Args>
void fold_comma(Args... args) {
    (... , args);
}

template <typename... Args>
int fold_bin_add(Args... args) {
    return (42 + ... + args);
}

int main() {
    printf("and_empty: %d\n", fold_and() ? 1 : 0);
    printf("or_empty: %d\n", fold_or() ? 1 : 0);
    fold_comma();
    printf("comma_empty: ok\n");
    printf("bin_empty: %d\n", fold_bin_add());
    return 0;
}
