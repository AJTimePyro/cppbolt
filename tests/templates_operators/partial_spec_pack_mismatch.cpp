#include <cstdio>
template <typename... Ts> struct First { static const int v = 0; };
template <typename A, typename B> struct First<A, B> { static const int v = 1; };
int main() { std::printf("%d %d\n", First<int>::v, First<int, double>::v); }
