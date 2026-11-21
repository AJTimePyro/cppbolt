#include <cstdio>
#include <type_traits>

// True/false conditions and default/explicit second template argument.
template <bool B, typename T = void> struct MyEnable { typedef T type; };
template <typename T> struct MyEnable<false, T> {};
template <bool B, typename T = void> using MyEnableT = typename MyEnable<B, T>::type;

// Return-type SFINAE: exactly one of the two candidates is viable.
template <typename T> std::enable_if_t<std::is_integral<T>::value, int> chosen(T) { return 1; }
template <typename T> std::enable_if_t<!std::is_integral<T>::value, int> chosen(T) { return 2; }

// Parameter-type SFINAE.
template <typename T> int param(T, std::enable_if_t<std::is_integral<T>::value>* = nullptr) { return 1; }
template <typename T> int param(T, std::enable_if_t<!std::is_integral<T>::value>* = nullptr) { return 2; }

// Partial-specialization SFINAE.
template <typename T, typename = void> struct Pick { static const int value = 0; };
template <typename T> struct Pick<T, MyEnableT<std::is_integral<T>::value> > { static const int value = 1; };

int main() {
  MyEnable<true, int>::type a = 11;
  MyEnableT<true, double> b = 2.5;
  MyEnable<true>::type* c = nullptr;
  std::printf("%d %.1f %d %d %d %d %d %d\n",
    a, b, c == nullptr,
    chosen(1), chosen(1.5),
    param(3), param(2.5),
    Pick<int>::value);
  return 0;
}
