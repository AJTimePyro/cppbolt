#include <cstdio>
template <typename...> struct List {};
template <typename T> struct HasPtrs { static const int v = 0; };
template <typename... Ts> struct HasPtrs<List<Ts*...>> { static const int v = 1; };
int main() { std::printf("%d %d\n", HasPtrs<List<int*, double*>>::v, HasPtrs<List<int>>::v); }
