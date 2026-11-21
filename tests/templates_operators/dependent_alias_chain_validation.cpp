#include <cstdio>
#include <type_traits>

// Valid one-, two- and three-level dependent nested alias chains, both at
// namespace scope and after substitution inside a template.
template <class T> struct Id { typedef T type; };
template <class T> struct Maybe { typedef T type; };

template <class T> using One = typename Maybe<T>::type;
template <class T> using Two = typename Id<typename Maybe<T>::type>::type;
template <class T> using Three = typename Id<typename Id<typename Maybe<T>::type>::type>::type;

using A = One<int>;
using B = Two<int>;
using C = Three<int>;

template <class T> struct Check {
  static const int v = std::is_same<One<T>, T>::value && std::is_same<Two<T>, T>::value &&
                       std::is_same<Three<T>, T>::value;
};

int main() {
  A a = 1;
  B b = 2;
  C c = 3;
  printf("%d %d %d %d\n", a, b, c, Check<int>::v);
  return 0;
}
