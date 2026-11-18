#include <cstdio>

struct S {
    int x;
    int y;
    int f() { return 1; }
    int g() { return 2; }
};

int main() {
    // Integer literal zero is a null pointer constant for member pointers.
    int S::* pd0 = 0;
    int S::* pdnull = nullptr;
    int S::* pdval{};
    int (S::* pf0)() = 0;
    int (S::* pfnull)() = nullptr;
    int (S::* pfval)(){};

    std::printf("%d %d %d\n", pd0 == pdnull, pdval == pdnull, pd0 == nullptr);
    std::printf("%d %d %d\n", pf0 == pfnull, pfval == pfnull, pf0 == nullptr);

    int S::* px = &S::x;
    int S::* py = &S::y;
    int (S::* pf)() = &S::f;
    int (S::* pg)() = &S::g;
    std::printf("%d %d %d %d\n", px == px, px == py, px == 0, 0 == px);
    std::printf("%d %d %d %d\n", pf == pf, pf == pg, pf == 0, 0 == pf);

    // Assignment and copy through a function parameter.
    int S::* assigned = &S::y;
    assigned = 0;
    std::printf("%d\n", assigned == pdnull);

    S s{7, 8};
    std::printf("%d %d\n", s.*px, (s.*pf)());
    return 0;
}
