#include <cstdio>
template <typename... Ts> struct Pack { static const int v = 0; };
template <typename T, typename... Rest> struct Pack<T, Rest...> { static const int v = 1; };
int main() { std::printf("%d %d\n", Pack<>::v, Pack<int, double>::v); }
