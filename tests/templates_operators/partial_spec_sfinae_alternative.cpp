#include <cstdio>
struct Yes { using type = int; };
struct No { };
template <typename T, typename U> struct Pick { static const int v = 0; };
template <typename T> struct Pick<T, decltype(alignof(typename T::type))> { static const int v = 1; };
template <typename T> struct Pick<T, void*> { static const int v = 2; };
int main() { std::printf("%d %d %d\n", Pick<Yes, size_t>::v, Pick<No, void*>::v, Pick<Yes, void*>::v); }
