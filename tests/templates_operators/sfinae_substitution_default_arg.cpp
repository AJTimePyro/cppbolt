#include <cstdio>

// M is defaulted to a value that cannot form the parameter's array type.
template <int N, int M = N - 100> int pick(char (*p)[M]) { return 1; }
template <int N> int pick(int x) { return 2; }

int main() {
    std::printf("%d\n", pick<4>(0));
    return 0;
}
