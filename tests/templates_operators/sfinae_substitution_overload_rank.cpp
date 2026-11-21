#include <cstdio>

// One candidate is discarded during substitution; the remaining candidates
// are ranked normally and the exact match wins.
template <int N> int rank(char (*p)[N - 100]) { return 1; }
template <int N> int rank(long x) { return 2; }
template <int N> int rank(int x) { return 3; }

int main() {
    std::printf("%d\n", rank<4>(1));
    return 0;
}
