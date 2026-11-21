#include <cstdio>
struct Yes { static int value; };
struct No { };
template <typename T, typename U> struct HasValue { static const int v = 0; };
template <typename T> struct HasValue<T, decltype(T::value)> { static const int v = 1; };
int Yes::value = 7;
int main() { std::printf("%d %d\n", HasValue<Yes, int>::v, HasValue<No, int>::v); }
