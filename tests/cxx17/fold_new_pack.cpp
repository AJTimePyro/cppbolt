#include <cstdio>

template <typename... A>
int sum(A... a) {
    return (... + *new int(a));
}

int main() {
    std::printf("%d\n", sum(1, 2, 3));
    return 0;
}
