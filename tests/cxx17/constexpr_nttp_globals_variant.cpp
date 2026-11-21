#include <cstdio>
#include <type_traits>

template <int N> struct Grid { static const int value = N; };
template <bool B> struct Flag { static const int value = B ? 1 : 0; };
constexpr int bound = sizeof(int) * 2;
constexpr bool floating = std::is_integral<float>::value;
constexpr bool negated = !floating;

int main() {
    std::printf("%d %d %d %d\n", Grid<bound>::value,
                Grid<sizeof(int)>::value,
                Grid<negated + 5>::value,
                Flag<negated>::value);
    return 0;
}
