#include <cstdio>

// Instantiating Trait<No> fails inside the sizeof operand. The error comes
// from another template's instantiation and must stay a hard error.
template<class T> struct Trait { using type = typename T::type; };
template<class T> typename Trait<T>::type boom(T x);

template<class T>
auto f(T x) -> decltype(sizeof((boom(x)))) { return 1; }

struct No {};
int f(...) { return 2; }

int main() { No n; return f(n); }
