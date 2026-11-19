#include <type_traits>
template <bool B> struct Flag { static const int v = B ? 1 : 0; };
int invalid = Flag<later>::v;
constexpr bool later = std::is_integral<int>::value;
int main() { return 0; }
