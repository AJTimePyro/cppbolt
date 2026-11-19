#include <type_traits>

using Invalid = std::enable_if<false, int>::type;

int main() {}
