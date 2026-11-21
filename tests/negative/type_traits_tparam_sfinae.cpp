#include <type_traits>
template <typename T, typename = std::enable_if_t<std::is_integral<T>::value> > int only_integral(T) { return 1; }
int main() { return only_integral<double>(2.5); }
