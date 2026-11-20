#include <utility>
template <class T> struct Box { typedef T type; };
struct Yes { int get() const { return 7; } };

template <class T, class U> struct A { typedef int type; static const int which = 0; };
template <class T> struct A<T, typename Box<decltype(std::declval<T>().get())>::type> {
  static const int which = 1;
};

int main() {
  A<Yes, int>::type x = 0;
  return x;
}
