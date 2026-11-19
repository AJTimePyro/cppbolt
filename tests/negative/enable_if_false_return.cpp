#include <type_traits>

std::enable_if<false, int>::type f() { return 0; }

int main() {}
