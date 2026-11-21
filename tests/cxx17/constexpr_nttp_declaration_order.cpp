#include <cstdio>
#include <type_traits>

template <int N> struct Box { static const int v = N; };
template <bool B> struct Flag { static const int v = B ? 1 : 0; };

constexpr int first = 3;
constexpr int second = first + 2;
constexpr bool derived = std::is_integral<int>::value;
constexpr bool negated = !derived;

static_assert(second == 5, "chained constexpr");

int main() {
    std::printf("%d %d %d\n", Box<first>::v, Box<second>::v, Flag<negated>::v);
    return 0;
}
