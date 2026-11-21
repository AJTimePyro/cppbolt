#include <cstdio>
template <typename... Ts> struct Select { static const int v = 0; };
template <typename T, typename... Rest> struct Select<T, Rest...> { static const int v = 1; };
template <typename A, typename B> struct Select<A, B> { static const int v = 2; };
int main() { std::printf("%d %d %d\n", Select<int>::v, Select<int, double>::v, Select<int, double, char>::v); }
