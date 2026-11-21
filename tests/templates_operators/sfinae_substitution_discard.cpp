#include <cstdio>

// The first candidate's array bound becomes negative for N = 4, so its
// substitution fails and only the second candidate remains viable.
template <int N> int pick(char (*p)[N - 100]) { return 1; }
template <int N> int pick(int x) { return 2; }

int main() {
    std::printf("%d\n", pick<4>(0));
    return 0;
}
