#include <cstdio>
#include <type_traits>
#include <utility>

// Resolve declval's declared type through an alias so it can be compared.
template <typename T> using DeclvalT = decltype(std::declval<T>());

struct S { int m; S(int x) : m(x) {} int get() const { return m; } };

// declval in trailing return types (unevaluated operands).
template <typename T> auto read_m(const T& t) -> decltype(std::declval<const T&>().m) { return t.m; }
template <typename T> auto call_get(const T& t) -> decltype(std::declval<const T&>().get()) { return t.get(); }
template <typename T> auto add(const T& a, const T& b)
    -> decltype(std::declval<const T&>() + std::declval<const T&>()) { return a + b; }

int main() {
  // Reference behavior.
  bool r1 = std::is_same<DeclvalT<int>, int&&>::value;
  bool r2 = std::is_same<DeclvalT<int&>, int&>::value;
  bool r3 = std::is_same<DeclvalT<const int&>, const int&>::value;
  // Member functions, const-qualified expressions, and operator expressions.
  S s(7);
  bool r4 = read_m(s) == 7;
  bool r5 = call_get(s) == 7;
  bool r6 = add(2, 3) == 5;
  std::printf("%d %d %d %d %d %d\n", (int)r1, (int)r2, (int)r3, (int)r4, (int)r5, (int)r6);
  return 0;
}
