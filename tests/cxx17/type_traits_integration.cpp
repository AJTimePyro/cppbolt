#include <cstdio>
#include <type_traits>
#include <utility>

struct Good { int member() { return 5; } };
struct Bad {};

// void_t detection using declval on a member expression.
template <typename T, typename = void> struct has_member : std::false_type {};
template <typename T>
struct has_member<T, std::void_t<decltype(std::declval<T>().member())> > : std::true_type {};

// enable_if overload whose condition depends on is_integral_v.
template <typename T> std::enable_if_t<std::is_integral_v<T>, int> dispatch(T) { return 1; }
template <typename T> std::enable_if_t<!std::is_integral_v<T>, int> dispatch(T) { return 2; }

// Detection result feeding another enable_if condition.
template <typename T> std::enable_if_t<has_member<T>::value, int> probe(T& t) { return t.member(); }
template <typename T> std::enable_if_t<!has_member<T>::value, int> probe(T&) { return -1; }

int main() {
  Good g;
  Bad b;
  std::printf("%d %d %d %d %d %d\n",
    (int)has_member<Good>::value,
    (int)has_member<Bad>::value,
    dispatch(1), dispatch(1.5),
    probe(g), probe(b));
  return 0;
}
