#include <cstdio>
template <typename...> struct List {};
template <typename T> struct HasList { static const int v = 0; };
template <typename... Ts> struct HasList<List<Ts...>> { static const int v = 1; };
int main() { std::printf("%d %d\n", HasList<List<int, double>>::v, HasList<int>::v); }
