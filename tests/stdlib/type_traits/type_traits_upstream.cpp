#include <cstdio>
#include <type_traits>

// Upstream libc++ 23.1.5 <type_traits> integration: primary categories,
// transformations, properties and variable templates.
struct S { int m; void f() {} };
enum E { e0 };

using std::is_same;

int main() {
  // integral_constant family and alias templates.
  bool helpers = std::is_same<std::true_type, std::integral_constant<bool, true> >::value
              && std::is_same<std::false_type, std::integral_constant<bool, false> >::value
              && std::bool_constant<true>::value
              && std::true_type::value && !std::false_type::value;

  // Primary classification traits.
  bool categories = std::is_void<void>::value && std::is_void<const void>::value
                 && std::is_integral<int>::value && std::is_integral<const unsigned>::value
                 && std::is_floating_point<double>::value
                 && std::is_null_pointer<decltype(nullptr)>::value
                 && std::is_array<int[3]>::value && std::is_pointer<int*>::value
                 && std::is_lvalue_reference<int&>::value
                 && std::is_rvalue_reference<int&&>::value && std::is_reference<int&>::value
                 && std::is_const<const int>::value
                 && std::is_member_pointer<int S::*>::value
                 && std::is_member_function_pointer<void (S::*)()>::value;

  // Composite categories.
  bool composite = std::is_arithmetic<int>::value && std::is_arithmetic<float>::value
                && std::is_fundamental<void>::value && !std::is_fundamental<S>::value
                && std::is_compound<S>::value && !std::is_compound<int>::value
                && std::is_signed<int>::value && std::is_unsigned<unsigned>::value
                && !std::is_integral<E>::value && !std::is_integral<S>::value
                && !std::is_integral<float>::value && !std::is_pointer<int>::value;

  // Variable templates.
  bool variables = std::is_integral_v<int> && !std::is_integral_v<double>
                && std::is_same_v<int, int> && std::is_void_v<void>
                && std::is_array_v<int[]> && std::is_floating_point_v<float>
                && std::is_signed_v<long> && std::is_unsigned_v<unsigned long>;

  // Transformations and type relationships.
  bool transforms = is_same<typename std::remove_cv<const int>::type, int>::value
                 && is_same<typename std::remove_reference<int&>::type, int>::value
                 && is_same<typename std::add_lvalue_reference<int>::type, int&>::value
                 && is_same<typename std::add_rvalue_reference<int>::type, int&&>::value
                 && is_same<typename std::remove_pointer<int*>::type, int>::value
                 && is_same<typename std::add_pointer<int>::type, int*>::value
                 && is_same<typename std::remove_extent<int[3]>::type, int>::value
                 && is_same<typename std::remove_all_extents<int[2][3]>::type, int>::value
                 && is_same<typename std::decay<int[3]>::type, int*>::value
                 && is_same<typename std::make_signed<unsigned>::type, int>::value
                 && is_same<typename std::make_unsigned<int>::type, unsigned>::value;

  // Property queries.
  bool properties = std::rank<int[2][3]>::value == 2
                 && std::alignment_of<int>::value >= 1;

  // Logical helper traits.
  bool logical = std::negation<std::false_type>::value
              && !std::negation<std::true_type>::value;

  std::printf("%d %d %d %d %d %d %d\n", (int)helpers, (int)categories, (int)composite,
              (int)variables, (int)transforms, (int)properties, (int)logical);
  return 0;
}
