#include <cstdio>

struct S {
    int x;
    int arr[3];
    int c() const { return 1; }
    int v() volatile { return 2; }
    int cv() const volatile { return 3; }
    int ne() noexcept { return 4; }
    int f() { return 5; }
    int f() const { return 6; }
};

int main() {
    S s{};
    s.arr[1] = 9;
    int (S::*pc)() const = &S::c;
    int (S::*pv)() volatile = &S::v;
    int (S::*pcv)() const volatile = &S::cv;
    int (S::*pne)() noexcept = &S::ne;
    int (S::*pplain)() = &S::f;
    int (S::*pconst)() const = &S::f;
    int (S::*parr)[3] = &S::arr;
    std::printf("%d %d %d %d\n", (s.*pc)(), (s.*pv)(), (s.*pcv)(), (s.*pne)());
    std::printf("%d %d %d\n", (s.*pplain)(), (s.*pconst)(), (s.*parr)[1]);
    return 0;
}
