#include <cstdio>

struct Pad { int pad; };
struct B { int b; int f() { return b; } };
struct Single : B { int d; };
struct Multi : Pad, B { int d; };
struct P0 { int p0; };
struct Mid : P0, B { int m; };
struct Deep : Pad, Mid { int d; };

int main() {
    Single s{}; s.b = 2; s.d = 3;
    int B::* pb = &B::b;
    int (B::* pf)() = &B::f;
    std::printf("%d %d\n", s.*pb, (s.*pf)());

    Multi m{}; m.pad = 1; m.b = 2; m.d = 3;
    std::printf("%d %d\n", m.*pb, (m.*pf)());
    Multi* mp = &m;
    std::printf("%d %d\n", mp->*pb, (mp->*pf)());

    // Applying an already-converted pointer must not adjust twice.
    int Multi::* pmb = pb;
    int (Multi::* pmf)() = pf;
    std::printf("%d %d\n", m.*pmb, (m.*pmf)());

    Deep e{}; e.p0 = 1; e.b = 2; e.m = 3; e.pad = 4; e.d = 5;
    int Mid::* pmm = pb;
    int Deep::* pmd = pmm;
    int (Mid::* pfm)() = pf;
    int (Deep::* pfd)() = pfm;
    std::printf("%d %d %d %d\n", e.*pb, e.*pmm, e.*pmd, (e.*pfd)());
    return 0;
}
