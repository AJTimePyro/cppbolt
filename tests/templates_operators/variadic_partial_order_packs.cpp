#include <cstdio>

template <typename... Ts>
int pick(Ts...) { return 1; }

template <typename T, typename... Rest>
int pick(T, Rest...) { return 2; }

template <typename... Ts>
int two(Ts...) { return 1; }

template <typename T, typename U>
int two(T, U) { return 2; }

template <typename T, typename... Rest>
int three(T, Rest...) { return 3; }

template <typename T, typename U, typename... Rest>
int three(T, U, Rest...) { return 4; }

int main() {
    std::printf("%d %d %d %d %d\n", pick(1), pick(1, 2), two(1, 2), three(1, 2), three(1, 2, 3));
    return 0;
}
