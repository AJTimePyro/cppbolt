#include <cstdio>

template <typename... A>
int assign_fold(A... a) {
    int x = 0;
    (x = ... = a);
    return x;
}

template <typename... A>
int compound_fold(A... a) {
    int x = 0;
    (x += ... += a);
    return x;
}

int main() {
    int y = 0;
    (y += 1) += 2;
    std::printf("%d %d %d\n", assign_fold(1, 2, 3), compound_fold(1, 2), y);
    return 0;
}
