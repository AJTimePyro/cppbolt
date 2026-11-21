#include <cstdio>
#include <type_traits>

enum E { e0 };
struct S {};

int main() {
  // All supported standard integral types.
  bool b1 = std::is_integral<bool>::value;
  bool b2 = std::is_integral<char>::value;
  bool b3 = std::is_integral<signed char>::value;
  bool b4 = std::is_integral<unsigned char>::value;
  bool b5 = std::is_integral<short>::value;
  bool b6 = std::is_integral<unsigned short>::value;
  bool b7 = std::is_integral<int>::value;
  bool b8 = std::is_integral<unsigned int>::value;
  bool b9 = std::is_integral<long>::value;
  bool b10 = std::is_integral<unsigned long>::value;
  bool b11 = std::is_integral<long long>::value;
  bool b12 = std::is_integral<unsigned long long>::value;
  // cv-qualified forms.
  bool c1 = std::is_integral<const int>::value;
  bool c2 = std::is_integral<volatile int>::value;
  bool c3 = std::is_integral<const volatile int>::value;
  // Non-integral types.
  bool n1 = std::is_integral<float>::value;
  bool n2 = std::is_integral<double>::value;
  bool n3 = std::is_integral<int*>::value;
  bool n4 = std::is_integral<S>::value;
  bool n5 = std::is_integral<E>::value;
  // Variable template.
  bool v1 = std::is_integral_v<int>;
  bool v2 = std::is_integral_v<double>;

  bool integral = b1 && b2 && b3 && b4 && b5 && b6 && b7 && b8 && b9 && b10 && b11 && b12
               && c1 && c2 && c3;
  bool non_integral = !n1 && !n2 && !n3 && !n4 && !n5;
  bool var = v1 && !v2;
  std::printf("%d %d %d\n", (int)integral, (int)non_integral, (int)var);
  return 0;
}
