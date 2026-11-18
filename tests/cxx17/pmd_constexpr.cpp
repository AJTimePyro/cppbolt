#include <cstdio>

struct S { int x; int f() { return x; } };
constexpr int S::* pm = &S::x;
constexpr int (S::*pf)() = &S::f;
constexpr bool b1 = (pm == &S::x);
constexpr bool b2 = (pm != nullptr);
constexpr bool b3 = (pf == &S::f);
static_assert(b1 && b2 && b3, "constexpr member pointer");

int main() {
    constexpr S s{5};
    static_assert((s.*pm) == 5, "constexpr data member access");
    std::printf("%d %d %d %d\n", b1, b2, b3, s.*pm);
    return 0;
}
