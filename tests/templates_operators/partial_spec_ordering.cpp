#include <cstdio>
template <typename T> struct Rank { static const int v = 0; };
template <typename T> struct Rank<T*> { static const int v = 1; };
template <typename T> struct Rank<const T*> { static const int v = 2; };
int main() { int x = 0; std::printf("%d %d %d\n", Rank<int>::v, Rank<int*>::v, Rank<const int*>::v); (void)x; }
