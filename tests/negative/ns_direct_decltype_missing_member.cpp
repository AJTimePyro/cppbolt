#include <utility>
template <class T> struct Box { typedef T type; };
struct Has { int get() const { return 7; } };
template <class T, class U> struct NoType { typedef int type; static const int which = 0; };
template <class T> struct NoType<T, typename Box<decltype(std::declval<T>().get())>::type> {
  static const int which = 1;
};
int main() { NoType<Has, int>::type x = 0; return x; }
