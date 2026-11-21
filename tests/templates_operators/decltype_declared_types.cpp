#include <cstdio>

struct S { int v; int& r; S(int& x) : v(3), r(x) {} };

int main() {
    int x = 1;
    int& alias = x;
    decltype(alias) a = x;   // int&, not int
    a = 5;
    S s(x);
    decltype(s.r) b = x;     // int&, not int
    b = 7;
    const S cs(x);
    decltype(cs.v) c = 1;    // int, not const int
    c = 9;
    std::printf("%d %d %d %d\n", x, a, b, c);
    return 0;
}
