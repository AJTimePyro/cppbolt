#include <type_traits>

static_assert(!std::is_integral_v<int>, "int is integral");

int main() { return 0; }
