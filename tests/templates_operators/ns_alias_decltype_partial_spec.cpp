#include <cstdio>
#include <utility>

template <class T> struct Box { typedef T type; };
struct Has { int get() const { return 7; } };
struct Plain {};

// The decltype specialization is selected and provides `type`.
template <class T, class U> struct WithType { typedef int type; static const int which = 0; };
template <class T> struct WithType<T, typename Box<decltype(std::declval<T>().get())>::type> {
  typedef T type;
  static const int which = 1;
};
static_assert(WithType<Has, int>::which == 1, "specialization must be selected");

using Direct = WithType<Has, int>::type;
template <class T> using One = typename WithType<T, int>::type;
template <class T> using Two = typename Box<One<T> >::type;
using A = One<Has>;
using B = Two<Has>;

// Confirmation fails for Plain, so the primary supplies `type`.
template <class T, class U> struct Fallback { typedef int type; static const int which = 0; };
template <class T> struct Fallback<T, typename Box<decltype(std::declval<T>().get())>::type> {
  static const int which = 1;
};
static_assert(Fallback<Plain, int>::which == 0, "primary must be selected");
using F = Fallback<Plain, int>::type;

// Ordinary partial specialization as a control.
template <class T, class U> struct Ordinary { typedef int type; static const int which = 0; };
template <class T> struct Ordinary<T, int> { typedef T type; static const int which = 1; };
using O = Ordinary<Has, int>::type;

int main() {
  Direct d;
  A a;
  B b;
  F f = 5;
  O o;
  (void)d; (void)a; (void)b; (void)o;
  printf("types %d %d %d %d %d\n", (int)sizeof(Direct), (int)sizeof(A),
         (int)sizeof(B), f, (int)sizeof(O));
  return 0;
}
