#include <cstdio>
struct Yes { using type = int; };
template <typename T, typename U> struct H { static const int v = 0; };
template <typename T> struct H<T, decltype(alignof(typename T::type))> { static const int v = 1; };
template <typename T> struct H<T, size_t> { static const int v = 2; };
int main() { std::printf("%d\n", H<Yes, size_t>::v); }
