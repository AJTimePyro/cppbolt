#include <utility>
template <class T> struct Box { typedef T type; };
struct Plain {};
template <class T, class U> struct NoType { static const int which = 0; };
template <class T> struct NoType<T, typename Box<decltype(std::declval<T>().get())>::type> {
  typedef T type;
  static const int which = 1;
};
using Bad = NoType<Plain, int>::type;
int main() { return 0; }
