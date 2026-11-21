#include <cstdio>

template <class... Ts, class U> int count_pack(U) { return sizeof...(Ts); }
template <class T, class... Ts, class U> int count_two(U) { return sizeof...(Ts); }

// The pack overload consumes all explicit arguments; the single-parameter
// template is discarded for having too many explicit arguments.
template <class... Ts, class U> int select(U) { return 10 + sizeof...(Ts); }
template <class U> int select(U) { return -1; }

int main() {
    std::printf("%d %d %d %d\n",
                count_pack<int, double, char>(3),
                count_two<long, int, float, double>(3),
                select<int, double>(3),
                select<char>(3));
    return 0;
}
