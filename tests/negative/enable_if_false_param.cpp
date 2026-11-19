#include <type_traits>

void f(std::enable_if<false, int>::type x) {}

int main() {}
