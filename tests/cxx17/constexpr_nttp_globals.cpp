#include <cstdio>
#include <type_traits>

template <int N> struct Bucket { static const int value = N; };
template <bool B> struct Flag { static const int value = B ? 1 : 0; };

constexpr int size = 4;
constexpr bool enabled = !false;
constexpr bool integral = std::is_integral<int>::value;
constexpr bool not_integral = !std::is_integral<double>::value;

template <int N> int choose(Bucket<N>*) { return 1; }
int choose(void*) { return 2; }

int main() {
    Bucket<size> b;
    void* raw = nullptr;
    std::printf("%d %d %d %d\n", Bucket<size>::value, Flag<enabled>::value,
                Flag<integral>::value, Flag<not_integral>::value);
    std::printf("%d %d\n", choose(&b), choose(raw));
    return 0;
}
