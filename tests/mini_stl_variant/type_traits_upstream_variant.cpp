#include <cstdio>
#include <type_traits>
#include <utility>

// Dependent-context and SFINAE use of the integrated upstream traits.
struct HasType { typedef int type; };

template <typename T, typename = void> struct has_nested : std::false_type {};
template <typename T>
struct has_nested<T, std::void_t<typename T::type> > : std::true_type {};

template <typename T> std::enable_if_t<std::is_integral_v<T>, int> dispatch(T) { return 1; }
template <typename T> std::enable_if_t<!std::is_integral_v<T>, int> dispatch(T) { return 2; }

template <typename T> using NoRef = std::remove_reference_t<T>;
template <typename T> using Ptr = std::add_pointer_t<T>;

int main() {
  // Alias templates in dependent contexts.
  NoRef<int&> a = 4;
  NoRef<const char*&> msg = "upstream traits";
  Ptr<int> p = &a;

  // Detection idiom fed by the trait's own base/value.
  int detected = (int)has_nested<HasType>::value
               + (int)has_nested<int>::value;

  // SFINAE overload selection through variable templates.
  int d1 = dispatch(1);
  int d2 = dispatch(1.5);

  std::printf("%d %s %d %d %d %d\n", a, msg, *p, detected, d1, d2);
  return 0;
}
