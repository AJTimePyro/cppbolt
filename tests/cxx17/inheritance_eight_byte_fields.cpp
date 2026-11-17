#include <cstdio>

struct PadLL { long long x; };
struct PadD { double d; };
struct Poly { int p; virtual ~Poly() {} virtual int value() { return p; } };

// 8-byte field base before the polymorphic base.
struct LLThenPoly : PadLL, Poly {
    int own;
    LLThenPoly() : own(7) { x = 3; p = 4; }
    int value() override { return p + own; }
};

// Polymorphic base before the 8-byte field base.
struct PolyThenLL : Poly, PadLL {
    int own;
    PolyThenLL() : own(9) { x = 5; p = 6; }
};

// double field, polymorphic base second.
struct DoubleThenPoly : PadD, Poly {
    int value() override { return 11; }
};

int main() {
    LLThenPoly a;
    LLThenPoly b = a;             // synthesized copy ctor
    std::printf("%lld %d\n", b.x, b.value());

    PolyThenLL c;
    PolyThenLL d = c;
    Poly* pc = &d;
    std::printf("%lld %d\n", d.x, pc->value());

    DoubleThenPoly e;
    DoubleThenPoly f = e;
    std::printf("%.1f %d\n", f.d, f.value());
    return 0;
}
