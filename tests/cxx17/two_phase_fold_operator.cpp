#include <cstdio>

template <class... Ts> int sum(Ts... ts) {
    return (ts + ... + 0);
}

namespace N {
struct A {};
int operator+(A, int) { std::printf("op\n"); return 1; }
}

int main() {
    std::printf("%d\n", sum(N::A{}));
    return 0;
}
