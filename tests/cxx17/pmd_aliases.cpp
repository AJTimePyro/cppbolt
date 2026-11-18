#include <cstdio>

struct Real {
    int x;
    int y;
    int get(int a) const { return x + y + a; }
};
using Alias = Real;
typedef int Real::* PMD;
using PMF = int (Alias::*)(int) const;
using PMD2 = decltype(&Real::x);

int main() {
    Real r{2, 3};
    PMD a = &Alias::x;
    PMD2 b = &Real::y;
    PMF f = &Real::get;
    Alias* p = &r;
    int Alias::* pa = &Alias::x;
    std::printf("%d %d %d %d\n", r.*a, r.*b, (r.*f)(4), p->*pa);
    using R = int;
    R (Alias::*g)(int) const = &Alias::get;
    std::printf("%d\n", (r.*g)(1));
    return 0;
}
