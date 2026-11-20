#include <cstdio>
#include <type_traits>

// Chains of dependent nested `::type` resolve to the final type in template
// type arguments, non-type arguments, variable templates and enable_if.
// Verified against Clang 23.1.5 and GCC 16.2.1.

template <class T> struct Id { typedef T type; };
template <class T> struct Twice { typedef typename Id<T>::type type; };
template <class T> using IdT = typename Id<T>::type;
template <class T> using TwiceT = typename Twice<typename Id<T>::type>::type;
template <class T> using TripleT = typename Twice<typename Twice<typename Id<T>::type>::type>::type;
template <class T> struct Box { typedef typename Id<T>::type type; static const int value = 11; };

template <class T> struct One { static const int v = std::is_same<typename Id<T>::type, T>::value; };
template <class T> struct Two { static const int v = std::is_same<TwiceT<T>, T>::value; };
template <class T> struct Nttp {
  static const int v = std::integral_constant<int, Box<TripleT<T> >::value>::value;
};
template <class T> struct VarT { static const int v = std::is_integral_v<TwiceT<T> >; };
template <class T> struct Enable {
  static const int v = std::is_same<typename std::enable_if<std::is_integral<TwiceT<T> >::value, int>::type, int>::value;
};
template <class T> struct Qual { static const int v = std::is_same<typename Box<IdT<T> >::type, T>::value; };
template <class T> struct Stat { static const int v = Box<IdT<T> >::value; };
template <class T> typename Id<typename Id<T>::type>::type f(T t, int*) { return t; }
template <class T> int f(T, ...) { return -1; }

int main() {
  printf("One %d Two %d Nttp %d VarT %d Enable %d Qual %d Stat %d\n",
         One<int>::v, Two<int>::v, Nttp<int>::v, VarT<int>::v,
         Enable<int>::v, Qual<int>::v, Stat<int>::v);
  printf("f %d direct %d %d\n", f(3, 0), std::is_same<TripleT<int>, int>::value,
         std::is_integral<TripleT<int> >::value);
  return 0;
}
