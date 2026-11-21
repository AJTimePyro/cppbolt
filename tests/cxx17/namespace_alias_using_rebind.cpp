#include <cstdio>

namespace A { int g(int) { return 1; } }
namespace B { double g(double) { return 2; } }

using A::g;
using Alias = decltype(g(1));
using B::g;

int main() {
    printf("%zu\n", sizeof(Alias));
    return 0;
}
