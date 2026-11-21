#include <cstdio>
#include <type_traits>

// An invalid nested type in the immediate substitution context discards only
// the affected candidate. The fallback overload or primary specialization wins.
template <class T> struct Id { typedef T type; };
struct NoType {};
template <class T> struct Maybe { typedef T type; };
template <> struct Maybe<NoType> {};

template <class T, class = void> struct One { static const int v = 0; };
template <class T> struct One<T, std::void_t<typename Maybe<T>::type> > { static const int v = 1; };

template <class T, class = void> struct Two { static const int v = 0; };
template <class T> struct Two<T, std::void_t<typename Id<typename Maybe<T>::type>::type> > {
  static const int v = 1;
};

template <class T> typename Maybe<T>::type pick(T t, int) { return t; }
template <class T> int pick(T, double) { return -1; }

template <class T> auto pick2(T t, int) -> typename Id<typename Maybe<T>::type>::type { return t; }
template <class T> int pick2(T, double) { return -2; }

int main() {
  printf("One %d %d Two %d %d pick %d %d %d %d\n", One<int>::v, One<NoType>::v,
         Two<int>::v, Two<NoType>::v, pick(7, 0), pick(NoType(), 0),
         pick2(7, 0), pick2(NoType(), 0));
  return 0;
}
