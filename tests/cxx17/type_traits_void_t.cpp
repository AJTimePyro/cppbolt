#include <cstdio>
#include <type_traits>
#include <utility>

struct HasType { typedef int type; };
struct NoType {};
struct HasFoo { int foo() { return 42; } };
struct NoFoo {};

// Basic void_t use.
template <typename... Ts> using AllVoid = std::void_t<Ts...>;
AllVoid<int, double, char> basic() { return; }

// Nested-type detection idiom (existing / missing).
template <typename T, typename = void> struct has_type : std::false_type {};
template <typename T> struct has_type<T, std::void_t<typename T::type> > : std::true_type {};

// Member-expression detection idiom (valid / invalid).
template <typename T, typename = void> struct has_foo : std::false_type {};
template <typename T> struct has_foo<T, std::void_t<decltype(std::declval<T>().foo())> > : std::true_type {};

// Multiple dependent types in one detection.
template <typename T, typename U, typename = void> struct both_types : std::false_type {};
template <typename T, typename U>
struct both_types<T, U, std::void_t<typename T::type, typename U::type> > : std::true_type {};

int main() {
  basic();
  std::printf("%d %d %d %d %d %d\n",
    (int)has_type<HasType>::value,
    (int)has_type<NoType>::value,
    (int)has_foo<HasFoo>::value,
    (int)has_foo<NoFoo>::value,
    (int)both_types<HasType, HasType>::value,
    (int)both_types<HasType, NoType>::value);
  return 0;
}
