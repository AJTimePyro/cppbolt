#include <type_traits>
template <typename T>
typename std::enable_if<std::is_integral<T>::value, int>::type f(T) { return 1; }
int main() { return f(1.5); }
