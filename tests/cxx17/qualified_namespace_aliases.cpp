#include <cstdio>
#include <type_traits>

// Qualified namespace aliases are resolved with their full names kept intact.

namespace A {
    using Good = std::enable_if<true, int>::type;
    struct S { using type = double; };
    using Nested = S::type;
}

namespace B {
    using Good = std::enable_if<true, char>::type;
    using Ref = A::Good;
}

int main() {
    A::Good a = 1;
    B::Good b = 2;
    B::Ref c = 3;
    A::Nested d = 4.5;
    std::printf("%d %d %d %.1f\n", a, b, c, d);
    return 0;
}
