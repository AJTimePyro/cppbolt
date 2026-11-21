#include <cstdio>

struct S { int v; };
int make() { return 11; }
int& lref() { static int v = 13; return v; }
int&& rref() { static int v = 15; return static_cast<int&&>(v); }

int main() {
    int x = 1;
    decltype(x) a = 2;          // int
    decltype((x)) rx = x;       // int&
    rx = 9;
    S s{3};
    decltype(s.v) b = 4;        // int
    decltype((s.v)) rs = s.v;   // int&
    rs = 7;
    decltype(make()) c = make();
    decltype(lref()) d = lref();      // int&
    decltype((lref())) e = lref();    // int&
    decltype(rref()) f = rref();      // int&&
    d = 17;
    e = 19;
    std::printf("%d %d %d %d %d %d %d\n", x, a, s.v, b, c, f, lref());
    return 0;
}
