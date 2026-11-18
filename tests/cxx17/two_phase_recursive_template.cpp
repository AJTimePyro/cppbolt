#include <cstdio>

template <class T> int recurse(T n) {
    return n <= 0 ? 0 : 1 + recurse(n - 1);
}

int main() {
    std::printf("%d\n", recurse(4));
    return 0;
}
