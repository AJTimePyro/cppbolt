#include <cstdio>
struct Yes { using type = int; };
struct No { };
template <typename T, typename U> struct HasType { static const int v = 0; };
template <typename T> struct HasType<T, decltype(alignof(typename T::type))> { static const int v = 1; };
int main() {
    std::printf("%d %d %d %d\n", HasType<Yes, size_t>::v, HasType<No, size_t>::v,
                               HasType<Yes, size_t>::v, HasType<No, size_t>::v);
}
